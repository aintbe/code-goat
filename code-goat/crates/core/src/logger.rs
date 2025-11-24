use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io,
    os::fd::AsFd,
    sync::OnceLock,
};

use log::info;
use nix::unistd;

use parking_lot::Mutex;
use tracing_subscriber::{
    Registry,
    fmt::{
        self,
        format::{DefaultFields, Format},
        writer::BoxMakeWriter,
    },
    prelude::*,
    reload::{self, Handle},
    util::TryInitError,
};

#[derive(Clone)]
enum LoggerDestination {
    None,
    Stdout,
    /// [`String`] The absolute path to the log file, if any.
    File(String),
}

impl Display for LoggerDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerDestination::None => write!(f, ""),
            LoggerDestination::Stdout => write!(f, "stdout"),
            LoggerDestination::File(path) => write!(f, "'{}'", path),
        }
    }
}

type Layer = fmt::Layer<Registry, DefaultFields, Format, BoxMakeWriter>;

/// This is used to check if the logger needs to be re-configured.
static LOGGER_DESTINATION: Mutex<LoggerDestination> = Mutex::new(LoggerDestination::None);

/// This is used to reload the tracing layer when the log destination changes.
static LOGGER_RELOAD_HANDLE: OnceLock<Handle<Layer, Registry>> = OnceLock::new();

struct LoggerSettings {
    destination: LoggerDestination,
    file: Option<File>,
    ansi: bool,
}

impl LoggerSettings {
    fn new(destination: LoggerDestination, file: File, ansi: bool) -> Self {
        Self {
            destination,
            file: Some(file),
            ansi,
        }
    }

    fn default() -> Self {
        Self {
            destination: LoggerDestination::None,
            file: None,
            ansi: false,
        }
    }

    fn apply(self) -> Result<LoggerDestination, LoggerError> {
        // Create a new layer based on the requested log destination.
        let make_writer = match self.file {
            Some(file) => BoxMakeWriter::new(file),
            None => BoxMakeWriter::new(|| io::sink()),
        };
        let layer: Layer = fmt::layer().with_writer(make_writer).with_ansi(self.ansi);

        // Get the global layer handle to install the logger layer.
        // If the `handle` is already registered, reload the layer with new settings.
        // Otherwise, initialize the tracing subscriber and register the handle.
        if let Some(handle) = LOGGER_RELOAD_HANDLE.get() {
            handle.reload(layer).map_err(LoggerError::Reload)?;
        } else {
            let (reloadable_layer, reload_handle) = reload::Layer::new(layer);
            tracing_subscriber::registry()
                .with(reloadable_layer)
                .try_init()
                .map_err(LoggerError::Subscribe)?;

            LOGGER_RELOAD_HANDLE
                .set(reload_handle)
                // This should never fail as we check existence above.
                .or(Err(LoggerError::Register))?;
        };

        Ok(self.destination)
    }
}

/// Set the global logger to write logs to the specified path.
/// If `log_path` is [`None`], logs will be written to stdout.
/// If `log_path` is [`Some`], logs will be written to the specified file.
///
/// This function is thread-safe and idempotent, so it can be called
/// multiple times to change the log destination at runtime.
pub fn configure_logger(log_path: &Option<String>) -> Result<(), LoggerError> {
    // Check if the requested log destination is different from the
    // current one. If not, do nothing and return early.
    let mut guard = LOGGER_DESTINATION.lock();
    match &*guard {
        LoggerDestination::None => {}
        LoggerDestination::Stdout => {
            if log_path.is_none() {
                return Ok(());
            }
        }
        LoggerDestination::File(path) => {
            if let Some(new_path) = log_path {
                if new_path == path {
                    return Ok(());
                }
            }
        }
    };

    // Create a new writer based on the requested log path.
    // If `log_path` is `None`, use duped stdout so that the logs from
    // cloned process can also be captured.
    let settings = match log_path {
        Some(path) => {
            // Open or create the log file for appending.
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)
                .map_err(LoggerError::File)?;

            Ok(LoggerSettings::new(
                LoggerDestination::File(path.clone()),
                file,
                false,
            ))
        }
        None => {
            let stdout = io::stdout();
            let file = unistd::dup(stdout.as_fd())
                .map(File::from)
                .map_err(LoggerError::Dup)?;
            Ok(LoggerSettings::new(LoggerDestination::Stdout, file, true))
        }
    };

    // Apply new settings and update the global destination.
    let result = settings.and_then(|s| s.apply());
    match result {
        Ok(destination) => {
            info!("Logging ready. Output destination set to {}.", destination);
            *guard = destination;
            Ok(())
        }
        Err(_) => {
            let _ = LoggerSettings::default().apply();
            // Always update the destination to `LoggerDestination::None`
            // (even if it failed to disable logger), so that future calls
            // can retry initializing the logger.
            *guard = LoggerDestination::None;
            Err(LoggerError::Disable)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error("Failed to create/open file: {0}")]
    File(#[from] std::io::Error),

    #[error("Failed to duplicate file descriptor: {0}")]
    Dup(#[from] nix::Error),

    #[error("Failed to register tracker")]
    Register,

    #[error("Failed to initialize logger: {0}")]
    Subscribe(#[from] TryInitError),

    #[error("Failed to change logger settings: {0}")]
    Reload(#[from] reload::Error),

    #[error("Failed to configure logger, logger disabled")]
    Disable,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod configure_logger {
        use std::fs;

        use super::*;

        use log::error;

        /// Contains test data to be cleaned up after tests.
        struct TestData {
            log_path: String,
        }

        impl TestData {
            fn new(log_path: &str) -> Self {
                Self {
                    log_path: log_path.to_string(),
                }
            }
        }

        impl Drop for TestData {
            fn drop(&mut self) {
                if fs::metadata(&self.log_path).is_ok() {
                    let message = format!("Removing test log file: {}", &self.log_path);
                    fs::remove_file(&self.log_path).expect(&message);
                }
            }
        }

        #[test]
        fn write_logs_to_file() {
            let data = TestData::new("test.log");
            let initial_config = configure_logger(&Some(data.log_path.clone()));
            assert!(initial_config.is_ok());

            let log = "LOG_CONTENT".to_string();
            error!("{}", log);

            let content = fs::read_to_string(&data.log_path).expect("Failed to read test.log");
            assert!(content.contains(&log));
        }

        #[test]
        fn work_with_multiple_calls() {
            let data1 = TestData::new("test1.log");
            let log1 = "LOG_CONTENT_1".to_string();
            let data2 = TestData::new("test2.log");
            let log2 = "LOG_CONTENT_2".to_string();

            configure_logger(&Some(data1.log_path.clone()))
                .expect("Failed to configure first logger");
            error!("{}", log1);

            let second_config = configure_logger(&Some(data2.log_path.clone()));
            error!("{}", log2);
            assert!(second_config.is_ok());

            let content1 = fs::read_to_string(&data1.log_path).expect("Failed to read test1.log");
            let content2 = fs::read_to_string(&data2.log_path).expect("Failed to read test2.log");

            assert!(content1.contains(&log1));
            assert!(!content1.contains(&log2));
            assert!(content2.contains(&log2));
            assert!(!content2.contains(&log1));
        }
    }
}
