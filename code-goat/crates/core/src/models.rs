use std::{ffi::CString, iter, num::TryFromIntError};

use log::warn;
use serde::Serialize;

use crate::sandbox::seccomp::ScmpPolicy;

#[derive(Debug)]
pub struct JudgeSpec {
    /// Absolute path to the executable file.
    pub exe_path: CString,

    /// Absolute path to the input file (for stdin redirection).
    pub input_path: Option<String>,

    /// Absolute path to the error file (for stderr redirection).
    pub answer_path: Option<String>,

    /// Absolute path to the output file (for stdout redirection).
    pub output_path: Option<String>,

    /// Absolute path to the error file (for stderr redirection).
    pub error_path: Option<String>,

    /// List of arguments to pass to the program.
    /// Example: ["/usr/bin/echo", "hello"]
    pub args: Vec<CString>,

    /// Environment variables to set for the program.
    /// Example: ["PATH=/usr/bin"]
    pub envs: Vec<CString>,

    /// Seccomp rule set name.
    pub scmp_policy: ScmpPolicy,

    /// The judging policy (resource usage limits) to apply.
    pub resource_limit: ResourceLimit,
}

impl<'a> JudgeSpec {
    pub fn try_new(
        exe_path: &str,
        input_path: Option<&str>,
        answer_path: Option<&str>,
        output_path: Option<&str>,
        error_path: Option<&str>,
        args: Vec<&str>,
        envs: Vec<&str>,
        scmp_policy: ScmpPolicy,
        resource_limit: ResourceLimit,
    ) -> Result<Self, std::ffi::NulError> {
        let exe_path_cstr = CString::new(exe_path)?;

        let is_exe_missing = match args.get(0) {
            Some(&first) => first != exe_path,
            None => true,
        };
        let args = args.into_iter().map(CString::new);
        let args_cstr = if is_exe_missing {
            iter::once(Ok(exe_path_cstr.clone()))
                .chain(args)
                .collect::<Result<Vec<CString>, std::ffi::NulError>>()
        } else {
            args.collect()
        }?;

        let envs_cstr = envs
            .into_iter()
            .map(|env| CString::new(env))
            .collect::<Result<Vec<CString>, std::ffi::NulError>>()?;

        Ok(Self {
            exe_path: exe_path_cstr,
            input_path: input_path.map(String::from),
            answer_path: answer_path.map(String::from),
            output_path: output_path.map(String::from),
            error_path: error_path.map(String::from),
            args: args_cstr,
            envs: envs_cstr,
            scmp_policy,
            resource_limit,
        })
    }

    pub fn from_c_spec(
        exe_path: CString,
        input_path: Option<String>,
        answer_path: Option<String>,
        output_path: Option<String>,
        error_path: Option<String>,
        args: Vec<CString>,
        envs: Vec<CString>,
        scmp_policy: ScmpPolicy,
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
            answer_path,
            output_path,
            error_path,
            args: full_args,
            envs,
            scmp_policy,
            resource_limit,
        }
    }
}

#[derive(Debug)]
pub struct ResourceLimit {
    /// Peak memory usage in bytes.
    pub memory: Option<U63>,

    /// CPU time used in milliseconds.
    pub cpu_time: Option<u32>,

    /// Real time used in milliseconds.
    pub real_time: Option<u32>,

    /// Upper limit to stack size in bytes.
    pub stack: Option<u32>,

    /// Maximum number of process.
    pub n_process: Option<u16>,

    /// Upper limit to output size in bytes.
    pub output: Option<u32>,
}

impl ResourceLimit {
    pub fn new(
        memory: Option<U63>,
        cpu_time: Option<u32>,
        real_time: Option<u32>,
        stack: Option<u32>,
        n_process: Option<u16>,
        output: Option<u32>,
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

/// A 63-bit unsigned integer type.
///
/// Use this to represent [`i64`] values that are guaranteed to be
/// non-negative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct U63(u64);

impl U63 {
    pub const MAX: Self = Self(1 << 63 - 1);

    fn saturating_new(value: u64) -> Self {
        if Self::does_fit(value) {
            Self(value)
        } else {
            warn!("Value {} exceeds 63 bits, saturating to 63 bits", value);
            Self::MAX
        }
    }

    pub fn does_fit(value: u64) -> bool {
        value <= Self::MAX.0
    }

    /// Saturating addition. Computes `self + rhs`, saturating at the boundary
    /// of 63 bits if overflow occurs.
    pub fn saturating_add<T: Into<u64>>(&self, other: T) -> Self {
        Self::saturating_new(self.0.saturating_add(other.into()))
    }
}

impl TryFrom<u64> for U63 {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if Self::does_fit(value) {
            Ok(Self(value))
        } else {
            // We need to generate a [`TryFromIntError`] using std library functions
            // because TryFromIntError does not have a public constructor.
            // [`u32::try_from will`] always fail here as `!does_fit` ensures that
            // the value is larger than u32::MAX.
            Err(u32::try_from(value).unwrap_err())
        }
    }
}

impl From<U63> for u64 {
    fn from(value: U63) -> Self {
        value.0
    }
}

impl From<U63> for i64 {
    fn from(value: U63) -> Self {
        value.0 as i64
    }
}

impl From<u8> for U63 {
    fn from(value: u8) -> Self {
        Self::saturating_new(value.into())
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
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum JudgeStatus {
    Exited,
    Accepted,
    WrongAnswer,
    CpuTimeLimitExceeded,
    RealTimeLimitExceeded,
    MemoryLimitExceeded,
    // TODO: OutputLimitExceeded,
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
    Notify(std::io::Error),

    #[error("Ended up in an unsupported wait status: {0}")]
    UnsupportedWait(String),

    #[error("Failed to wait runner process")]
    Wait(nix::Error),

    #[error("Failed to read output: {0}")]
    ReadOutput(std::io::Error),

    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    /// Peak memory usage in bytes.
    pub memory: u64,

    /// CPU time used in milliseconds.
    pub cpu_time: u32,

    /// Real time used in milliseconds.
    pub real_time: u32,
}

impl ResourceUsage {
    pub fn new(memory: u64, cpu_time: u32, real_time: u32) -> Self {
        Self {
            memory,
            cpu_time,
            real_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod judge_spec {
        use super::*;

        fn try_new_spec(args_include_exe: bool) {
            let exe_path = "/usr/bin/echo";
            let expected = vec![exe_path, "hello", "world"];
            let args = if args_include_exe {
                expected.clone()
            } else {
                expected[1..].to_vec()
            };

            let spec = JudgeSpec::try_new(
                exe_path,
                None,
                None,
                None,
                None,
                args,
                vec![],
                ScmpPolicy::Unsafe,
                ResourceLimit::new(None, None, None, None, None, None),
            )
            .expect("Cstring conversion failed.");

            let expected = expected
                .iter()
                .map(|&arg| CString::new(arg))
                .collect::<Result<Vec<CString>, std::ffi::NulError>>()
                .expect("Cstring conversion failed.");
            assert_eq!(spec.args, expected);
        }

        #[test]
        fn try_new_with_exe_in_args() {
            try_new_spec(true);
        }

        #[test]
        fn try_new_without_exe_in_args() {
            try_new_spec(false);
        }

        fn from_c_spec(args_include_exe: bool) {
            let exe_path = CString::new("/usr/bin/echo").expect("Cstring conversion failed.");
            let expected = vec![
                exe_path.clone(),
                CString::new("hello").expect("Cstring conversion failed."),
                CString::new("world").expect("Cstring conversion failed."),
            ];
            let args = if args_include_exe {
                expected.clone()
            } else {
                expected[1..].to_vec()
            };

            let spec = JudgeSpec::from_c_spec(
                exe_path,
                None,
                None,
                None,
                None,
                args,
                vec![],
                ScmpPolicy::Unsafe,
                ResourceLimit::new(None, None, None, None, None, None),
            );
            assert_eq!(spec.args, expected);
        }

        #[test]
        fn from_c_spec_with_exe_in_args() {
            from_c_spec(true);
        }

        #[test]
        fn from_c_spec_without_exe_in_args() {
            from_c_spec(false);
        }
    }

    mod u63 {
        use super::*;

        #[test]
        fn saturating_add_within_bounds() {
            let a = 1;
            let b = 2;
            let res = U63::saturating_new(a).saturating_add(b);
            assert_eq!(u64::from(res), a + b);
        }

        #[test]
        fn saturating_add_exceeds_bounds() {
            let a = U63::MAX.into();
            let b = u64::MAX;
            let res = U63::saturating_new(a).saturating_add(b);
            assert_eq!(u64::from(res), U63::MAX.0);
        }
    }
}
