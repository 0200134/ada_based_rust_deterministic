use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofExpr {
    Literal(i64),
    Add(Box<ProofExpr>, Box<ProofExpr>),
    Sub(Box<ProofExpr>, Box<ProofExpr>),
    Mul(Box<ProofExpr>, Box<ProofExpr>),
    Div(Box<ProofExpr>, Box<ProofExpr>),
    Rem(Box<ProofExpr>, Box<ProofExpr>),
    Neg(Box<ProofExpr>),
    Eq(Box<ProofExpr>, Box<ProofExpr>),
    Lt(Box<ProofExpr>, Box<ProofExpr>),
    If(Box<ProofExpr>, Box<ProofExpr>, Box<ProofExpr>),
}

impl ProofExpr {
    pub fn literal_expr(value: i64) -> Self {
        Self::Literal(value)
    }

    pub fn add_expr(left: Self, right: Self) -> Self {
        Self::Add(Box::new(left), Box::new(right))
    }

    pub fn sub_expr(left: Self, right: Self) -> Self {
        Self::Sub(Box::new(left), Box::new(right))
    }

    pub fn mul_expr(left: Self, right: Self) -> Self {
        Self::Mul(Box::new(left), Box::new(right))
    }

    pub fn div_expr(left: Self, right: Self) -> Self {
        Self::Div(Box::new(left), Box::new(right))
    }

    pub fn rem_expr(left: Self, right: Self) -> Self {
        Self::Rem(Box::new(left), Box::new(right))
    }

    pub fn neg_expr(operand: Self) -> Self {
        Self::Neg(Box::new(operand))
    }

    pub fn eq_expr(left: Self, right: Self) -> Self {
        Self::Eq(Box::new(left), Box::new(right))
    }

    pub fn lt_expr(left: Self, right: Self) -> Self {
        Self::Lt(Box::new(left), Box::new(right))
    }

    pub fn if_expr(condition: Self, then_value: Self, else_value: Self) -> Self {
        Self::If(
            Box::new(condition),
            Box::new(then_value),
            Box::new(else_value),
        )
    }

    pub fn evaluate(&self) -> Result<i64, FormalError> {
        match self {
            Self::Literal(value) => Ok(*value),
            Self::Add(left, right) => checked_binary(left, right, i64::checked_add, "add"),
            Self::Sub(left, right) => checked_binary(left, right, i64::checked_sub, "sub"),
            Self::Mul(left, right) => checked_binary(left, right, i64::checked_mul, "mul"),
            Self::Div(left, right) => checked_division(left, right, false),
            Self::Rem(left, right) => checked_division(left, right, true),
            Self::Neg(operand) => operand
                .evaluate()?
                .checked_neg()
                .ok_or(FormalError::ArithmeticOverflow("neg")),
            Self::Eq(left, right) => Ok(i64::from(left.evaluate()? == right.evaluate()?)),
            Self::Lt(left, right) => Ok(i64::from(left.evaluate()? < right.evaluate()?)),
            Self::If(condition, then_value, else_value) => {
                if condition.evaluate()? != 0 {
                    then_value.evaluate()
                } else {
                    else_value.evaluate()
                }
            }
        }
    }

    pub fn normalize(&self) -> Result<Self, FormalError> {
        match self {
            Self::Literal(value) => Ok(Self::Literal(*value)),
            Self::Add(left, right) => normalize_binary(left, right, i64::checked_add, "add"),
            Self::Sub(left, right) => normalize_binary(left, right, i64::checked_sub, "sub"),
            Self::Mul(left, right) => normalize_binary(left, right, i64::checked_mul, "mul"),
            Self::Div(left, right) => normalize_division(left, right, false),
            Self::Rem(left, right) => normalize_division(left, right, true),
            Self::Neg(operand) => {
                let value = operand.normalize()?.evaluate()?;
                Ok(Self::Literal(
                    value
                        .checked_neg()
                        .ok_or(FormalError::ArithmeticOverflow("neg"))?,
                ))
            }
            Self::Eq(left, right) => normalize_comparison(left, right, true),
            Self::Lt(left, right) => normalize_comparison(left, right, false),
            Self::If(condition, then_value, else_value) => {
                let condition = condition.normalize()?.evaluate()?;
                if condition != 0 {
                    then_value.normalize()
                } else {
                    else_value.normalize()
                }
            }
        }
    }
}

fn normalize_comparison(
    left: &ProofExpr,
    right: &ProofExpr,
    equality: bool,
) -> Result<ProofExpr, FormalError> {
    let left = left.normalize()?.evaluate()?;
    let right = right.normalize()?.evaluate()?;
    Ok(ProofExpr::literal_expr(if equality {
        i64::from(left == right)
    } else {
        i64::from(left < right)
    }))
}

fn checked_division(
    left: &ProofExpr,
    right: &ProofExpr,
    remainder: bool,
) -> Result<i64, FormalError> {
    let left_value = left.evaluate()?;
    let right_value = right.evaluate()?;
    if right_value == 0 {
        return Err(FormalError::DivisionByZero);
    }
    let result = if remainder {
        left_value.checked_rem(right_value)
    } else {
        left_value.checked_div(right_value)
    };
    result.ok_or(FormalError::ArithmeticOverflow(if remainder {
        "rem"
    } else {
        "div"
    }))
}

fn normalize_division(
    left: &ProofExpr,
    right: &ProofExpr,
    remainder: bool,
) -> Result<ProofExpr, FormalError> {
    let value = checked_division(&left.normalize()?, &right.normalize()?, remainder)?;
    Ok(ProofExpr::Literal(value))
}

pub fn parse_expression(input: &str) -> Result<ProofExpr, FormalError> {
    let mut parser = Parser {
        input: input.as_bytes(),
        position: 0,
    };
    let expression = parser.parse_comparison()?;
    parser.skip_whitespace();
    if parser.position != parser.input.len() {
        return Err(FormalError::Syntax {
            position: parser.position,
            detail: "unexpected input",
        });
    }
    Ok(expression)
}

struct Parser<'input> {
    input: &'input [u8],
    position: usize,
}

impl<'input> Parser<'input> {
    fn parse_comparison(&mut self) -> Result<ProofExpr, FormalError> {
        let mut expression = self.parse_additive()?;
        loop {
            self.skip_whitespace();
            let operation = if self.input.get(self.position..self.position + 2) == Some(b"==") {
                Some(ProofOperation::Eq)
            } else if self.peek() == Some(b'<') {
                Some(ProofOperation::Lt)
            } else {
                None
            };
            let Some(operation) = operation else { break };
            self.position += if matches!(operation, ProofOperation::Eq) {
                2
            } else {
                1
            };
            expression = operation.build(expression, self.parse_additive()?);
        }
        Ok(expression)
    }

    fn parse_additive(&mut self) -> Result<ProofExpr, FormalError> {
        let mut expression = self.parse_multiplicative()?;
        loop {
            self.skip_whitespace();
            let operation = match self.peek() {
                Some(b'+') => ProofOperation::Add,
                Some(b'-') => ProofOperation::Sub,
                _ => break,
            };
            self.position += 1;
            let right = self.parse_multiplicative()?;
            expression = operation.build(expression, right);
        }
        Ok(expression)
    }

    fn parse_multiplicative(&mut self) -> Result<ProofExpr, FormalError> {
        let mut expression = self.parse_primary()?;
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'*') {
                let operation = match self.peek() {
                    Some(b'/') => ProofOperation::Div,
                    Some(b'%') => ProofOperation::Rem,
                    _ => break,
                };
                self.position += 1;
                let right = self.parse_primary()?;
                expression = operation.build(expression, right);
                continue;
            }
            self.position += 1;
            expression = ProofExpr::mul_expr(expression, self.parse_primary()?);
        }
        Ok(expression)
    }

    fn parse_primary(&mut self) -> Result<ProofExpr, FormalError> {
        self.skip_whitespace();
        if self.consume_keyword("if") {
            let condition = self.parse_comparison()?;
            if !self.consume_keyword("then") {
                return Err(FormalError::Syntax {
                    position: self.position,
                    detail: "expected 'then'",
                });
            }
            let then_value = self.parse_comparison()?;
            if !self.consume_keyword("else") {
                return Err(FormalError::Syntax {
                    position: self.position,
                    detail: "expected 'else'",
                });
            }
            return Ok(ProofExpr::if_expr(
                condition,
                then_value,
                self.parse_comparison()?,
            ));
        }
        if matches!(self.peek(), Some(b'+') | Some(b'-')) {
            let negative = self.peek() == Some(b'-');
            self.position += 1;
            let operand = self.parse_primary()?;
            return Ok(if negative {
                ProofExpr::neg_expr(operand)
            } else {
                operand
            });
        }
        if self.peek() == Some(b'(') {
            self.position += 1;
            let expression = self.parse_additive()?;
            self.skip_whitespace();
            if self.peek() != Some(b')') {
                return Err(FormalError::Syntax {
                    position: self.position,
                    detail: "expected ')'",
                });
            }
            self.position += 1;
            return Ok(expression);
        }

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
                .ok_or(FormalError::Syntax {
                    position: start,
                    detail: "integer overflow",
                })?;
            self.position += 1;
        }
        if !found_digit {
            return Err(FormalError::Syntax {
                position: self.position,
                detail: "expected integer",
            });
        }
        Ok(ProofExpr::literal_expr(value))
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

enum ProofOperation {
    Add,
    Sub,
    Div,
    Rem,
    Eq,
    Lt,
}

impl ProofOperation {
    fn build(self, left: ProofExpr, right: ProofExpr) -> ProofExpr {
        match self {
            Self::Add => ProofExpr::add_expr(left, right),
            Self::Sub => ProofExpr::sub_expr(left, right),
            Self::Div => ProofExpr::div_expr(left, right),
            Self::Rem => ProofExpr::rem_expr(left, right),
            Self::Eq => ProofExpr::eq_expr(left, right),
            Self::Lt => ProofExpr::lt_expr(left, right),
        }
    }
}

fn checked_binary(
    left: &ProofExpr,
    right: &ProofExpr,
    operation: fn(i64, i64) -> Option<i64>,
    name: &'static str,
) -> Result<i64, FormalError> {
    let left_value = left.evaluate()?;
    let right_value = right.evaluate()?;
    operation(left_value, right_value).ok_or(FormalError::ArithmeticOverflow(name))
}

fn normalize_binary(
    left: &ProofExpr,
    right: &ProofExpr,
    operation: fn(i64, i64) -> Option<i64>,
    name: &'static str,
) -> Result<ProofExpr, FormalError> {
    let left_normalized = left.normalize()?;
    let right_normalized = right.normalize()?;
    let left_value = left_normalized.evaluate()?;
    let right_value = right_normalized.evaluate()?;
    let result = operation(left_value, right_value).ok_or(FormalError::ArithmeticOverflow(name))?;
    Ok(ProofExpr::Literal(result))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormalError {
    ArithmeticOverflow(&'static str),
    DivisionByZero,
    Counterexample {
        before: i64,
        after: i64,
    },
    Syntax {
        position: usize,
        detail: &'static str,
    },
    InvalidArgument(String),
}

impl fmt::Display for FormalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow(operation) => {
                write!(formatter, "formal model {operation} overflow")
            }
            Self::DivisionByZero => write!(formatter, "formal model division by zero"),
            Self::Counterexample { before, after } => {
                write!(formatter, "normalization changed {before} into {after}")
            }
            Self::Syntax { position, detail } => {
                write!(formatter, "formal expression error at {position}: {detail}")
            }
            Self::InvalidArgument(detail) => write!(formatter, "invalid argument: {detail}"),
        }
    }
}

impl std::error::Error for FormalError {}

pub fn verify_normalization(expression: &ProofExpr) -> Result<(), FormalError> {
    let before = expression.evaluate()?;
    let after = expression.normalize()?.evaluate()?;
    if before == after {
        Ok(())
    } else {
        Err(FormalError::Counterexample { before, after })
    }
}

pub fn verify_reference_suite() -> Result<(), FormalError> {
    let expressions = [
        ProofExpr::add_expr(ProofExpr::literal_expr(2), ProofExpr::literal_expr(3)),
        ProofExpr::sub_expr(
            ProofExpr::literal_expr(20),
            ProofExpr::mul_expr(ProofExpr::literal_expr(3), ProofExpr::literal_expr(4)),
        ),
        ProofExpr::mul_expr(
            ProofExpr::add_expr(ProofExpr::literal_expr(2), ProofExpr::literal_expr(3)),
            ProofExpr::literal_expr(5),
        ),
        ProofExpr::div_expr(ProofExpr::literal_expr(20), ProofExpr::literal_expr(4)),
        ProofExpr::rem_expr(ProofExpr::literal_expr(21), ProofExpr::literal_expr(4)),
    ];
    for expression in &expressions {
        verify_normalization(expression)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_preserves_checked_evaluation() -> Result<(), FormalError> {
        let expression = ProofExpr::mul_expr(
            ProofExpr::add_expr(ProofExpr::literal_expr(2), ProofExpr::literal_expr(3)),
            ProofExpr::sub_expr(ProofExpr::literal_expr(9), ProofExpr::literal_expr(4)),
        );
        verify_normalization(&expression)?;
        assert_eq!(expression.normalize()?.evaluate()?, 25);
        Ok(())
    }

    #[test]
    fn overflow_is_rejected_instead_of_wrapping() {
        let expression = ProofExpr::add_expr(
            ProofExpr::literal_expr(i64::MAX),
            ProofExpr::literal_expr(1),
        );
        assert_eq!(
            verify_normalization(&expression),
            Err(FormalError::ArithmeticOverflow("add"))
        );
    }

    #[test]
    fn parser_respects_precedence_and_parentheses() -> Result<(), FormalError> {
        let expression = parse_expression("-2 + 12 / 3 % 2")?;
        verify_normalization(&expression)?;
        assert_eq!(expression.evaluate()?, -2);
        Ok(())
    }

    #[test]
    fn division_by_zero_is_rejected() {
        let expression =
            ProofExpr::div_expr(ProofExpr::literal_expr(1), ProofExpr::literal_expr(0));
        assert_eq!(
            verify_normalization(&expression),
            Err(FormalError::DivisionByZero)
        );
    }

    #[test]
    fn conditional_selects_false_branch() -> Result<(), FormalError> {
        let expression = parse_expression("if 2 < 1 then 7 else 9")?;
        verify_normalization(&expression)?;
        assert_eq!(expression.evaluate()?, 9);
        Ok(())
    }
}
