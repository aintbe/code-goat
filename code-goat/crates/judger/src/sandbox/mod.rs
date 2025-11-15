pub(crate) mod seccomp;

use std::{
    cmp, env,
    ops::{Add, Div},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cgroups_rs::{
    CgroupPid,
    fs::{
        Cgroup, cgroup_builder::CgroupBuilder, cpu::CpuController, hierarchies,
        memory::MemController,
    },
};
use log::{debug, error, info};
use nix::{
    mount::{self, MsFlags},
    sys::{
        resource::{self, Resource},
        signal::{self, Signal},
    },
    unistd::{self, Pid},
};

use crate::models::{InternalError, ResourceLimit};

const MEBI_BYTE: u32 = 1 << 10 << 10;
const MEGA_BYTE: u32 = 1000 * 1000;

pub(crate) struct CgroupSandbox {
    inner: Cgroup,
}

impl CgroupSandbox {
    const CGROUP_NAME: &str = "code-goat";

    pub(crate) fn new(resource_limit: &ResourceLimit) -> Result<CgroupSandbox, InternalError> {
        let builder = CgroupBuilder::new(Self::CGROUP_NAME)
            // Forces processes in this cgroup to use CPU up to 100%.
            .cpu()
            .period(100 * 1000)
            .quota(100 * 1000)
            .done()
            // Minimize memory swapping.
            .memory()
            .swappiness(0);

        let cgroup = if let Some(limit) = resource_limit.memory {
            // Limit memory usage if specified.
            builder.memory_hard_limit(
                // Add margin of 4MiB to detect MLE.
                limit.add(4 * MEBI_BYTE).into(),
            )
        } else {
            builder
        }
        .done()
        .build(hierarchies::auto())
        // Return error if cgroup creation fails.
        .map_err(InternalError::CreateCgroup)?;

        Ok(CgroupSandbox { inner: cgroup })
    }

    pub(crate) fn add_process(&self, pid: Pid) -> Result<(), InternalError> {
        let cgroup_pid = CgroupPid::from(pid.as_raw() as u64);
        self.inner
            .add_task_by_tgid(cgroup_pid)
            .map_err(InternalError::AddToCgroup)
    }

    pub(crate) fn read_memory_usage(&self) -> Result<u64, InternalError> {
        let controller = self
            .inner
            .controller_of::<MemController>()
            .ok_or(InternalError::ReadCgroupMemoryStats)?;

        Ok(controller.memory_stat().max_usage_in_bytes)
    }

    pub(crate) fn read_cpu_time_usage(&self) -> Result<u32, InternalError> {
        let cpu = self
            .inner
            .controller_of::<CpuController>()
            .ok_or(InternalError::ReadCgroupCpuStats)?
            .cpu();

        let cpu_stat: Vec<&str> = cpu
            .stat
            .lines()
            .find(|line| line.starts_with("usage_usec"))
            .ok_or(InternalError::ReadCgroupCpuStats)?
            .split_whitespace()
            .collect();

        let cpu_time_in_us: u32 = cpu_stat
            .get(1)
            .ok_or(InternalError::ReadCgroupCpuStats)?
            .parse()
            .map_err(|_| InternalError::ReadCgroupCpuStats)?;

        Ok(cpu_time_in_us / 1000)
    }
}

impl Drop for CgroupSandbox {
    fn drop(&mut self) {
        if let Err(e) = self.inner.delete() {
            error!("Failed to delete cgroup: {:?}", e);
            return;
        }
    }
}

pub(crate) struct TimeSandbox {
    handle: Option<JoinHandle<bool>>,
    runner_exit_tx: Sender<bool>,
}

impl TimeSandbox {
    pub(crate) fn new(runner_pid: Pid, limit: u32) -> Self {
        let (runner_exit_tx, runner_exit_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            runner_exit_rx
                // If runner process exits before timeout, [`Drop`] will send
                // a message to this channel to stop waiting.
                .recv_timeout(Duration::from_millis(calculate_timeout(limit)))
                // If timeout occurs, kill the runner process.
                .is_err_and(|_| {
                    info!("Kill runner process due to timeout.");
                    signal::kill(runner_pid, Signal::SIGKILL).is_ok()
                })
        });

        Self {
            handle: Some(handle),
            runner_exit_tx,
        }
    }
}

impl Drop for TimeSandbox {
    fn drop(&mut self) {
        if let Ok(_) = self.runner_exit_tx.send(true) {
            info!("Runner process exited before timeout; Cancel timeout killer.");
        }
        if let Some(handle) = self.handle.take() {
            if let Ok(_) = handle.join() {
                debug!("Reaped time sandbox.");
            }
        }
    }
}

/// Mount runner process into a safe mount namespace.
pub(crate) fn mount_sandbox() -> Result<(), nix::Error> {
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
/// because the work is done by [`CgroupSandbox`]. Add extra bytes/time to
/// limit to avoid `JudgeStatus::RuntimeError` that cannot be traced.
pub(crate) fn set_limit_to_sandbox(resource_limit: &ResourceLimit) -> Result<(), nix::Error> {
    if let Some(limit_ms) = resource_limit.cpu_time {
        let limit_s = calculate_timeout(limit_ms) / 1000;
        resource::setrlimit(Resource::RLIMIT_CPU, limit_s, limit_s)?;
    };

    if let Some(limit) = resource_limit.n_process {
        let limit = limit.into();
        resource::setrlimit(Resource::RLIMIT_NPROC, limit, limit)?;
    };

    if let Some(limit) = resource_limit.stack {
        let limit = limit.add(4 * MEBI_BYTE).into();
        resource::setrlimit(Resource::RLIMIT_STACK, limit, limit)?;
    };

    if let Some(limit) = resource_limit.output {
        let limit = limit.add(1 * MEGA_BYTE).into();
        resource::setrlimit(Resource::RLIMIT_FSIZE, limit, limit)?;
    };

    Ok(())
}

fn calculate_timeout<T>(limit: T) -> u64
where
    T: Copy + Ord + Div<Output = T> + From<u8> + Add<Output = T> + Into<u64>,
{
    let margin = cmp::max(limit / T::from(20), T::from(1)); // 5% or 1ms
    (limit + margin).into()
}
