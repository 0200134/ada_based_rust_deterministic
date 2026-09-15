use ada_based_rust::{parse_expression, verify_normalization, verify_reference_suite, FormalError};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    match arguments.next().as_deref() {
        None | Some("--suite") => {
            verify_reference_suite()?;
            println!("Rust formal checker: reference suite passed");
        }
        Some("--expr") => {
            let input = arguments.next().ok_or_else(|| {
                FormalError::InvalidArgument(String::from("--expr requires an expression"))
            })?;
            if arguments.next().is_some() {
                return Err(FormalError::InvalidArgument(String::from(
                    "unexpected argument after expression",
                ))
                .into());
            }
            let expression = parse_expression(&input)?;
            verify_normalization(&expression)?;
            println!("Rust formal checker: expression preservation passed");
        }
        Some(argument) => {
            return Err(
                FormalError::InvalidArgument(format!("unknown argument '{argument}'")).into(),
            );
        }
    }
    Ok(())
}
