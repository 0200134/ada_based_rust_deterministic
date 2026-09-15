use crate::backend::{Instruction, ValueId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BlockId(u32);

impl BlockId {
    pub const fn new(number: u32) -> Self {
        Self(number)
    }

    pub const fn number(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Terminator {
    Return(ValueId),
    Jump(BlockId),
    Branch {
        condition: ValueId,
        then_block: BlockId,
        else_block: BlockId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    id: BlockId,
    label: String,
    instructions: Vec<Instruction>,
    terminator: Option<Terminator>,
}

impl BasicBlock {
    pub fn new(id: BlockId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            instructions: Vec::new(),
            terminator: None,
        }
    }

    pub fn id(&self) -> BlockId {
        self.id
    }

    pub fn push(&mut self, instruction: Instruction) -> Result<(), CfgError> {
        if self.terminator.is_some() {
            return Err(CfgError::InstructionAfterTerminator(self.id));
        }
        self.instructions.push(instruction);
        Ok(())
    }

    pub fn terminate(&mut self, terminator: Terminator) -> Result<(), CfgError> {
        if self.terminator.is_some() {
            return Err(CfgError::DuplicateTerminator(self.id));
        }
        self.terminator = Some(terminator);
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CfgFunction {
    name: String,
    blocks: Vec<BasicBlock>,
}

impl CfgFunction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            blocks: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: BasicBlock) -> Result<(), CfgError> {
        if self.blocks.iter().any(|existing| existing.id == block.id) {
            return Err(CfgError::DuplicateBlock(block.id));
        }
        self.blocks.push(block);
        Ok(())
    }

    pub fn verify(&self) -> Result<(), CfgError> {
        if self.name.is_empty() {
            return Err(CfgError::EmptyFunctionName);
        }
        let mut blocks = BTreeMap::new();
        for block in &self.blocks {
            if block.label.is_empty() {
                return Err(CfgError::EmptyBlockLabel(block.id));
            }
            if blocks.insert(block.id, block).is_some() {
                return Err(CfgError::DuplicateBlock(block.id));
            }
        }
        if self.blocks.is_empty() {
            return Err(CfgError::NoBlocks);
        }

        let mut values = BTreeSet::new();
        for block in &self.blocks {
            for instruction in &block.instructions {
                for operand in instruction_operands(instruction) {
                    if !values.contains(&operand) {
                        return Err(CfgError::UnknownValue(operand));
                    }
                }
                if let Some(result) = instruction_result(instruction) {
                    if !values.insert(result) {
                        return Err(CfgError::DuplicateValue(result));
                    }
                }
            }
            let Some(terminator) = &block.terminator else {
                return Err(CfgError::MissingTerminator(block.id));
            };
            match terminator {
                Terminator::Return(value) => {
                    if !values.contains(value) {
                        return Err(CfgError::UnknownValue(*value));
                    }
                }
                Terminator::Jump(target) => ensure_target(*target, &blocks)?,
                Terminator::Branch {
                    condition,
                    then_block,
                    else_block,
                } => {
                    if !values.contains(condition) {
                        return Err(CfgError::UnknownValue(*condition));
                    }
                    ensure_target(*then_block, &blocks)?;
                    ensure_target(*else_block, &blocks)?;
                }
            }
        }
        Ok(())
    }

    pub fn emit_text(&self) -> Result<String, CfgError> {
        self.verify()?;
        let mut output = String::new();
        output.push_str("define @");
        output.push_str(&self.name);
        output.push_str("() {\n");
        for block in &self.blocks {
            output.push_str(&block.label);
            output.push_str(":\n");
            for instruction in &block.instructions {
                output.push_str("  ");
                output.push_str(&format_instruction(instruction));
                output.push('\n');
            }
            output.push_str("  ");
            output.push_str(&format_terminator(
                block
                    .terminator
                    .as_ref()
                    .ok_or(CfgError::MissingTerminator(block.id))?,
            ));
            output.push('\n');
        }
        output.push_str("}\n");
        Ok(output)
    }
}

fn ensure_target(target: BlockId, blocks: &BTreeMap<BlockId, &BasicBlock>) -> Result<(), CfgError> {
    if blocks.contains_key(&target) {
        Ok(())
    } else {
        Err(CfgError::UnknownBlock(target))
    }
}

fn instruction_result(instruction: &Instruction) -> Option<ValueId> {
    match instruction {
        Instruction::Constant { result, .. }
        | Instruction::Add { result, .. }
        | Instruction::Sub { result, .. }
        | Instruction::Mul { result, .. }
        | Instruction::Div { result, .. }
        | Instruction::Rem { result, .. }
        | Instruction::Neg { result, .. } => Some(*result),
        Instruction::Eq { result, .. } | Instruction::Lt { result, .. } => Some(*result),
        Instruction::Select { result, .. } => Some(*result),
        Instruction::Return { .. } => None,
    }
}

fn instruction_operands(instruction: &Instruction) -> Vec<ValueId> {
    match instruction {
        Instruction::Constant { .. } => Vec::new(),
        Instruction::Add { left, right, .. }
        | Instruction::Sub { left, right, .. }
        | Instruction::Mul { left, right, .. }
        | Instruction::Div { left, right, .. }
        | Instruction::Rem { left, right, .. } => vec![*left, *right],
        Instruction::Neg { operand, .. } | Instruction::Return { value: operand } => vec![*operand],
        Instruction::Eq { left, right, .. } | Instruction::Lt { left, right, .. } => {
            vec![*left, *right]
        }
        Instruction::Select {
            condition,
            then_value,
            else_value,
            ..
        } => vec![*condition, *then_value, *else_value],
    }
}

fn format_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Constant { result, value } => {
            format!("%{} = const {}", result.number(), value)
        }
        Instruction::Add {
            result,
            left,
            right,
        } => format!(
            "%{} = add %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Sub {
            result,
            left,
            right,
        } => format!(
            "%{} = sub %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Mul {
            result,
            left,
            right,
        } => format!(
            "%{} = mul %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Div {
            result,
            left,
            right,
        } => format!(
            "%{} = div %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Rem {
            result,
            left,
            right,
        } => format!(
            "%{} = rem %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Neg { result, operand } => {
            format!("%{} = neg %{}", result.number(), operand.number())
        }
        Instruction::Eq {
            result,
            left,
            right,
        } => format!(
            "%{} = eq %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Lt {
            result,
            left,
            right,
        } => format!(
            "%{} = lt %{}, %{}",
            result.number(),
            left.number(),
            right.number()
        ),
        Instruction::Select {
            result,
            condition,
            then_value,
            else_value,
        } => format!(
            "%{} = select %{}, %{}, %{}",
            result.number(),
            condition.number(),
            then_value.number(),
            else_value.number()
        ),
        Instruction::Return { value } => format!("return %{}", value.number()),
    }
}

fn format_terminator(terminator: &Terminator) -> String {
    match terminator {
        Terminator::Return(value) => format!("return %{}", value.number()),
        Terminator::Jump(target) => format!("jump block{}", target.number()),
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => format!(
            "branch %{}, block{}, block{}",
            condition.number(),
            then_block.number(),
            else_block.number()
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CfgError {
    EmptyFunctionName,
    NoBlocks,
    DuplicateBlock(BlockId),
    EmptyBlockLabel(BlockId),
    MissingTerminator(BlockId),
    DuplicateTerminator(BlockId),
    InstructionAfterTerminator(BlockId),
    UnknownBlock(BlockId),
    UnknownValue(ValueId),
    DuplicateValue(ValueId),
}

impl fmt::Display for CfgError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid CFG: {self:?}")
    }
}

impl std::error::Error for CfgError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_and_emits_branching_function() -> Result<(), CfgError> {
        let mut entry = BasicBlock::new(BlockId::new(0), "entry");
        entry.push(Instruction::Constant {
            result: ValueId::new(0),
            value: 1,
        })?;
        entry.terminate(Terminator::Branch {
            condition: ValueId::new(0),
            then_block: BlockId::new(1),
            else_block: BlockId::new(2),
        })?;

        let mut then_block = BasicBlock::new(BlockId::new(1), "then");
        then_block.terminate(Terminator::Return(ValueId::new(0)))?;
        let mut else_block = BasicBlock::new(BlockId::new(2), "else");
        else_block.terminate(Terminator::Return(ValueId::new(0)))?;

        let mut function = CfgFunction::new("branching");
        function.add_block(entry)?;
        function.add_block(then_block)?;
        function.add_block(else_block)?;
        let text = function.emit_text()?;
        assert!(text.contains("branch %0, block1, block2"));
        Ok(())
    }

    #[test]
    fn rejects_missing_branch_target() -> Result<(), CfgError> {
        let mut entry = BasicBlock::new(BlockId::new(0), "entry");
        entry.push(Instruction::Constant {
            result: ValueId::new(0),
            value: 1,
        })?;
        entry.terminate(Terminator::Jump(BlockId::new(9)))?;
        let mut function = CfgFunction::new("broken");
        function.add_block(entry)?;
        assert_eq!(
            function.verify(),
            Err(CfgError::UnknownBlock(BlockId::new(9)))
        );
        Ok(())
    }
}
