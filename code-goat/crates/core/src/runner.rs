use std::{
    fs::File,
    io::{ErrorKind, PipeReader, PipeWriter},
};

use libseccomp::error::SeccompErrno;
use log::error;
use nix::{
    sched::{self, CloneFlags},
    sys::signal::Signal,
    unistd::{self, Pid},
};

use crate::{
    models::{InternalError, JudgeSpec},
    sandbox::{self, seccomp},
};

/// Clone a new process with specified namespaces.
/// Returns the PID of the cloned process.
pub fn clone(
    spec: &JudgeSpec,
    setup_rx: PipeReader,
    abort_tx: PipeWriter,
) -> Result<Pid, InternalError> {
    let runner = {
        Box::new(|| match run(&spec, &setup_rx, &abort_tx) {
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

/// The function executed in the cloned child process.
/// Run the untrusted code in an isolated environment
/// and return the exit status.
fn run(
    spec: &JudgeSpec,
    setup_rx: &PipeReader,
    abort_tx: &PipeWriter,
) -> Result<isize, nix::Error> {
    // If any sandboxing mechanism fails, abort runner process with message.
    // Judger will collect the message and handle this request as a
    // `JudgeStatus::InternalError`.
    let abort = |e: nix::Error, message: &str| {
        let _ = unistd::write(&abort_tx, message.as_bytes());

        error!("{}; aborting runner...", message);
        Err(e)
    };

    if let Err(e) = sandbox::mount_sandbox() {
        return abort(e, "Failed to mount user namespace");
    }

    if let Err(e) = sandbox::set_limit_to_sandbox(&spec.resource_limit) {
        return abort(e, "Failed to set resource limit");
    }

    if let Err(e) = redirect(spec) {
        return abort(e.source, &e.context);
    }

    // Apply seccomp right before `execve` so that runner can provoke
    // prohibited syscalls while creating the sandbox environment.
    if let Err(e) = seccomp::apply_filter(&spec.scmp_policy, &spec.exe_path) {
        let errno = e.errno().unwrap_or(SeccompErrno::EFAULT);
        return abort(
            nix::Error::from_raw(errno as i32),
            "Failed to apply secure computing mode",
        );
    }

    // Wait until judger set up cgroups and timeout handler.
    if let Err(e) = unistd::read(setup_rx, &mut [0u8; 1]) {
        return abort(e, "Failed to get notified");
    }

    // Run the untrusted code in a sandboxed environment.
    let Err(e) = unistd::execve(&spec.exe_path, &spec.args, &spec.envs);
    return abort(e, &format!("Failed to execute spec {:#?}.", &spec));
}

struct RedirectError {
    source: nix::Error,
    context: String,
}

impl RedirectError {
    fn from_io_error(io_error: std::io::Error, action: &str, file_path: &str) -> Self {
        let source = match io_error.kind() {
            ErrorKind::NotFound => nix::Error::ENOENT,
            ErrorKind::PermissionDenied => nix::Error::EACCES,
            ErrorKind::IsADirectory => nix::Error::EISDIR,
            _ => nix::Error::UnknownErrno,
        };

        Self {
            source,
            context: format!(
                "Failed to {} file {}: {}",
                action,
                file_path,
                io_error.kind()
            ),
        }
    }

    fn from_errno(errno: nix::Error, fd: &str) -> Self {
        Self {
            source: errno,
            context: format!("Failed to redirect {}", fd),
        }
    }
}

/// Redirect stdin, stdout, stderr according to the JudgeSpec.
fn redirect<'a>(spec: &'a JudgeSpec) -> Result<(), RedirectError> {
    if let Some(path) = &spec.input_path {
        let f_in =
            File::open(path.clone()).map_err(|e| RedirectError::from_io_error(e, "open", &path))?;
        unistd::dup2_stdin(f_in).map_err(|e| RedirectError::from_errno(e, "stdin"))?;
    }

    if let Some(path) = &spec.output_path {
        let f_out = File::create(path.clone())
            .map_err(|e| RedirectError::from_io_error(e, "create", &path))?;
        unistd::dup2_stdout(f_out).map_err(|e| RedirectError::from_errno(e, "stdout"))?;
    }

    if let Some(path) = &spec.error_path {
        let f_err = File::create(path.clone())
            .map_err(|e| RedirectError::from_io_error(e, "create", &path))?;
        unistd::dup2_stderr(f_err).map_err(|e| RedirectError::from_errno(e, "stderr"))?;
    }

    Ok(())
}
