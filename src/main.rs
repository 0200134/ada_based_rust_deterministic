use ada_based_rust::{Runtime, RuntimeError};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("ada-based-rust error: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), RuntimeError> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        println!("usage: ada-based-rust [--log-path PATH | --log-path=PATH]");
        return Ok(());
    }

    let log_path = parse_log_path(arguments.into_iter())?;
    let mut runtime = Runtime::start(log_path)?;
    let operation_result = runtime.execute("bootstrap");
    if let Err(error) = operation_result {
        runtime.fail(error)?;
    }
    runtime.shutdown()?;
    println!(
        "ada-based-rust: log synchronized at {}",
        runtime.log_path().display()
    );
    Ok(())
}

fn parse_log_path(mut arguments: impl Iterator<Item = String>) -> Result<PathBuf, RuntimeError> {
    match (arguments.next().as_deref(), arguments.next()) {
        (None, None) => Ok(default_log_path()),
        (Some("--log-path"), Some(path)) if !path.is_empty() => Ok(PathBuf::from(path)),
        (Some(argument), None) if argument.starts_with("--log-path=") => {
            let path = argument.trim_start_matches("--log-path=");
            if path.is_empty() {
                Err(RuntimeError::invalid_input(
                    "parse_args",
                    "--log-path requires a non-empty path",
                ))
            } else {
                Ok(PathBuf::from(path))
            }
        }
        _ => Err(RuntimeError::invalid_input(
            "parse_args",
            "usage: ada-based-rust [--log-path PATH | --log-path=PATH]",
        )),
    }
}

fn default_log_path() -> PathBuf {
    let mut directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if directory.join("Cargo.toml").is_file() {
            return directory.join("target/ada-based-rust.log");
        }
        if !directory.pop() {
            return env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(Path::new("target").join("ada-based-rust.log"));
        }
    }
}
