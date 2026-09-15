use crate::error::RuntimeError;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct LogRecord {
    pub sequence: u64,
    pub code: u16,
    pub operation: String,
    pub detail: String,
}

#[derive(Debug)]
pub struct Logger {
    path: PathBuf,
    file: File,
    next_sequence: u64,
}

impl Logger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| RuntimeError::io("resolve_log_path", error))?
                .join(path)
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| RuntimeError::io("create_log_directory", error))?;
            }
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| RuntimeError::io("open_log", error))?;
        Ok(Self {
            path,
            file,
            next_sequence: 1,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn record_error(&mut self, error: &RuntimeError) -> Result<LogRecord, RuntimeError> {
        let record = LogRecord {
            sequence: self.next_sequence,
            code: error.code as u16,
            operation: error.operation.to_owned(),
            detail: error.detail.clone(),
        };
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or_else(|| RuntimeError::invalid_input("record_error", "log sequence exhausted"))?;
        writeln!(
            self.file,
            "seq={} code={} operation={} detail={}",
            record.sequence, record.code, record.operation, record.detail
        )
        .map_err(|error| RuntimeError::io("write_log", error))?;
        self.sync()
            .map_err(|error| RuntimeError::io("sync_log", error))?;
        Ok(record)
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.file.flush()?;
        self.file.sync_all()
    }
}
