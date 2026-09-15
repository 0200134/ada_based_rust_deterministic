# Rust-native LLVM Rewrite

This project is a staged rewrite of LLVM components in Rust. It must not claim
to be a complete LLVM replacement until the compatibility and verification
gates below are met.

## Components

1. **IR core**: types, constants, instructions, modules, functions, basic
   blocks, CFG edges, and SSA validation. The current implementation now has
   verified basic blocks, jumps, conditional branches, and returns.
2. **Text and bitcode formats**: deterministic parser and serializer with
   round-trip tests against a documented compatibility subset.
3. **Optimizer**: independently verifiable passes with explicit pass ordering,
   starting with constant folding and dead-code elimination.
4. **Target-independent lowering**: calling conventions, data layout, and a
   legalizable machine-independent instruction form.
5. **Target backends**: one target at a time, beginning with a documented
   architecture and verified instruction-selection rules.
6. **Register allocation and emission**: deterministic allocation, assembler,
   object files, relocations, and debug metadata.
7. **Execution tooling**: linker integration, JIT, diagnostics, and command
   line compatibility where required.

## Current implementation

`src/backend.rs` contains integer values, constants, arithmetic, returns,
verification, deterministic text emission, and checked constant folding.
`src/cfg.rs` adds verified basic blocks and control-flow terminators.
`src/frontend.rs` parses the matching arithmetic subset with operator
precedence and parentheses. It intentionally does not use LLVM bindings.

`spark/ada_based_ir.ads` and `.adb` provide the Ada/SPARK side of the
LLVM-like tooling: bounded instructions, stable opcode names, append
postconditions, and a proof-visible program bound. Rust remains the bootstrap
driver; Ada/SPARK is the contract and verification boundary.

The repository also has a Cargo-independent Rust bootstrap at
`scripts/bootstrap-pure-rust.sh`. It uses only a seed `rustc`; the compiler
backend itself does not depend on LLVM libraries.

## Completion gates

Each component must have deterministic unit tests, negative verification tests,
and differential tests against the corresponding LLVM behavior before the
next component is considered complete. SPARK contracts cover the safety-
critical state machines; Rust tests cover parsing, transformation, and output
compatibility.