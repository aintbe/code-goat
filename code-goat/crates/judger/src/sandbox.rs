use std::{
    cmp, env,
    io::{PipeReader, PipeWriter},
    ops::Add,
};

use cgroups_rs::{
    CgroupPid,
    fs::{Cgroup, cgroup_builder::CgroupBuilder, hierarchies, memory::MemController},
};
use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall, error::SeccompError};
use log::error;
use nix::{
    mount::{self, MsFlags},
    sched::{self, CloneFlags},
    sys::{
        resource::{self, Resource},
        signal::Signal,
    },
    unistd::{self, Pid},
};

use crate::{
    runner,
    spec::{InternalError, ResourceLimit, RunSpec},
};

const MEGA_BYTE: u64 = 1_000_000;

pub struct CgroupSandbox {
    inner: Cgroup,
}

impl CgroupSandbox {
    pub fn new(resource_limit: &ResourceLimit) -> Result<CgroupSandbox, InternalError> {
        const CGROUP_NAME: &str = "code-goat";

        let builder = CgroupBuilder::new(CGROUP_NAME)
            // Forces processes in this cgroup to use CPU up to 100%.
            .cpu()
            // .period(100 * 1000)
            // .quota(100 * 1000)
            .done()
            // Minimize memory swapping.
            .memory()
            .swappiness(0);

        let cgroup = if let Some(limit) = resource_limit.memory {
            // Limit memory usage if specified.
            builder.memory_hard_limit(limit.add(16 * MEGA_BYTE as u32).into())
        } else {
            builder
        }
        .done()
        .build(hierarchies::auto())
        // Return error if cgroup creation fails.
        .map_err(InternalError::CreateCgroup)?;

        Ok(CgroupSandbox { inner: cgroup })
    }

    pub fn add_process(&self, pid: Pid) -> Result<(), InternalError> {
        let cgroup_pid = CgroupPid::from(pid.as_raw() as u64);
        self.inner
            .add_task_by_tgid(cgroup_pid)
            .map_err(InternalError::AddToCgroup)
    }

    pub fn read_memory_usage(&self) -> Result<u64, InternalError> {
        let controller = self
            .inner
            .controller_of::<MemController>()
            .ok_or_else(|| InternalError::ReadCgroupStat)?;

        Ok(controller.memory_stat().max_usage_in_bytes)
    }
}

impl Drop for CgroupSandbox {
    fn drop(&mut self) {
        if let Err(e) = self.inner.delete() {
            error!("Failed to delete cgroup: {:?}", e);
        }
    }
}

/// Clone a new process with specified namespaces.
/// Returns the PID of the cloned process.
pub fn clone_runner(
    spec: &RunSpec,
    setup_rx: PipeReader,
    abort_tx: PipeWriter,
) -> Result<Pid, InternalError> {
    let runner = {
        Box::new(|| match runner::run(&spec, &setup_rx, &abort_tx) {
            Ok(status) => status,
            Err(e) => e as isize,
        })
    };

    // `unistd::clone` requires a stack pointer, so we allocate the stack
    // on the heap. However, since a new stack is allocated again when
    // `unistd::execv` is executed after specifying the stack during clone,
    // we only have to allocate a stack large enough for the child process.
    const STACK_SIZE: usize = 1024 * 1024; // 1MB
    let mut stack = vec![0u8; STACK_SIZE].into_boxed_slice();

    let flags = CloneFlags::CLONE_NEWUSER
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWUTS;

    // Let parent notified when cloned process is terminated.
    let signal = Some(Signal::SIGCHLD as i32);

    // Return the PID of the cloned process.
    unsafe { sched::clone(runner, &mut stack, flags, signal) }.map_err(InternalError::Clone)
}

/// Mount runner process into a safe mount namespace.
pub fn mount_sandbox() -> Result<(), nix::Error> {
    // Make mount namespace private to avoid affecting the host system.
    mount::mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )?;

    // Remount root filesystem as read-only.
    mount::mount(
        Some("/"),
        "/",
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_REC,
        Some("mode=000"),
    )?;

    // Mount empty space for sensitive directories
    const SENSITIVE_DIRS: [&str; 4] = [
        "/etc", "/root", "/var",
        "/home",
        // "/lib", "/lib64" // Has important shared libraries
        // "/usr"           // Has basic commands & binaries like JVM
    ];
    for dir_path in SENSITIVE_DIRS {
        mount::mount(
            Some("tmpfs"),
            dir_path,
            Some("tmpfs"),
            MsFlags::empty(),
            Some("size=2m,mode=000"),
        )?;
    }

    // Remount working directory to be writable for logging.
    if let Ok(workspace) = env::var("SANDBOX_WORKSPACE") {
        let workspace = workspace.as_str();
        mount::mount(
            Some(workspace),
            workspace,
            None::<&str>,
            MsFlags::MS_BIND | MsFlags::MS_REC,
            None::<&str>,
        )?;
        unistd::chdir(workspace)?;
    }

    Ok(())
}

/// Set resource limits to the sandbox. Memory usage is not limited here
/// because the work is done by `CgroupSandbox`. Add extra bytes/time to
/// limit to avoid `RuntimeError` that cannot be traced.
pub fn limit_sandbox(resource_limit: &ResourceLimit) -> Result<(), nix::Error> {
    if let Some(limit_ms) = resource_limit.cpu_time {
        let margin = cmp::max(limit_ms / 10, 1); // 10% or 1ms
        let limit = limit_ms.add(margin) / 1000;
        resource::setrlimit(Resource::RLIMIT_CPU, limit, limit)?;
    };

    if let Some(limit) = resource_limit.n_process {
        resource::setrlimit(Resource::RLIMIT_NPROC, limit, limit)?;
    };

    if let Some(limit) = resource_limit.stack {
        let limit = limit.add(16 * MEGA_BYTE);
        resource::setrlimit(Resource::RLIMIT_STACK, limit, limit)?;
    };

    if let Some(limit) = resource_limit.output {
        let limit = limit.add(1 * MEGA_BYTE);
        resource::setrlimit(Resource::RLIMIT_FSIZE, limit, limit)?;
    };

    Ok(())
}

/// Apply seccomp whitelist.
/// TODO: more comment
pub fn apply_seccomp() -> Result<(), SeccompError> {
    const WHITELIST: [&str; 26] = [
        "access",
        "arch_prctl",
        "brk",
        "clock_gettime",
        "close",
        "execve",
        "exit_group",
        "faccessat",
        "fstat",
        "futex",
        "getrandom",
        "lseek",
        "mmap",
        "mprotect",
        "munmap",
        "newfstatat",
        "open",
        "openat",
        "prlimit64",
        "read",
        "readlink",
        "readlinkat",
        "rseq",
        "set_robust_list",
        "set_tid_address",
        "write",
    ];

    // Kill process if the runner provokes any syscall not whitelisted.
    let mut filter = ScmpFilterContext::new(ScmpAction::KillProcess)?;
    for syscall_name in WHITELIST {
        let syscall = ScmpSyscall::from_name(syscall_name)?;
        filter.add_rule(ScmpAction::Allow, syscall)?;
    }

    filter.load()
}
