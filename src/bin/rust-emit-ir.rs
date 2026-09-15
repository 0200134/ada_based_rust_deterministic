use ada_based_rust::compile_expression;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    let expression = match (arguments.next().as_deref(), arguments.next()) {
        (Some("--expr"), Some(expression)) if arguments.next().is_none() => expression,
        _ => return Err("usage: rust-emit-ir --expr EXPR".into()),
    };
    let module = compile_expression(&expression)?;
    print!("{}", module.emit_text()?);
    Ok(())
}
