use ada_based_rust::{compile_expression, parse_expression, verify_normalization, Program};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    if arguments.next().as_deref() != Some("--expr") {
        return Err("usage: rust-native-pipeline --expr EXPR".into());
    }
    let input = arguments.next().ok_or("--expr requires an expression")?;
    if arguments.next().is_some() {
        return Err("unexpected argument after expression".into());
    }

    let proof_expression = parse_expression(&input)?;
    verify_normalization(&proof_expression)?;
    let module = compile_expression(&input)?;
    let ir = module.emit_text()?;
    let bytecode = Program::compile(&proof_expression)?;
    let result = bytecode.execute()?;

    println!("source: {input}");
    println!("formal: passed");
    println!("ir: verified");
    println!("ir-bytes: {}", ir.len());
    println!("bytecode-ops: {}", bytecode.code().len());
    println!("result: {result}");
    Ok(())
}
