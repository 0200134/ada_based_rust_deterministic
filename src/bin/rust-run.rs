use ada_based_rust::{parse_expression, Program};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    if arguments.next().as_deref() != Some("--expr") {
        return Err("usage: rust-run --expr EXPR".into());
    }
    let input = arguments.next().ok_or("--expr requires an expression")?;
    if arguments.next().is_some() {
        return Err("unexpected argument after expression".into());
    }
    let expression = parse_expression(&input)?;
    let program = Program::compile(&expression)?;
    println!("{}", program.execute()?);
    Ok(())
}
