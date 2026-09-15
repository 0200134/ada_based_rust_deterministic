#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' "bootstrap error: cargo is required" >&2
    exit 2
fi

export CARGO_INCREMENTAL=0
export SOURCE_DATE_EPOCH=0

STAGE0_TARGET="target/bootstrap/cargo/stage0"
STAGE1_TARGET="target/bootstrap/cargo/stage1"

printf '%s\n' "Cargo bootstrap stage 0: locked tests and release build"
cargo test --locked
CARGO_TARGET_DIR="$STAGE0_TARGET" cargo build --release --locked

printf '%s\n' "Cargo bootstrap stage 1: isolated locked rebuild"
CARGO_TARGET_DIR="$STAGE1_TARGET" cargo build --release --locked

printf '%s\n' "Cargo bootstrap check: execute stage 1"
"$STAGE1_TARGET/release/ada-based-rust" \
    --log-path "$STAGE1_TARGET/bootstrap.log"

printf '%s\n' "Cargo bootstrap formal check: execute stage-1 verifier"
CARGO_TARGET_DIR="$STAGE1_TARGET" cargo run --release --locked \
    --bin rust-formal-check -- --expr '2 + 3 * (5 - 1)'

if command -v clang >/dev/null 2>&1; then
    printf '%s\n' "Cargo bootstrap LLVM check: clang validates emitted IR"
    ./scripts/verify-llvm.sh
else
    printf '%s\n' "Cargo bootstrap note: clang unavailable; LLVM check skipped"
fi

printf '%s\n' "Cargo bootstrap Rust-native execution check"
./scripts/verify-rust-native.sh

printf '%s\n' "Cargo bootstrap complete: $STAGE0_TARGET -> $STAGE1_TARGET"