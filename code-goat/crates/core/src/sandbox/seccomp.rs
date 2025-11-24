use std::ffi::CString;

use libseccomp::{
    ScmpAction, ScmpArgCompare, ScmpFilterContext, ScmpSyscall, error::SeccompError, scmp_cmp,
};
use log::warn;
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum ScmpPolicy {
    #[strum(ascii_case_insensitive)]
    Unsafe,
    #[strum(ascii_case_insensitive)]
    Strict,
    #[strum(ascii_case_insensitive)]
    Python,
    // Java,
}

/// Apply seccomp whitelist.
/// TODO: more comment
pub(crate) fn apply_filter(
    scmp_policy: &ScmpPolicy,
    exe_path: &CString,
) -> Result<(), SeccompError> {
    let whitelist = get_whitelist(scmp_policy, exe_path)?;
    if whitelist.len() == 0 {
        warn!(
            "Running under an UNSAFE seccomp policy!
    The current policy, {:?}, means ALL system calls are permitted.
    This should ONLY be used for debugging or testing trusted code.",
            scmp_policy
        );
        return Ok(());
    };

    // Kill process if the runner provokes any syscall not whitelisted.
    let mut filter = ScmpFilterContext::new(ScmpAction::KillProcess)?;

    for rule in whitelist {
        match rule.comparator {
            Some(cmp) => filter.add_rule_conditional(ScmpAction::Allow, rule.syscall, &[cmp]),
            None => filter.add_rule(ScmpAction::Allow, rule.syscall),
        }?;
    }
    filter.load()
}

struct ScmpRule {
    syscall: ScmpSyscall,
    comparator: Option<ScmpArgCompare>,
}

impl ScmpRule {
    fn new(name: &str) -> Result<Self, SeccompError> {
        let syscall = ScmpSyscall::from_name(name)?;
        Ok(ScmpRule {
            syscall,
            comparator: None,
        })
    }
}

fn get_whitelist(
    scmp_policy: &ScmpPolicy,
    exe_path: &CString,
) -> Result<Vec<ScmpRule>, SeccompError> {
    let exe_path_addr = exe_path.as_ptr() as u64;
    let common_rules = COMMON_SYSCALLS
        .into_rules()
        // Allow forbidden syscalls only for the initial execution.
        .chain(EXEC_SYSCALLS.into_cond_rules(Some(scmp_cmp!($arg0 == exe_path_addr))))
        // Allow file access only for read operation.
        .chain(FILE_SYSCALLS.into_cond_rules(Some(scmp_cmp!($arg1 & WRITE_FLAGS == 0))))
        // Disable changing resource limits except getting them.
        // todo: arg0 == 0 자기 것만 확인하게 하기 (괜찮나?)
        .chain(["prlimit64"].into_cond_rules(Some(scmp_cmp!($arg2 == 0))));

    match scmp_policy {
        ScmpPolicy::Unsafe => Ok(vec![]),
        ScmpPolicy::Strict => common_rules.collect(),
        ScmpPolicy::Python => common_rules.chain(PYTHON_SYSCALLS.into_rules()).collect(),
    }
}

trait SyscallList {
    fn into_rules(&self) -> impl Iterator<Item = Result<ScmpRule, SeccompError>>;
    fn into_cond_rules(
        &self,
        cmp: Option<ScmpArgCompare>,
    ) -> impl Iterator<Item = Result<ScmpRule, SeccompError>>;
}

impl SyscallList for [&str] {
    fn into_rules(&self) -> impl Iterator<Item = Result<ScmpRule, SeccompError>> {
        self.into_iter().map(|&name| ScmpRule::new(name))
    }

    fn into_cond_rules(
        &self,
        cmp: Option<ScmpArgCompare>,
    ) -> impl Iterator<Item = Result<ScmpRule, SeccompError>> {
        self.into_iter().map(move |&name| {
            let syscall = ScmpSyscall::from_name(name)?;
            Ok(ScmpRule {
                syscall,
                comparator: cmp,
            })
        })
    }
}

const COMMON_SYSCALLS: [&str; 23] = [
    "brk",             // Change data segment size
    "close",           // Close a fd
    "exit",            // Exit the current thread
    "exit_group",      // Exit all threads in the current thread group
    "faccessat",       // Check user's permissions for a file
    "fstat",           // Get status of an open file
    "futex",           // Use synchronization (mutex, semaphore, etc.)
    "getrandom",       // Get random bytes
    "lseek",           // Reposition read/write file offset
    "mmap",            // Map memory region
    "mprotect",        // Change protections of an assigned memory region
    "munmap",          // Unmap memory region
    "newfstatat",      // Get status of a file relative to a directory
    "pread64",         // Read without changing file offset
    "read",            // Read from a fd
    "readlink",        // Read value of a symbolic link
    "readlinkat",      // Read value of a symbolic link relative to a directory
    "readv",           // Read from a fd using vector I/O
    "rseq",            // Faster synchonization without locking
    "set_robust_list", // Set list of robust futexes to prevent deadlocks
    "set_tid_address", // Initialize thread ID address upon exit
    "write",           // Write to a fd
    "writev",          // Write to a fd using vector I/O
];

// TODO: consider adding these syscalls as well
// const GENERAL_SYSCALLS: [&str; 3] = ["access", "arch_prctl", "clock_gettime"];

const PYTHON_SYSCALLS: [&str; 12] = [
    "fcntl", // Manipulate file descriptor
    // "getcwd",       // Get current working directory
    "getdents64",   // Get directory entries
    "getegid",      // Get effective user group ID
    "geteuid",      // Get effective user ID
    "getgid",       // Get user group ID
    "gettid",       // Get thread ID
    "getuid",       // Get user ID
    "ioctl",        // Control device I/O
    "mremap",       // Resize memory mapping
    "rt_sigaction", // Register a signal handler
    //
    "socket",  // Create a socket
    "connect", // Connect a socket
];

// TODO: add more syscalls used in Python runtime
// socket(AF_UNIX, SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK, 0) = 3
// [pid 65108] connect(3, {sa_family=AF_UNIX, sun_path="/var/run/nscd/socket"}, 110) = -1 EACCES (Permission denied)
// [pid 65108] close(3)                    = 0
// [pid 65108] socket(AF_UNIX, SOCK_STREAM|SOCK_CLOEXEC|SOCK_NONBLOCK, 0) = 3

const EXEC_SYSCALLS: [&str; 1] = ["execve"];

const FILE_SYSCALLS: [&str; 2] = ["open", "openat"];
const WRITE_FLAGS: u64 = (nix::libc::O_WRONLY | nix::libc::O_RDWR) as u64;
