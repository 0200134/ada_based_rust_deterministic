# Ada-based Rust Compiler Manual

## 1. What This Project Is

Ada-based Rust is currently a deterministic Rust compiler foundation. Its implemented compiler path accepts a small integer-expression language, lowers it to a verified SSA-like intermediate representation, performs checked constant folding, and emits deterministic LLVM-like text.

The project also contains a runtime error/logging foundation, a Rust-native bounded formal checker, and a verified control-flow IR.

It is not yet a complete Rust compiler, LLVM replacement, linker, assembler, JIT, or self-hosting compiler.

## 2. Requirements

For normal Cargo builds:

- Rust 1.78 or newer
- Cargo

For direct Rust bootstrap:

- `rustc` as the stage-0 seed compiler

Optional formal tools:

- GNAT/SPARK: `gnatprove`, `gprbuild`
- Coq: `coqc`
- Lean: `lean`
- Isabelle/HOL: `isabelle`

Provision GNATprove and the SPARK libraries through Alire:

```sh
./scripts/setup-spark.sh
alr exec -- ./scripts/verify-spark.sh
```

Missing optional provers are reported as unavailable. They are never reported as passing.

## 3. Build and Test

Run the standard Rust checks:

```sh
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Run the compiler runtime:

```sh
cargo run -- --log-path target/ada-based-rust.log
```

Command-line help and equivalent log-path forms are also available:

```sh
cargo run -- --help
cargo run -- --log-path=target/ada-based-rust.log
```

The current executable performs a runtime bootstrap operation and shuts down explicitly. It does not yet accept source files as command-line input.

When no path is supplied, the runtime searches upward from the current
directory for `Cargo.toml` and writes to that project root's
`target/ada-based-rust.log`. It prints the synchronized path when it exits.
Every log path is fully qualified before the file is opened, including paths
supplied with `--log-path` and the fallback used outside a Cargo project.

## 4. Source-to-IR Compiler API

The frontend API is:

```rust
ada_based_rust::compile_expression(source)
```

It returns a `Module` on success or `FrontendError` on failure. The generated module contains one function named `main` with an `i64` return type.

Example:

```rust
use ada_based_rust::compile_expression;

let module = compile_expression("2 + 3 * (5 - 1)")?;
let text = module.emit_text()?;
println!("{text}");
```

The implementation pipeline is:

```text
source expression
    -> recursive-descent parser
    -> constants and arithmetic instructions
    -> SSA/use verification
    -> optional constant folding
    -> deterministic LLVM-like text
```

## 5. Supported Language

The supported grammar is:

```text
expression      := comparison | conditional
conditional     := 'if' comparison 'then' comparison 'else' comparison
comparison      := additive (('==' | '<') additive)*
additive        := multiplicative (('+' | '-') multiplicative)*
multiplicative  := unary (('*' | '/' | '%') unary)*
unary           := ('+' | '-') unary | primary
primary         := integer | '(' expression ')'
integer         := one or more ASCII decimal digits
```

Whitespace is accepted between tokens.

Examples that compile:

```text
2 + 3
20 - 3 * 4
2 + 3 * (5 - 1)
  100 * (2 + 7) - 5
if 1 < 2 then 7 else 9
```

Operator precedence is conventional:

1. Parentheses
2. Unary `+` and `-`
3. Multiplication, division, and remainder
4. Addition and subtraction, evaluated left to right
5. Comparisons `==` and `<`, producing `0` or `1`

Not implemented in this frontend:

- identifiers and variables
- comparisons and booleans
- function calls
- statements and control flow
- arrays, pointers, structs, or memory
- source files, modules, imports, or type inference

## 6. Frontend Errors

Malformed source returns a `FrontendError`; it does not panic.

Possible errors include:

- `EmptyInput`: no expression was supplied
- `ExpectedInteger(position)`: an operand is missing
- `InvalidCharacter`: a non-ASCII or unsupported character was found
- `IntegerOverflow(position)`: a decimal literal does not fit in `i64`
- `ExpectedClosingParen(position)`: a parenthesized expression is incomplete
- `TrailingInput(position)`: input remains after a complete expression

## 7. Backend IR

The backend currently defines these instruction forms:

- `Constant { result, value }`
- `Add { result, left, right }`
- `Sub { result, left, right }`
- `Mul { result, left, right }`
- `Div { result, left, right }`
- `Rem { result, left, right }`
- `Neg { result, operand }`
- `Eq { result, left, right }`
- `Lt { result, left, right }`
- `Select { result, condition, then_value, else_value }`
- `Return { value }`

The verifier checks that:

- function names are non-empty
- every operand was defined earlier
- every result value is defined only once
- no instruction follows a return
- every function has a return
- module function names are unique

Invalid IR returns `BackendError` and is not emitted.

For control flow, `CfgFunction` contains `BasicBlock` values with explicit
`Jump`, conditional `Branch`, or `Return` terminators. CFG verification checks
that every block has one terminator, every branch target exists, and values are
defined before use in the deterministic block order. CFG emission produces a
stable block-oriented text form.

Conditional expressions lower to `Select` in the linear expression backend.
Any nonzero condition selects the `then` value; zero selects the `else` value.

## 8. Optimization and Emission

`OptimizationLevel::Basic` folds operations whose operands are known constants. Arithmetic uses checked `i64` operations. Add, subtract, multiply, divide, remainder, and negation overflow return an error instead of wrapping. Division and remainder by zero return an explicit error.

The text emitter produces a deterministic LLVM-like representation, for example:

```text
; ada-based-rust deterministic IR
define i64 @main() {
entry:
  %0 = add i64 2, 0
  %1 = add i64 3, 0
  %2 = mul i64 %1, %0
  ret i64 %2
}
```

This text is an internal stable format inspired by LLVM IR. It is not currently guaranteed to be accepted by LLVM tools and does not produce machine code.

When `clang` is installed, the emitted text can be checked and compiled as
LLVM IR:

```sh
./scripts/verify-llvm.sh
```

The script writes `target/llvm-verify/expression.ll` and the resulting object
file. This is the current LLVM compatibility gate; full LLVM semantic and
target compatibility is not implemented.

The Cargo and direct-`rustc` bootstrap scripts run this gate when `clang` is
available. The direct bootstrap also compiles its own IR emitter and Rust-native
VM in each stage.

Run the complete Rust-native pipeline with:

```sh
./scripts/verify-rust-native-pipeline.sh
```

This performs parsing, formal normalization checking, verified IR construction,
deterministic IR emission, bytecode lowering, and Rust VM execution in one
command.

## 9. Rust Formal Checker

The Rust formal checker models the same arithmetic and comparison expression subset. It:

1. parses an expression
2. evaluates it with checked `i64` arithmetic
3. normalizes it to a constant expression
4. evaluates the normalized expression
5. rejects the result if the two evaluations differ

Run the built-in reference suite:

```sh
./scripts/verify-rust-formal.sh
```

Verify one expression:

```sh
cargo run --bin rust-formal-check -- --expr "2 + 3 * (5 - 1)"
```

The checker rejects overflow, division by zero, and malformed input. It is a bounded executable checker for this language subset, not a general theorem prover.

## 10. Formal Verification Backends

Run all configured verification backends:

```sh
./scripts/verify-formal.sh
```

The repository includes reference models in Coq, Lean, and Isabelle/HOL with Sledgehammer, plus the Ada/SPARK runtime contract. The current models prove arithmetic and constant-folding properties. They do not yet prove a machine-checked refinement from the Rust implementation to each external model.

Require every optional prover to be installed:

```sh
FORMAL_VERIFICATION=required ./scripts/verify-formal.sh
```

## 11. Bootstrap Modes

### Cargo bootstrap

```sh
./scripts/bootstrap-cargo.sh
```

Runs locked tests, builds an isolated stage 0, rebuilds an isolated stage 1, executes stage 1, and runs the stage-1 formal checker.

### Standard bootstrap

```sh
./scripts/bootstrap.sh
```

Runs the host Cargo tests/build, creates stage 0 and stage 1, executes the stage-1 runtime and formal checker, and optionally runs SPARK proof checking.

### Direct-rustc bootstrap

```sh
./scripts/bootstrap-pure-rust.sh
```

Compiles the library, runtime executable, and formal checker directly with `rustc` in two isolated stages. Cargo is not used. A seed `rustc` is still required; this is not yet a from-zero self-hosting compiler bootstrap.

## 12. Runtime Safety and Logging

The runtime uses `Result`-based error handling and `panic = "abort"` profiles. Errors have stable numeric codes. Log records are appended, flushed, and synchronized immediately.

Shutdown is explicit. Repeated shutdown is rejected. Missing log directories are created automatically. A hard process kill or power loss cannot guarantee a final shutdown record, but accepted error records are synchronized before returning success.

## 13. Current Boundary

Implemented now:

- deterministic integer-expression frontend
- verified SSA-like backend IR
- checked basic constant folding
- deterministic LLVM-like text emission
- Rust bounded formal checker
- Cargo and direct-rustc two-stage bootstrap
- optional SPARK, Coq, Lean, and Isabelle/Sledgehammer runners

Not implemented yet:

- complete Rust language
- complete LLVM IR compatibility
- LLVM bitcode compatibility
- optimization pipeline beyond basic constant folding
- target lowering and machine-code generation
- assembler, linker, object files, debug information, or JIT
- complete formal refinement proof of the Rust code
- bootstrap without a seed `rustc`
