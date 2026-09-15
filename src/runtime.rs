use crate::error::RuntimeError;
use crate::log::Logger;
use std::path::Path;

#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownStatus {
    Synchronized,
}

#[derive(Debug)]
pub struct Runtime {
    logger: Logger,
    shutdown: bool,
}

impl Runtime {
    pub fn start(log_path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        Ok(Self {
            logger: Logger::open(log_path)?,
            shutdown: false,
        })
    }

    pub fn log_path(&self) -> &Path {
        self.logger.path()
    }

    pub fn execute(&mut self, input: &str) -> Result<(), RuntimeError> {
        if self.shutdown {
            return Err(RuntimeError::already_shutdown("execute"));
        }
        if input.is_empty() {
            let error = RuntimeError::invalid_input("execute", "input must not be empty");
            return match self.logger.record_error(&error) {
                Ok(_record) => Err(error),
                Err(logging_error) => Err(logging_error),
            };
        }
        Ok(())
    }

    pub fn fail(&mut self, error: RuntimeError) -> Result<(), RuntimeError> {
        if self.shutdown {
            return Err(RuntimeError::already_shutdown("record_failure"));
        }
        self.logger.record_error(&error).map(|_record| ())
    }

    pub fn shutdown(&mut self) -> Result<ShutdownStatus, RuntimeError> {
        if self.shutdown {
            return Err(RuntimeError::already_shutdown("shutdown"));
        }
        self.logger
            .sync()
            .map_err(|error| RuntimeError::io("shutdown", error))?;
        self.shutdown = true;
        Ok(ShutdownStatus::Synchronized)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.logger.sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn invalid_input_is_logged_without_panicking() -> Result<(), RuntimeError> {
        let path = std::env::temp_dir().join("ada-based-rust-test.log");
        let mut runtime = Runtime::start(&path)?;
        let result = runtime.execute("");
        assert!(result.is_err());
        runtime.shutdown()?;
        let contents =
            fs::read_to_string(path).map_err(|error| RuntimeError::io("read_test_log", error))?;
        assert!(contents.contains("code=1 operation=execute"));
        Ok(())
    }

    #[test]
    fn shutdown_is_explicit_and_repeated_shutdown_is_rejected() -> Result<(), RuntimeError> {
        let path = std::env::temp_dir().join("ada-based-rust-shutdown-test.log");
        let mut runtime = Runtime::start(&path)?;
        assert_eq!(runtime.shutdown()?, ShutdownStatus::Synchronized);
        assert_eq!(
            runtime.shutdown().map(|_status| ()),
            Err(RuntimeError::already_shutdown("shutdown"))
        );
        Ok(())
    }

    #[test]
    fn startup_creates_a_missing_log_directory() -> Result<(), RuntimeError> {
        let path = std::env::temp_dir().join("ada-based-rust-bootstrap-test/nested/runtime.log");
        let mut runtime = Runtime::start(&path)?;
        runtime.shutdown()?;
        assert!(path.is_file());
        Ok(())
    }

    #[test]
    fn log_path_is_fully_qualified() -> Result<(), RuntimeError> {
        let path = std::env::temp_dir().join("ada-based-rust-qualified/runtime.log");
        let runtime = Runtime::start(&path)?;
        assert!(runtime.log_path().is_absolute());
        Ok(())
    }
}
