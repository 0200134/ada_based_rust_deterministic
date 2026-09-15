#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v clang >/dev/null 2>&1; then
    printf '%s\n' "LLVM compatibility unavailable: clang is not installed" >&2
    exit 3
fi

OUTPUT_DIR="target/llvm-verify"
mkdir -p "$OUTPUT_DIR"
IR_FILE="$OUTPUT_DIR/expression.ll"
OBJECT_FILE="$OUTPUT_DIR/expression.o"

printf '%s\n' "Rust backend: emitting LLVM IR"
cargo run --locked --bin rust-emit-ir -- --expr '2 + 3 * (5 - 1)' > "$IR_FILE"

printf '%s\n' "LLVM compatibility: clang parsing and compiling emitted IR"
clang -x ir -c "$IR_FILE" -o "$OBJECT_FILE"

printf '%s\n' "LLVM compatibility check passed: $OBJECT_FILE"
