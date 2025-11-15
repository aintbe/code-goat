use std::{
    fs::{self},
    io::{self, Read},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use log::{error, info};
use nix::{
    sys::{
        signal::{self, Signal::SIGKILL},
        wait::{self, WaitStatus},
    },
    unistd::{self, Pid},
};

use crate::{
    models::{InternalError, JudgeResult, JudgeSpec, JudgeStatus, ResourceUsage},
    runner,
    sandbox::{CgroupSandbox, TimeSandbox},
};

/// The entry point for judging a submission.
pub fn judge(spec: JudgeSpec) -> JudgeResult {
    match try_judge(&spec) {
        Ok(result) => result,
        Err(e) => JudgeResult {
            status: JudgeStatus::InternalError,
            message: Some(e.to_string()),
            exit_code: None,
            signal: None,
            resource_usage: None,
        },
    }
}

/// The main judging logic.
/// It sets up the sandbox to run the untrusted code, monitors its
/// execution, and collects resource usage.
fn try_judge(spec: &RunSpec) -> Result<JudgeResult, InternalError> {
    let cg_sandbox = CgroupSandbox::new(&spec.resource_limit)?;
    let (setup_rx, setup_tx) = io::pipe()?;
    let (mut abort_rx, abort_tx) = io::pipe()?;

    // Clone a runner process in a new user namespace.
    let runner_pid = runner::clone(spec, setup_rx, abort_tx)?;

    cg_sandbox.add_process(runner_pid)?;

    // Prevent the runner process from running longer than specified limit.
    // Its scope (and thus its drop execution) is intentionally extended
    // to judger's lifetime so that judger can reap it with `drop`.
    let _timeout_guard = spec
        .resource_limit
        .real_time
        .map(|limit| TimeoutGuard::new(runner_pid, limit));

    match unistd::write(&setup_tx, b"1") {
        Ok(_) => info!("Judger finished setting sandbox; notifying runner to resume..."),
        Err(nix::Error::EPIPE) => {
            error!("Read end is closed due to runnner's abortion; waiting to reap it...");
        }
        Err(e) => return Err(InternalError::Notify(e)),
    };

    // Capture the start time of runner after wait time.
    let runner_clock = Instant::now();

    match wait::waitpid(runner_pid, None) {
        Ok(WaitStatus::Exited(_, exit_code)) => {
            let runner_duration = runner_clock.elapsed();

            // Check if runner aborted while setting up the sandbox.
            // If so, respond with an `JudgeStatus::InternalError`.
            let mut aborted_message = String::new();
            let _ = abort_rx.read_to_string(&mut aborted_message);
            if aborted_message.len() > 0 {
                return Ok(JudgeResult {
                    status: JudgeStatus::InternalError,
                    message: Some(aborted_message),
                    exit_code: None,
                    signal: None,
                    resource_usage: None,
                });
            };

            // Parse judge status and resource usage.
            let resource_usage = get_resource_usage(cg_sandbox, runner_duration)?;
            let status = get_judge_status(&spec, &resource_usage, JudgeStatus::Exited)?;

            Ok(JudgeResult {
                status,
                message: None,
                exit_code: Some(exit_code),
                signal: None,
                resource_usage: Some(resource_usage),
            })
        }
        Ok(WaitStatus::Signaled(_, signal, _)) | Ok(WaitStatus::Stopped(_, signal)) => {
            let runner_duration = runner_clock.elapsed();
            let resource_usage = get_resource_usage(cg_sandbox, runner_duration)?;
            let status = get_judge_status(&spec, &resource_usage, JudgeStatus::RuntimeError)?;

            info!("what the ... {:?} {:?}", resource_usage, status);

            Ok(JudgeResult {
                status,
                // TODO: read from error
                message: None,
                exit_code: None,
                signal: Some(format!("{:?}", signal)),
                resource_usage: Some(resource_usage),
            })
        }
        Ok(ws) => Err(InternalError::UnsupportedWait(format!("{:?}", ws))),
        Err(e) => Err(InternalError::Wait(e)),
    }
}

/// Calculate the amount of resources used by runner process.
fn get_resource_usage(
    cg_sandbox: CgroupSandbox,
    duration: Duration,
) -> Result<ResourceUsage, InternalError> {
    let memory = cg_sandbox.read_memory_usage()?;
    let cpu_time = cg_sandbox.read_cpu_time_usage()?;

    Ok(ResourceUsage::new(memory, cpu_time, duration.as_millis()))
}

/// Determine the judge status based on resource usage.
fn get_judge_status(
    spec: &RunSpec,
    resource_usage: &ResourceUsage,
    default_status: JudgeStatus,
) -> Result<JudgeStatus, InternalError> {
    if let Some(limit) = spec.resource_limit.cpu_time
        && resource_usage.cpu_time > 0
        && (resource_usage.cpu_time as u64) > limit
    {
        Ok(JudgeStatus::CpuTimeLimitExceeded)
    } else if let Some(limit) = spec.resource_limit.real_time
        && resource_usage.real_time > limit.into()
    {
        Ok(JudgeStatus::RealTimeLimitExceeded)
    } else if let Some(limit) = spec.resource_limit.memory
        && resource_usage.memory > limit.into()
    {
        Ok(JudgeStatus::MemoryLimitExceeded)
    } else if default_status == JudgeStatus::Exited
        && spec.output_path.is_some()
        && spec.answer_path.is_some()
    {
        let output = &spec.output_path.clone().unwrap();
        let answer = &spec.answer_path.clone().unwrap();

        is_accepted(output, answer).map(|accepted| {
            if accepted {
                JudgeStatus::Accepted
            } else {
                JudgeStatus::WrongAnswer
            }
        })
    } else {
        Ok(default_status)
    }
}

/// Check runner's output and expected output to retrieve status.
pub fn is_accepted(output_path: &str, answer_path: &str) -> Result<bool, InternalError> {
    let output_content = get_clean_content(output_path)?;
    let answer_content = get_clean_content(answer_path)?;

    Ok(output_content == answer_content)
}

fn get_clean_content(path: &str) -> Result<String, InternalError> {
    let content = fs::read_to_string(path).map_err(InternalError::ReadOutput)?;
    let clean_content = content
        .trim_end()
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n");

    Ok(clean_content)
}
