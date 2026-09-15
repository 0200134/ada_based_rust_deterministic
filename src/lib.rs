#![deny(unsafe_code)]

pub mod backend;
pub mod cfg;
pub mod error;
pub mod formal;
pub mod frontend;
pub mod log;
pub mod runtime;
pub mod vm;

pub use backend::{
    BackendError, Function, Instruction, IntegerWidth, Module, OptimizationLevel, ValueId,
};
pub use cfg::{BasicBlock, BlockId, CfgError, CfgFunction, Terminator};
pub use error::{ErrorCode, RuntimeError};
pub use formal::{
    parse_expression, verify_normalization, verify_reference_suite, FormalError, ProofExpr,
};
pub use frontend::{compile_expression, FrontendError};
pub use log::{LogRecord, Logger};
pub use runtime::{Runtime, ShutdownStatus};
pub use vm::{OpCode, Program, VmError};
