use std::{ffi::CString, io, iter};

use serde::Serialize;

#[derive(Debug)]
pub struct RunSpec {
    /// Absolute path to the executable file.
    pub exe_path: CString,

    /// Absolute path to the input file (for stdin redirection).
    pub input_path: Option<String>,

    /// Absolute path to the output file (for stdout redirection).
    pub output_path: Option<String>,

    /// Absolute path to the error file (for stderr redirection).
    pub error_path: Option<String>,

    /// Absolute path to the error file (for stderr redirection).
    pub answer_path: Option<String>,

    /// List of arguments to pass to the program.
    /// Example: ["/usr/bin/echo", "hello"]
    pub args: Vec<CString>,

    /// Environment variables to set for the program.
    /// Example: ["PATH=/usr/bin"]
    pub envs: Vec<CString>,

    /// The judging policy (resource usage limits) to apply.
    pub resource_limit: ResourceLimit,
}

impl<'a> RunSpec {
    pub fn from_cstr(
        exe_path: CString,
        input_path: Option<String>,
        output_path: Option<String>,
        error_path: Option<String>,
        answer_path: Option<String>,
        args: Vec<CString>,
        envs: Vec<CString>,
        resource_limit: ResourceLimit,
    ) -> Self {
        let is_exe_missing = match &args.get(0) {
            Some(first) => **first != exe_path,
            None => true,
        };
        let full_args = if is_exe_missing {
            iter::once(exe_path.clone()).chain(args).collect()
        } else {
            args
        };

        Self {
            exe_path,
            input_path,
            output_path,
            error_path,
            answer_path,
            args: full_args,
            envs,
            resource_limit,
        }
    }

    pub fn new(
        exe_path: &str,
        input_path: Option<&str>,
        output_path: Option<&str>,
        error_path: Option<&str>,
        answer_path: Option<&str>,
        args: Vec<&str>,
        envs: Vec<&str>,
        resource_limit: ResourceLimit,
    ) -> Self {
        let exe_path_cstr = CString::new(exe_path).unwrap();

        let is_exe_missing = match args.get(0) {
            Some(&first) => first != exe_path,
            None => true,
        };
        let args = args.into_iter().map(|arg| CString::new(arg).unwrap());
        let args_cstr = if is_exe_missing {
            iter::once(exe_path_cstr.clone()).chain(args).collect()
        } else {
            args.collect()
        };

        let envs_cstr = envs
            .into_iter()
            .map(|env| CString::new(env).unwrap())
            .collect();

        Self {
            exe_path: exe_path_cstr,
            input_path: input_path.map(String::from),
            output_path: output_path.map(String::from),
            answer_path: answer_path.map(String::from),
            error_path: error_path.map(String::from),
            args: args_cstr,
            envs: envs_cstr,
            resource_limit,
        }
    }
}

#[derive(Debug)]
pub struct ResourceLimit {
    /// Peak memory usage in bytes.
    pub memory: Option<u32>,

    /// CPU time used in milliseconds.
    pub cpu_time: Option<u64>,

    /// Real time used in milliseconds.
    pub real_time: Option<u32>,

    /// Upper limit to stack size in bytes.
    pub stack: Option<u64>,

    /// Maximum number of process.
    pub n_process: Option<u64>,

    /// Upper limit to output size in bytes.
    pub output: Option<u64>,
}

impl ResourceLimit {
    pub fn new(
        memory: Option<u32>,
        cpu_time: Option<u64>,
        real_time: Option<u32>,
        stack: Option<u64>,
        n_process: Option<u64>,
        output: Option<u64>,
    ) -> Self {
        Self {
            memory,
            cpu_time,
            real_time,
            stack,
            n_process,
            output,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    /// Peak memory usage in bytes.
    pub memory: u64,

    /// CPU time used in milliseconds.
    pub cpu_time: u64,

    /// Real time used in milliseconds.
    pub real_time: u128,
}

impl ResourceUsage {
    pub fn new(memory: u64, cpu_time: u64, real_time: u128) -> Self {
        Self {
            memory,
            cpu_time,
            real_time,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JudgeResult {
    /// The status of the judged run.
    pub status: JudgeStatus,

    /// Descriptive message (e.g., internal error details or stderr)
    pub message: Option<String>,

    /// The exit code of the process, if it exited normally.
    pub exit_code: Option<i32>,

    /// The signal number that terminated the process, if any.
    pub signal: Option<String>,

    /// Resource usage statistics.
    pub resource_usage: Option<ResourceUsage>,
    // cpu_time
    // real_time
    // pub memory: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum JudgeStatus {
    Exited,
    WrongAnswer,
    Accepted,
    CpuTimeLimitExceeded,
    RealTimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError,
    InternalError,
}

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("Failed to initialize cgroup: {0}")]
    CreateCgroup(cgroups_rs::fs::error::Error),

    #[error("Failed to add process to cgroup: {0}")]
    AddToCgroup(cgroups_rs::fs::error::Error),

    #[error("Failed to read memory stats from cgroup")]
    ReadCgroupMemoryStats,

    #[error("Failed to read cpu stats from cgroup")]
    ReadCgroupCpuStats,

    #[error("Failed to clone: {0}")]
    Clone(nix::Error),

    #[error("Failed to notify via channel: {0}")]
    Notify(nix::Error),

    #[error("Ended up in an unsupported wait status: {0}")]
    UnsupportedWait(String),

    #[error("Failed to wait runner process")]
    Wait(nix::Error),

    #[error("Failed to read output: {0}")]
    ReadOutput(io::Error),

    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
}
