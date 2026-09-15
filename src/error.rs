use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ErrorCode {
    InvalidInput = 1,
    Io = 2,
    AlreadyShutdown = 3,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RuntimeError {
    pub code: ErrorCode,
    pub operation: &'static str,
    pub detail: String,
}

impl RuntimeError {
    pub fn invalid_input(operation: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidInput,
            operation,
            detail: detail.into(),
        }
    }

    pub fn io(operation: &'static str, error: std::io::Error) -> Self {
        Self {
            code: ErrorCode::Io,
            operation,
            detail: error.to_string(),
        }
    }

    pub fn already_shutdown(operation: &'static str) -> Self {
        Self {
            code: ErrorCode::AlreadyShutdown,
            operation,
            detail: String::from("runtime has already shut down"),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} ({}): {}",
            self.operation, self.code as u16, self.detail
        )
    }
}

impl std::error::Error for RuntimeError {}
