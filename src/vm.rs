use crate::formal::{FormalError, ProofExpr};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpCode {
    Push(i64),
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,
    Eq,
    Lt,
    Select,
    Return,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    code: Vec<OpCode>,
}

impl Program {
    pub fn compile(expression: &ProofExpr) -> Result<Self, VmError> {
        let mut code = Vec::new();
        emit_expression(expression, &mut code)?;
        code.push(OpCode::Return);
        Ok(Self { code })
    }

    pub fn execute(&self) -> Result<i64, VmError> {
        let mut stack = Vec::new();
        for instruction in &self.code {
            match instruction {
                OpCode::Push(value) => stack.push(*value),
                OpCode::Add => binary(&mut stack, i64::checked_add, "add")?,
                OpCode::Sub => binary(&mut stack, i64::checked_sub, "sub")?,
                OpCode::Mul => binary(&mut stack, i64::checked_mul, "mul")?,
                OpCode::Div => division(&mut stack, false)?,
                OpCode::Rem => division(&mut stack, true)?,
                OpCode::Neg => {
                    let value = stack.pop().ok_or(VmError::StackUnderflow)?;
                    stack.push(value.checked_neg().ok_or(VmError::Overflow("neg"))?);
                }
                OpCode::Eq => comparison(&mut stack, true)?,
                OpCode::Lt => comparison(&mut stack, false)?,
                OpCode::Select => select(&mut stack)?,
                OpCode::Return => {
                    let value = stack.pop().ok_or(VmError::StackUnderflow)?;
                    if stack.is_empty() {
                        return Ok(value);
                    }
                    return Err(VmError::InvalidProgram("values remain on stack"));
                }
            }
        }
        Err(VmError::InvalidProgram("program has no return"))
    }

    pub fn code(&self) -> &[OpCode] {
        &self.code
    }
}

fn emit_expression(expression: &ProofExpr, code: &mut Vec<OpCode>) -> Result<(), VmError> {
    match expression {
        ProofExpr::Literal(value) => code.push(OpCode::Push(*value)),
        ProofExpr::Add(left, right) => emit_binary(left, right, code, OpCode::Add)?,
        ProofExpr::Sub(left, right) => emit_binary(left, right, code, OpCode::Sub)?,
        ProofExpr::Mul(left, right) => emit_binary(left, right, code, OpCode::Mul)?,
        ProofExpr::Div(left, right) => emit_binary(left, right, code, OpCode::Div)?,
        ProofExpr::Rem(left, right) => emit_binary(left, right, code, OpCode::Rem)?,
        ProofExpr::Neg(operand) => {
            emit_expression(operand, code)?;
            code.push(OpCode::Neg);
        }
        ProofExpr::Eq(left, right) => emit_binary(left, right, code, OpCode::Eq)?,
        ProofExpr::Lt(left, right) => emit_binary(left, right, code, OpCode::Lt)?,
        ProofExpr::If(condition, then_value, else_value) => {
            emit_expression(condition, code)?;
            emit_expression(then_value, code)?;
            emit_expression(else_value, code)?;
            code.push(OpCode::Select);
        }
    }
    Ok(())
}

fn comparison(stack: &mut Vec<i64>, equality: bool) -> Result<(), VmError> {
    let right = stack.pop().ok_or(VmError::StackUnderflow)?;
    let left = stack.pop().ok_or(VmError::StackUnderflow)?;
    stack.push(if equality {
        i64::from(left == right)
    } else {
        i64::from(left < right)
    });
    Ok(())
}

fn select(stack: &mut Vec<i64>) -> Result<(), VmError> {
    let else_value = stack.pop().ok_or(VmError::StackUnderflow)?;
    let then_value = stack.pop().ok_or(VmError::StackUnderflow)?;
    let condition = stack.pop().ok_or(VmError::StackUnderflow)?;
    stack.push(if condition != 0 {
        then_value
    } else {
        else_value
    });
    Ok(())
}

fn emit_binary(
    left: &ProofExpr,
    right: &ProofExpr,
    code: &mut Vec<OpCode>,
    operation: OpCode,
) -> Result<(), VmError> {
    emit_expression(left, code)?;
    emit_expression(right, code)?;
    code.push(operation);
    Ok(())
}

fn binary(
    stack: &mut Vec<i64>,
    operation: fn(i64, i64) -> Option<i64>,
    name: &'static str,
) -> Result<(), VmError> {
    let right = stack.pop().ok_or(VmError::StackUnderflow)?;
    let left = stack.pop().ok_or(VmError::StackUnderflow)?;
    stack.push(operation(left, right).ok_or(VmError::Overflow(name))?);
    Ok(())
}

fn division(stack: &mut Vec<i64>, remainder: bool) -> Result<(), VmError> {
    let right = stack.pop().ok_or(VmError::StackUnderflow)?;
    let left = stack.pop().ok_or(VmError::StackUnderflow)?;
    if right == 0 {
        return Err(VmError::DivisionByZero);
    }
    let value = if remainder {
        left.checked_rem(right)
    } else {
        left.checked_div(right)
    };
    stack.push(value.ok_or(VmError::Overflow(if remainder { "rem" } else { "div" }))?);
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmError {
    StackUnderflow,
    Overflow(&'static str),
    DivisionByZero,
    InvalidProgram(&'static str),
    Formal(FormalError),
}

impl fmt::Display for VmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "virtual machine error: {self:?}")
    }
}

impl std::error::Error for VmError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::parse_expression;

    #[test]
    fn executes_expression_without_external_runtime() -> Result<(), VmError> {
        let expression = parse_expression("2 + 3 * (5 - 1)").map_err(VmError::Formal)?;
        let program = Program::compile(&expression)?;
        assert_eq!(program.execute()?, 14);
        Ok(())
    }

    #[test]
    fn rejects_division_by_zero() -> Result<(), VmError> {
        let expression = parse_expression("1 / 0").map_err(VmError::Formal)?;
        let program = Program::compile(&expression)?;
        assert_eq!(program.execute(), Err(VmError::DivisionByZero));
        Ok(())
    }

    #[test]
    fn executes_conditional_expression() -> Result<(), VmError> {
        let expression = parse_expression("if 1 < 2 then 7 else 9").map_err(VmError::Formal)?;
        let program = Program::compile(&expression)?;
        assert_eq!(program.execute()?, 7);
        Ok(())
    }

    #[test]
    fn executes_nested_conditional_expression() -> Result<(), VmError> {
        let expression = parse_expression("if 1 < 2 then if 3 < 2 then 7 else 8 else 9")
            .map_err(VmError::Formal)?;
        let program = Program::compile(&expression)?;
        assert_eq!(program.execute()?, 8);
        Ok(())
    }
}
