use crate::backend::{Function, Instruction, IntegerWidth, Module, ValueId};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrontendError {
    EmptyInput,
    ExpectedInteger(usize),
    InvalidCharacter {
        position: usize,
        character: char,
    },
    IntegerOverflow(usize),
    ExpectedClosingParen(usize),
    ExpectedKeyword {
        position: usize,
        keyword: &'static str,
    },
    TrailingInput(usize),
}

impl fmt::Display for FrontendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "expression is empty"),
            Self::ExpectedInteger(position) => {
                write!(formatter, "expected integer at position {position}")
            }
            Self::InvalidCharacter {
                position,
                character,
            } => {
                write!(
                    formatter,
                    "invalid character '{character}' at position {position}"
                )
            }
            Self::IntegerOverflow(position) => {
                write!(formatter, "integer overflows at position {position}")
            }
            Self::ExpectedClosingParen(position) => {
                write!(formatter, "expected ')' at position {position}")
            }
            Self::ExpectedKeyword { position, keyword } => {
                write!(formatter, "expected '{keyword}' at position {position}")
            }
            Self::TrailingInput(position) => {
                write!(formatter, "unexpected input at position {position}")
            }
        }
    }
}

impl std::error::Error for FrontendError {}

pub fn compile_expression(input: &str) -> Result<Module, FrontendError> {
    let mut parser = Parser::new(input);
    let mut function = Function::new("main", IntegerWidth::I64);
    let result = parser.parse_comparison(&mut function)?;
    parser.skip_whitespace();
    if parser.position < parser.input.len() {
        return Err(FrontendError::TrailingInput(parser.position));
    }
    function.push(Instruction::Return { value: result });

    let mut module = Module::new();
    module.add_function(function);
    module
        .verify()
        .map_err(|_| FrontendError::TrailingInput(parser.position))?;
    Ok(module)
}

struct Parser<'input> {
    input: &'input [u8],
    position: usize,
    next_value: u32,
}

impl<'input> Parser<'input> {
    fn new(input: &'input str) -> Self {
        Self {
            input: input.as_bytes(),
            position: 0,
            next_value: 0,
        }
    }

    fn parse_expression(&mut self, function: &mut Function) -> Result<ValueId, FrontendError> {
        self.skip_whitespace();
        if self.position == self.input.len() {
            return Err(FrontendError::EmptyInput);
        }

        let mut result = self.parse_multiplicative(function)?;
        loop {
            self.skip_whitespace();
            let operation = match self.peek() {
                Some(b'+') => InstructionKind::Add,
                Some(b'-') => InstructionKind::Sub,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_multiplicative(function)?;
            let result_id = self.allocate_value();
            function.push(operation.build(result_id, result, right));
            result = result_id;
        }
        Ok(result)
    }

    fn parse_comparison(&mut self, function: &mut Function) -> Result<ValueId, FrontendError> {
        let mut result = self.parse_expression(function)?;
        loop {
            self.skip_whitespace();
            let operation = if self.input.get(self.position..self.position + 2) == Some(b"==") {
                Some(InstructionKind::Eq)
            } else if self.peek() == Some(b'<') {
                Some(InstructionKind::Lt)
            } else {
                None
            };
            let Some(operation) = operation else { break };
            self.position += if matches!(operation, InstructionKind::Eq) {
                2
            } else {
                1
            };
            let right = self.parse_expression(function)?;
            let result_id = self.allocate_value();
            function.push(operation.build(result_id, result, right));
            result = result_id;
        }
        Ok(result)
    }

    fn parse_multiplicative(&mut self, function: &mut Function) -> Result<ValueId, FrontendError> {
        let mut result = self.parse_primary(function)?;
        loop {
            self.skip_whitespace();
            let operation = match self.peek() {
                Some(b'*') => InstructionKind::Mul,
                Some(b'/') => InstructionKind::Div,
                Some(b'%') => InstructionKind::Rem,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_primary(function)?;
            let result_id = self.allocate_value();
            function.push(operation.build(result_id, result, right));
            result = result_id;
        }
        Ok(result)
    }

    fn parse_primary(&mut self, function: &mut Function) -> Result<ValueId, FrontendError> {
        self.skip_whitespace();
        if self.consume_keyword("if") {
            let condition = self.parse_comparison(function)?;
            if !self.consume_keyword("then") {
                return Err(FrontendError::ExpectedKeyword {
                    position: self.position,
                    keyword: "then",
                });
            }
            let then_value = self.parse_comparison(function)?;
            if !self.consume_keyword("else") {
                return Err(FrontendError::ExpectedKeyword {
                    position: self.position,
                    keyword: "else",
                });
            }
            let else_value = self.parse_comparison(function)?;
            let result = self.allocate_value();
            function.push(Instruction::Select {
                result,
                condition,
                then_value,
                else_value,
            });
            return Ok(result);
        }
        if matches!(self.peek(), Some(b'+') | Some(b'-')) {
            let negative = self.peek() == Some(b'-');
            self.position += 1;
            let operand = self.parse_primary(function)?;
            if !negative {
                return Ok(operand);
            }
            let result = self.allocate_value();
            function.push(Instruction::Neg { result, operand });
            return Ok(result);
        }
        if self.peek() == Some(b'(') {
            self.position += 1;
            let result = self.parse_expression(function)?;
            self.skip_whitespace();
            if self.peek() != Some(b')') {
                return Err(FrontendError::ExpectedClosingParen(self.position));
            }
            self.position += 1;
            return Ok(result);
        }
        self.parse_integer(function)
    }

    fn parse_integer(&mut self, function: &mut Function) -> Result<ValueId, FrontendError> {
        self.skip_whitespace();
        let start = self.position;
        let mut value = 0_i64;
        let mut found_digit = false;
        while let Some(character) = self.peek() {
            if !character.is_ascii_digit() {
                break;
            }
            found_digit = true;
            value = value
                .checked_mul(10)
                .and_then(|current| current.checked_add((character - b'0') as i64))
                .ok_or(FrontendError::IntegerOverflow(start))?;
            self.position += 1;
        }
        if !found_digit {
            if let Some(character) = self.peek() {
                return Err(if character.is_ascii() {
                    FrontendError::ExpectedInteger(self.position)
                } else {
                    FrontendError::InvalidCharacter {
                        position: self.position,
                        character: character as char,
                    }
                });
            }
            return Err(FrontendError::ExpectedInteger(self.position));
        }

        let result = self.allocate_value();
        function.push(Instruction::Constant { result, value });
        Ok(result)
    }

    fn allocate_value(&mut self) -> ValueId {
        let value = ValueId::new(self.next_value);
        self.next_value += 1;
        value
    }

    fn skip_whitespace(&mut self) {
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_whitespace())
        {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        self.skip_whitespace();
        let bytes = keyword.as_bytes();
        if self.input.get(self.position..self.position + bytes.len()) != Some(bytes) {
            return false;
        }
        let end = self.position + bytes.len();
        if self
            .input
            .get(end)
            .is_some_and(|character| character.is_ascii_alphanumeric())
        {
            return false;
        }
        self.position = end;
        true
    }
}

#[derive(Clone, Copy)]
enum InstructionKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Lt,
}

impl InstructionKind {
    fn build(self, result: ValueId, left: ValueId, right: ValueId) -> Instruction {
        match self {
            Self::Add => Instruction::Add {
                result,
                left,
                right,
            },
            Self::Sub => Instruction::Sub {
                result,
                left,
                right,
            },
            Self::Mul => Instruction::Mul {
                result,
                left,
                right,
            },
            Self::Div => Instruction::Div {
                result,
                left,
                right,
            },
            Self::Rem => Instruction::Rem {
                result,
                left,
                right,
            },
            Self::Eq => Instruction::Eq {
                result,
                left,
                right,
            },
            Self::Lt => Instruction::Lt {
                result,
                left,
                right,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_expression_to_verified_ir() -> Result<(), FrontendError> {
        let module = compile_expression("2 + 3 * (5 - 1)")?;
        let text = module
            .emit_text()
            .map_err(|_| FrontendError::TrailingInput(0))?;
        assert!(text.contains("define i64 @main()"));
        assert!(text.contains("%6 = add i64 %0, %5"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_expression_without_panicking() {
        assert_eq!(
            compile_expression("2 + x"),
            Err(FrontendError::ExpectedInteger(4))
        );
    }

    #[test]
    fn compiles_unary_division_and_remainder() -> Result<(), FrontendError> {
        let module = compile_expression("-20 / 3 % 2")?;
        let text = module
            .emit_text()
            .map_err(|_| FrontendError::TrailingInput(0))?;
        assert!(text.contains("sdiv i64"));
        assert!(text.contains("srem i64"));
        Ok(())
    }

    #[test]
    fn compiles_conditional_expression() -> Result<(), FrontendError> {
        let module = compile_expression("if 1 < 2 then 7 else 9")?;
        assert!(module.emit_text().is_ok());
        Ok(())
    }
}
