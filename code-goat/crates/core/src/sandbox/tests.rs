use super::*;

mod mount_sandbox {
    use std::{
        fs::{self, File},
        io::{self, Read, Write},
        panic::{self, UnwindSafe},
        time::{SystemTime, UNIX_EPOCH},
    };

    use nix::{
        sched::{self, CloneFlags},
        sys::wait::{self, WaitStatus},
        unistd::ForkResult,
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    enum Action {
        Read,
        Write,
    }

    fn test_mount<F, R>(test_function: F)
    where
        F: FnOnce() -> R + UnwindSafe,
    {
        let (mut consumer, mut producer) = io::pipe().expect("Failed to create pipe");

        // Fork the process to create a new mount namespace.
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                drop(producer);

                match wait::waitpid(child, None) {
                    Ok(WaitStatus::Exited(_1, _2)) => {
                        let mut msg = String::new();
                        let _ = consumer.read_to_string(&mut msg);
                        if !msg.is_empty() {
                            panic!("{}", msg);
                        }
                    }
                    Ok(status) => {
                        panic!("Unexpected wait status: {:?}", status);
                    }
                    Err(e) => {
                        panic!("Failed to wait for child process: {}", e);
                    }
                }
            }
            Ok(ForkResult::Child) => {
                drop(consumer);

                let result = panic::catch_unwind(|| {
                    // Create new mount namespace and mount the sandbox.
                    sched::unshare(
                        CloneFlags::CLONE_NEWUSER
                            | CloneFlags::CLONE_NEWNS
                            | CloneFlags::CLONE_NEWPID,
                    )
                    .expect("Failed to unshare mount namespace");
                    mount_sandbox();
                    test_function(); // Run the test function after mounting.
                });

                if let Err(e) = result {
                    if let Some(msg) = e.downcast_ref::<String>() {
                        producer.write(msg.as_bytes()).expect("Failed to write");
                    } else if let Some(msg) = e.downcast_ref::<&'static str>() {
                        producer.write(msg.as_bytes()).expect("Failed to write");
                    } else {
                        producer
                            .write(b"Unknown type of error occurred.")
                            .expect("Failed to write");
                    }
                }
            }
            Err(e) => {
                panic!("Failed to fork process: {}", e);
            }
        }
    }

    fn has_permission_to(action: Action, dir_path: &Path) -> bool {
        match action {
            Action::Read => fs::read_dir(dir_path).and(Ok(true)),
            Action::Write => {
                let time_in_ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .subsec_nanos();
                let test_file = dir_path.join(format!("test.{:x}.tmp", time_in_ns));

                File::create(&test_file)
                    .and(fs::remove_file(&test_file))
                    .and(Ok(true))
            }
        }
        .unwrap_or_else(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                false
            } else if action == Action::Write && e.kind() == std::io::ErrorKind::ReadOnlyFilesystem
            {
                false
            } else {
                true
            }
        })
    }

    #[test]
    fn root_is_read_only() {
        test_mount(|| {
            let root_dir = fs::read_dir("/").expect("Failed to read root directory");
            let workspace = env::var("SANDBOX_WORKSPACE").unwrap_or_default();
            for entry in root_dir {
                let path = entry.expect("Failed to read directory entry").path();

                if path.is_dir()
                    && path.to_str() != Some(&workspace)
                    // NOTE: /proc is a special filesystem that allows writing 
                    // to some files even in read-only mount.
                    && path.to_str() != Some("/proc")
                {
                    assert!(!has_permission_to(Action::Write, &path));
                }
            }
        });
    }

    #[test]
    fn sensitive_dirs_are_empty() {
        test_mount(|| {
            for entry in SENSITIVE_DIRS {
                assert!(!has_permission_to(Action::Read, Path::new(entry)));
            }
        });
    }

    #[test]
    fn workspace_is_writable() {
        test_mount(|| {
            if let Ok(workspace) = env::var("SANDBOX_WORKSPACE") {
                let path = Path::new(&workspace);
                assert!(has_permission_to(Action::Read, path));
                assert!(has_permission_to(Action::Write, path));
            }
        });
    }
}
