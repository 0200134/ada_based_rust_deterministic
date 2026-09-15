use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueId(u32);

impl ValueId {
    pub const fn new(number: u32) -> Self {
        Self(number)
    }

    pub const fn number(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegerWidth {
    I32,
    I64,
}

impl fmt::Display for IntegerWidth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(formatter, "i32"),
            Self::I64 => write!(formatter, "i64"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Constant {
        result: ValueId,
        value: i64,
    },
    Add {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Sub {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Mul {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Div {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Rem {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Neg {
        result: ValueId,
        operand: ValueId,
    },
    Eq {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Lt {
        result: ValueId,
        left: ValueId,
        right: ValueId,
    },
    Select {
        result: ValueId,
        condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    },
    Return {
        value: ValueId,
    },
}

impl Instruction {
    fn result(&self) -> Option<ValueId> {
        match self {
            Self::Constant { result, .. }
            | Self::Add { result, .. }
            | Self::Sub { result, .. }
            | Self::Mul { result, .. }
            | Self::Div { result, .. }
            | Self::Rem { result, .. }
            | Self::Neg { result, .. } => Some(*result),
            Self::Eq { result, .. } | Self::Lt { result, .. } => Some(*result),
            Self::Select { result, .. } => Some(*result),
            Self::Return { .. } => None,
        }
    }

    fn operands(&self) -> impl Iterator<Item = ValueId> + '_ {
        let operands = match self {
            Self::Constant { .. } => Vec::new(),
            Self::Add { left, right, .. }
            | Self::Sub { left, right, .. }
            | Self::Mul { left, right, .. }
            | Self::Div { left, right, .. }
            | Self::Rem { left, right, .. } => vec![*left, *right],
            Self::Neg { operand, .. } => vec![*operand],
            Self::Eq { left, right, .. } | Self::Lt { left, right, .. } => vec![*left, *right],
            Self::Select {
                condition,
                then_value,
                else_value,
                ..
            } => {
                vec![*condition, *then_value, *else_value]
            }
            Self::Return { value } => vec![*value],
        };
        operands.into_iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    name: String,
    return_width: IntegerWidth,
    instructions: Vec<Instruction>,
}

impl Function {
    pub fn new(name: impl Into<String>, return_width: IntegerWidth) -> Self {
        Self {
            name: name.into(),
            return_width,
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn verify(&self) -> Result<(), BackendError> {
        if self.name.is_empty() {
            return Err(BackendError::InvalidFunction("function name is empty"));
        }

        let mut defined = BTreeSet::new();
        let mut terminated = false;
        for instruction in &self.instructions {
            if terminated {
                return Err(BackendError::InvalidFunction(
                    "instruction appears after return",
                ));
            }
            for operand in instruction.operands() {
                if !defined.contains(&operand) {
                    return Err(BackendError::UnknownValue(operand));
                }
            }
            if let Some(result) = instruction.result() {
                if !defined.insert(result) {
                    return Err(BackendError::DuplicateValue(result));
                }
            }
            if matches!(instruction, Instruction::Return { .. }) {
                terminated = true;
            }
        }
        if !terminated {
            return Err(BackendError::InvalidFunction("function has no return"));
        }
        Ok(())
    }

    pub fn optimize(&mut self, level: OptimizationLevel) -> Result<(), BackendError> {
        self.verify()?;
        if level == OptimizationLevel::None {
            return Ok(());
        }

        let mut constants = BTreeMap::new();
        for instruction in &mut self.instructions {
            match instruction {
                Instruction::Constant { result, value } => {
                    constants.insert(*result, *value);
                }
                Instruction::Add {
                    result,
                    left,
                    right,
                }
                | Instruction::Sub {
                    result,
                    left,
                    right,
                }
                | Instruction::Mul {
                    result,
                    left,
                    right,
                }
                | Instruction::Div {
                    result,
                    left,
                    right,
                }
                | Instruction::Rem {
                    result,
                    left,
                    right,
                } => {
                    let result_id = *result;
                    let Some(left_value) = constants.get(left).copied() else {
                        continue;
                    };
                    let Some(right_value) = constants.get(right).copied() else {
                        continue;
                    };
                    let (value, operation) = match instruction {
                        Instruction::Add { .. } => (left_value.checked_add(right_value), "add"),
                        Instruction::Sub { .. } => (left_value.checked_sub(right_value), "sub"),
                        Instruction::Mul { .. } => (left_value.checked_mul(right_value), "mul"),
                        Instruction::Div { .. } => {
                            if right_value == 0 {
                                return Err(BackendError::DivisionByZero);
                            }
                            (left_value.checked_div(right_value), "div")
                        }
                        Instruction::Rem { .. } => {
                            if right_value == 0 {
                                return Err(BackendError::DivisionByZero);
                            }
                            (left_value.checked_rem(right_value), "rem")
                        }
                        _ => (None, "unknown"),
                    };
                    let value = value.ok_or(BackendError::ArithmeticOverflow(operation))?;
                    *instruction = Instruction::Constant {
                        result: result_id,
                        value,
                    };
                    constants.insert(result_id, value);
                }
                Instruction::Neg { result, operand } => {
                    let result_id = *result;
                    let Some(value) = constants.get(operand).copied() else {
                        continue;
                    };
                    let value = value
                        .checked_neg()
                        .ok_or(BackendError::ArithmeticOverflow("neg"))?;
                    *instruction = Instruction::Constant {
                        result: result_id,
                        value,
                    };
                    constants.insert(result_id, value);
                }
                Instruction::Eq {
                    result,
                    left,
                    right,
                } => {
                    let result_id = *result;
                    let left_value = constants.get(left).copied();
                    let right_value = constants.get(right).copied();
                    let (Some(left_value), Some(right_value)) = (left_value, right_value) else {
                        continue;
                    };
                    let value = i64::from(left_value == right_value);
                    *instruction = Instruction::Constant {
                        result: result_id,
                        value,
                    };
                    constants.insert(result_id, value);
                }
                Instruction::Lt {
                    result,
                    left,
                    right,
                } => {
                    let result_id = *result;
                    let left_value = constants.get(left).copied();
                    let right_value = constants.get(right).copied();
                    let (Some(left_value), Some(right_value)) = (left_value, right_value) else {
                        continue;
                    };
                    let value = i64::from(left_value < right_value);
                    *instruction = Instruction::Constant {
                        result: result_id,
                        value,
                    };
                    constants.insert(result_id, value);
                }
                Instruction::Select {
                    result,
                    condition,
                    then_value,
                    else_value,
                } => {
                    let (Some(condition), Some(then_value), Some(else_value)) = (
                        constants.get(condition).copied(),
                        constants.get(then_value).copied(),
                        constants.get(else_value).copied(),
                    ) else {
                        continue;
                    };
                    let result_id = *result;
                    let value = if condition != 0 {
                        then_value
                    } else {
                        else_value
                    };
                    *instruction = Instruction::Constant {
                        result: result_id,
                        value,
                    };
                    constants.insert(result_id, value);
                }
                Instruction::Return { .. } => {}
            }
        }
        Ok(())
    }

    fn emit(&self, output: &mut String) {
        output.push_str("define ");
        output.push_str(&self.return_width.to_string());
        output.push_str(" @");
        output.push_str(&self.name);
        output.push_str("() {\nentry:\n");
        for instruction in &self.instructions {
            match instruction {
                Instruction::Constant { result, value } => {
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = add ");
                    output.push_str(&self.return_width.to_string());
                    output.push(' ');
                    output.push_str(&value.to_string());
                    output.push_str(", 0\n");
                }
                Instruction::Add {
                    result,
                    left,
                    right,
                } => {
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = add ");
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&left.number().to_string());
                    output.push_str(", %");
                    output.push_str(&right.number().to_string());
                    output.push('\n');
                }
                Instruction::Sub {
                    result,
                    left,
                    right,
                }
                | Instruction::Mul {
                    result,
                    left,
                    right,
                }
                | Instruction::Div {
                    result,
                    left,
                    right,
                }
                | Instruction::Rem {
                    result,
                    left,
                    right,
                } => {
                    let operation = match instruction {
                        Instruction::Sub { .. } => "sub",
                        Instruction::Mul { .. } => "mul",
                        Instruction::Div { .. } => "sdiv",
                        Instruction::Rem { .. } => "srem",
                        _ => "unknown",
                    };
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = ");
                    output.push_str(operation);
                    output.push(' ');
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&left.number().to_string());
                    output.push_str(", %");
                    output.push_str(&right.number().to_string());
                    output.push('\n');
                }
                Instruction::Neg { result, operand } => {
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = sub ");
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" 0, %");
                    output.push_str(&operand.number().to_string());
                    output.push('\n');
                }
                Instruction::Eq {
                    result,
                    left,
                    right,
                }
                | Instruction::Lt {
                    result,
                    left,
                    right,
                } => {
                    let operation = if matches!(instruction, Instruction::Eq { .. }) {
                        "eq"
                    } else {
                        "slt"
                    };
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = icmp ");
                    output.push_str(operation);
                    output.push(' ');
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&left.number().to_string());
                    output.push_str(", %");
                    output.push_str(&right.number().to_string());
                    output.push('\n');
                }
                Instruction::Select {
                    result,
                    condition,
                    then_value,
                    else_value,
                } => {
                    output.push_str("  %");
                    output.push_str(&result.number().to_string());
                    output.push_str(" = select i1 %");
                    output.push_str(&condition.number().to_string());
                    output.push_str(", ");
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&then_value.number().to_string());
                    output.push_str(", ");
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&else_value.number().to_string());
                    output.push('\n');
                }
                Instruction::Return { value } => {
                    output.push_str("  ret ");
                    output.push_str(&self.return_width.to_string());
                    output.push_str(" %");
                    output.push_str(&value.number().to_string());
                    output.push('\n');
                }
            }
        }
        output.push_str("}\n");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Module {
    functions: Vec<Function>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn verify(&self) -> Result<(), BackendError> {
        let mut names = BTreeSet::new();
        for function in &self.functions {
            if !names.insert(function.name.as_str()) {
                return Err(BackendError::DuplicateFunction(function.name.clone()));
            }
            function.verify()?;
        }
        Ok(())
    }

    pub fn emit_text(&self) -> Result<String, BackendError> {
        self.verify()?;
        let mut output = String::from("; ada-based-rust deterministic IR\n");
        for function in &self.functions {
            function.emit(&mut output);
        }
        Ok(output)
    }

    pub fn optimize(&mut self, level: OptimizationLevel) -> Result<(), BackendError> {
        for function in &mut self.functions {
            function.optimize(level)?;
        }
        self.verify()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackendError {
    InvalidFunction(&'static str),
    ArithmeticOverflow(&'static str),
    DivisionByZero,
    InvalidComparison,
    UnknownValue(ValueId),
    DuplicateValue(ValueId),
    DuplicateFunction(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFunction(detail) => write!(formatter, "invalid function: {detail}"),
            Self::ArithmeticOverflow(operation) => {
                write!(formatter, "constant {operation} overflow")
            }
            Self::DivisionByZero => write!(formatter, "constant division by zero"),
            Self::InvalidComparison => write!(formatter, "invalid comparison"),
            Self::UnknownValue(value) => write!(formatter, "unknown value %{}", value.number()),
            Self::DuplicateValue(value) => {
                write!(formatter, "duplicate value %{}", value.number())
            }
            Self::DuplicateFunction(name) => write!(formatter, "duplicate function @{name}"),
        }
    }
}

impl std::error::Error for BackendError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_stable_llvm_like_text() -> Result<(), BackendError> {
        let mut function = Function::new("add_constants", IntegerWidth::I32);
        function.push(Instruction::Constant {
            result: ValueId::new(0),
            value: 2,
        });
        function.push(Instruction::Constant {
            result: ValueId::new(1),
            value: 3,
        });
        function.push(Instruction::Add {
            result: ValueId::new(2),
            left: ValueId::new(0),
            right: ValueId::new(1),
        });
        function.push(Instruction::Return {
            value: ValueId::new(2),
        });

        let mut module = Module::new();
        module.add_function(function);
        assert_eq!(
            module.emit_text()?,
            concat!(
                "; ada-based-rust deterministic IR\n",
                "define i32 @add_constants() {\n",
                "entry:\n",
                "  %0 = add i32 2, 0\n",
                "  %1 = add i32 3, 0\n",
                "  %2 = add i32 %0, %1\n",
                "  ret i32 %2\n",
                "}\n"
            )
        );
        Ok(())
    }

    #[test]
    fn rejects_unknown_values() {
        let mut function = Function::new("broken", IntegerWidth::I32);
        function.push(Instruction::Return {
            value: ValueId::new(99),
        });
        assert_eq!(
            function.verify(),
            Err(BackendError::UnknownValue(ValueId::new(99)))
        );
    }

    #[test]
    fn basic_optimization_folds_constant_addition() -> Result<(), BackendError> {
        let mut function = Function::new("fold", IntegerWidth::I32);
        function.push(Instruction::Constant {
            result: ValueId::new(0),
            value: 2,
        });
        function.push(Instruction::Constant {
            result: ValueId::new(1),
            value: 3,
        });
        function.push(Instruction::Add {
            result: ValueId::new(2),
            left: ValueId::new(0),
            right: ValueId::new(1),
        });
        function.push(Instruction::Return {
            value: ValueId::new(2),
        });

        function.optimize(OptimizationLevel::Basic)?;
        assert!(matches!(
            function.instructions[2],
            Instruction::Constant { value: 5, .. }
        ));
        Ok(())
    }

    #[test]
    fn basic_optimization_folds_select() -> Result<(), BackendError> {
        let mut function = Function::new("select", IntegerWidth::I64);
        function.push(Instruction::Constant {
            result: ValueId::new(0),
            value: 1,
        });
        function.push(Instruction::Constant {
            result: ValueId::new(1),
            value: 7,
        });
        function.push(Instruction::Constant {
            result: ValueId::new(2),
            value: 9,
        });
        function.push(Instruction::Select {
            result: ValueId::new(3),
            condition: ValueId::new(0),
            then_value: ValueId::new(1),
            else_value: ValueId::new(2),
        });
        function.push(Instruction::Return {
            value: ValueId::new(3),
        });
        function.optimize(OptimizationLevel::Basic)?;
        assert!(matches!(
            function.instructions[3],
            Instruction::Constant { value: 7, .. }
        ));
        Ok(())
    }
}
