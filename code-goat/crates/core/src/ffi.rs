use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_ulonglong};
use std::num::TryFromIntError;
use std::str::FromStr;

use nix::libc::c_ushort;

use crate::logger::LoggerError;
use crate::models::{JudgeResult, JudgeSpec, JudgeStatus, ResourceLimit, U63};
use crate::sandbox::seccomp::ScmpPolicy;
use crate::{judger, logger};

#[repr(C)]
pub struct CJudgeSpec {
    pub exe_path: *const c_char,
    pub input_path: *const c_char,
    pub answer_path: *const c_char,
    pub output_path: *const c_char,
    pub error_path: *const c_char,
    /// Elements in `args` and `envs` should be separated by " ".
    pub args: *const c_char,
    pub envs: *const c_char,
    pub scmp_policy: *const c_char,
    pub resource_limit: CResourceLimit,
}

#[repr(C)]
pub struct CResourceLimit {
    pub memory: c_ulonglong,
    pub cpu_time: c_uint,
    pub real_time: c_uint,
    pub stack: c_uint,
    pub n_process: c_ushort,
    pub output: c_uint,
}

impl TryFrom<CResourceLimit> for ResourceLimit {
    type Error = TryFromIntError;

    fn try_from(limit: CResourceLimit) -> Result<Self, Self::Error> {
        let memory_limit: U63 = limit.memory.try_into()?;
        Ok(Self {
            memory: wrap_number(memory_limit),
            cpu_time: wrap_number(limit.cpu_time),
            real_time: wrap_number(limit.real_time),
            stack: wrap_number(limit.stack),
            n_process: wrap_number(limit.n_process),
            output: wrap_number(limit.output),
        })
    }
}

fn wrap_number<T>(value: T) -> Option<T>
where
    T: PartialOrd + From<u8>,
{
    if value > T::from(0) {
        Some(value)
    } else {
        None
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn judger_judge(spec: CJudgeSpec) -> *mut c_char {
    let generate_error = |message: String| JudgeResult {
        status: JudgeStatus::InternalError,
        message: Some(message),
        exit_code: None,
        signal: None,
        resource_usage: None,
    };

    let result = match parse(spec) {
        Ok(spec) => judger::judge(spec),
        Err(key) => generate_error(format!("Failed to parse: {}", key)),
    };

    let json = serde_json::to_string_pretty(&result).unwrap_or("{}".to_string());
    CString::new(json)
        .unwrap_or(CString::new("{}").expect("Cannot fail to unwrap an safe string."))
        .into_raw()
}

fn parse<'a>(cspec: CJudgeSpec) -> Result<JudgeSpec, &'a str> {
    let exe_path = {
        let source = parse_str("exe_path", cspec.exe_path)?;
        CString::new(source).or(Err("exe_path"))
    }?;
    let input_path = parse_optional_str("input_path", cspec.input_path)?;
    let answer_path = parse_optional_str("answer_path", cspec.answer_path)?;
    let output_path = parse_optional_str("output_path", cspec.output_path)?;
    let error_path = parse_optional_str("error_path", cspec.error_path)?;

    let args = parse_cstr_array("args", cspec.args)?;
    let envs = parse_cstr_array("envs", cspec.envs)?;

    let scmp_policy = parse_str("scmp_policy", cspec.scmp_policy)
        .and_then(|s| ScmpPolicy::from_str(s).or(Err("scmp_policy")))?;
    let resource_limit = cspec.resource_limit.try_into().or(Err("resource_limit"))?;

    Ok(JudgeSpec::from_c_spec(
        exe_path,
        input_path,
        answer_path,
        output_path,
        error_path,
        args,
        envs,
        scmp_policy,
        resource_limit,
    ))
}

fn parse_str(key: &str, string: *const c_char) -> Result<&str, &str> {
    if string.is_null() {
        return Err(key);
    }
    unsafe { CStr::from_ptr(string) }.to_str().or(Err(key))
}

fn parse_optional_str(key: &str, string: *const c_char) -> Result<Option<String>, &str> {
    if string.is_null() {
        return Ok(None);
    }
    let source = unsafe { CStr::from_ptr(string) }
        .to_str()
        .or(Err(key))?
        .to_string();

    Ok(Some(source))
}

fn parse_cstr_array(key: &str, array: *const c_char) -> Result<Vec<CString>, &str> {
    if array.is_null() {
        return Ok(vec![]);
    }
    let args_str = unsafe { CStr::from_ptr(array) }.to_str().or(Err(key))?;

    let (ok_args, err_args): (Vec<_>, Vec<_>) = args_str
        .split_whitespace()
        .map(|s| CString::new(s))
        .partition(Result::is_ok);

    // Stop operation if it failed to parse any of the argument.
    if !err_args.is_empty() {
        return Err(key);
    }
    // `unwrap` will not panic since all Err's are filtered out.
    Ok(ok_args.into_iter().map(Result::unwrap_or_default).collect())
}

#[unsafe(no_mangle)]
pub extern "C" fn judger_free(return_value: *mut c_char) {
    if return_value.is_null() {
        return;
    }
    // Retrieve ownership of returned value. When this function ends,
    // [`Drop`] is called and the memory will be reaped.
    let _ = unsafe { CString::from_raw(return_value) };
}

////
/// Examples
/// ```c
/// int res = c_grade_output(...);
/// if (res < 0) { printf("Error Occured"); }
/// else if (res == 0) { printf("Wrong Answer"); }
/// else { printf("Accepted"); }
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn judger_grade_output(
    output_path: *const c_char,
    answer_path: *const c_char,
) -> c_int {
    let output_path = match parse_str("output_path", output_path) {
        Ok(path) => path,
        Err(_) => return -1,
    };
    let answer_path = match parse_str("answer_path", answer_path) {
        Ok(path) => path,
        Err(_) => return -1,
    };

    match judger::is_accepted(output_path, answer_path) {
        Ok(is_accepted) => is_accepted.into(),
        Err(_) => return -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn judger_configure_logger(log_path: *const c_char) -> c_int {
    let path = parse_optional_str("log_path", log_path);
    let result = match path {
        Ok(path) => logger::configure_logger(&path),
        Err(_) => return 1,
    };

    match result {
        Ok(_) => 0,
        Err(e) => match e {
            LoggerError::File(_) => 2,
            LoggerError::Dup(_) => 3,
            LoggerError::Register => 4,
            LoggerError::Subscribe(_) => 5,
            LoggerError::Reload(_) => 6,
            LoggerError::Disable => 7,
        },
    }
}
