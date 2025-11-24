pub(crate) mod seccomp;

use std::{
    cmp,
    convert::Infallible,
    env,
    ops::{Add, Div},
    path::Path,
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
                // Add margin of 1MiB to detect MLE.
                limit.saturating_add(MEBI_BYTE).into(),
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
        // TODO: delete clocks here after testing.
        let clock = Instant::now();

        let cgroup_pid = CgroupPid::from(pid.as_raw() as u64);
        let res = self
            .inner
            .add_task_by_tgid(cgroup_pid)
            .map_err(InternalError::AddToCgroup);

        let duration = clock.elapsed();
        debug!("Duration to add process to cgroup: {:?}", duration);

        res
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
            .or(Err(InternalError::ReadCgroupCpuStats))?;

        Ok(cpu_time_in_us / 1000)
    }
}

impl Drop for CgroupSandbox {
    fn drop(&mut self) {
        // TODO: delete clocks here after testing.
        let clock = Instant::now();

        if let Err(e) = self.inner.delete() {
            error!("Failed to delete cgroup: {:?}", e);
            return;
        } else {
            debug!("Deleted cgroup successfully.");
        }

        let duration = clock.elapsed();
        debug!("Duration to delete cgroup: {:?}", duration,);
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
                .recv_timeout(Duration::from_millis(calculate_timeout(limit, 10)))
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
            info!("Runner process exited before timeout; canceling timeout killer...");
        }
        if let Some(handle) = self.handle.take() {
            if let Ok(_) = handle.join() {
                debug!("Reaped time sandbox.");
            }
        }
    }
}

const SENSITIVE_DIRS: [&str; 11] = [
    // NOTE: The following directories are not masked because they have...
    // "/bin",              // Core commands
    // "/lib", "/lib64"     // Shared libraries
    // "/proc",             // Process and system information
    // "/tmp",              // Temporary files
    // "/usr",              // User binaries and read-only data
    // "/var",              // Variable data files
    //
    "/boot", // Kernel images, GRUB configuration files, etc.
    "/dev",  // Hardware devices as files
    "/etc",  // System configuration files
    "/home", // User home directories
    "/mnt",  // Used by administrators for mounting filesystems
    "/opt",  // Third-party software packages
    "/root", // Root user's home directory
    "/run",  // Runtime variable data
    "/sbin", // System binaries
    "/srv",  // Data for services provided by the system
    "/sys",  // System and kernel information
];

/// Mount runner process into a safe mount namespace.
pub(crate) fn mount_sandbox() -> Result<(), Infallible> {
    // Make mount namespace private to avoid affecting the host system.
    mount::mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
    .expect("Failed to make mount namespace private.");

    // Remount root filesystem as read-only.
    mount::mount(
        Some("/"),
        "/",
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_REC,
        Some("mode=000"),
    )
    .expect("Failed to remount root filesystem as read-only.");

    // Mount empty space for sensitive directories.
    for dir_path in SENSITIVE_DIRS {
        if Path::new(dir_path).is_dir() {
            mount::mount(
                Some("tmpfs"),
                dir_path,
                Some("tmpfs"),
                MsFlags::empty(),
                Some("size=2m,mode=000"),
            )
            .expect(&format!(
                "Failed to mount empty tmpfs to sensitive directory {}.",
                dir_path
            ));
        }
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
        )
        .expect("Failed to remount workspace as writable.");
        unistd::chdir(workspace).expect("Failed to change directory to workspace.");
    }

    Ok(())
}

/// Set resource limits to the sandbox. Memory usage is not limited here
/// because the work is done by [`CgroupSandbox`]. Add extra bytes/time to
/// limit to avoid `JudgeStatus::RuntimeError` that cannot be traced.
pub(crate) fn set_limit_to_sandbox(resource_limit: &ResourceLimit) -> Result<(), nix::Error> {
    if let Some(limit_ms) = resource_limit.cpu_time {
        // Ensure to add at least 1 second to avoid immediate termination.
        let limit_s = calculate_timeout(limit_ms / 1000, 1).into();
        resource::setrlimit(Resource::RLIMIT_CPU, limit_s, limit_s)?;
    };

    if let Some(limit) = resource_limit.n_process {
        let limit = limit.into();
        resource::setrlimit(Resource::RLIMIT_NPROC, limit, limit)?;
    };

    if let Some(limit) = resource_limit.stack {
        let limit = limit.add(MEBI_BYTE).into();
        resource::setrlimit(Resource::RLIMIT_STACK, limit, limit)?;
    };

    if let Some(limit) = resource_limit.output {
        let limit = limit.add(MEGA_BYTE).into();
        resource::setrlimit(Resource::RLIMIT_FSIZE, limit, limit)?;
    };

    Ok(())
}

fn calculate_timeout<T>(limit: T, min_margin: T) -> u64
where
    T: Copy + Ord + Div<Output = T> + From<u8> + Add<Output = T> + Into<u64>,
{
    let margin = cmp::max(limit / T::from(20), min_margin); // 5% or `min_margin`
    (limit + margin).into()
}

#[cfg(test)]
mod tests;
