#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' "bootstrap error: cargo is required" >&2
    exit 2
fi

if ! command -v rustc >/dev/null 2>&1; then
    printf '%s\n' "bootstrap error: rustc is required" >&2
    exit 2
fi

export CARGO_INCREMENTAL=0
export SOURCE_DATE_EPOCH=0

printf '%s\n' "bootstrap stage 0: test and build with the host Rust toolchain"
cargo test --locked
cargo build --release --locked

STAGE0="target/bootstrap/stage0/ada-based-rust"
STAGE1_TARGET="target/bootstrap/stage1"
mkdir -p "$(dirname -- "$STAGE0")"
cp "target/release/ada-based-rust" "$STAGE0"

printf '%s\n' "bootstrap stage 1: rebuild from the same source in an isolated target"
CARGO_TARGET_DIR="$STAGE1_TARGET" cargo build --release --locked

STAGE1="$STAGE1_TARGET/release/ada-based-rust"
printf '%s\n' "bootstrap check: execute stage 1 and synchronize its log"
"$STAGE1" --log-path "$STAGE1_TARGET/bootstrap.log"

printf '%s\n' "bootstrap formal check: execute stage-1 Rust proof checker"
CARGO_TARGET_DIR="$STAGE1_TARGET" cargo run --release --locked \
    --bin rust-formal-check -- --expr '2 + 3 * (5 - 1)'

if command -v clang >/dev/null 2>&1; then
    printf '%s\n' "bootstrap LLVM check: clang validates emitted IR"
    ./scripts/verify-llvm.sh
else
    printf '%s\n' "bootstrap note: clang unavailable; LLVM check skipped"
fi

if command -v gnatprove >/dev/null 2>&1 && command -v gprbuild >/dev/null 2>&1; then
    printf '%s\n' "bootstrap formal verification: running SPARK proof"
    ./scripts/verify-spark.sh
elif [ "${VERIFY_SPARK:-auto}" = "required" ]; then
    printf '%s\n' "bootstrap error: SPARK tools are required but unavailable" >&2
    exit 3
else
    printf '%s\n' "bootstrap note: SPARK tools unavailable; set VERIFY_SPARK=required to enforce proof"
fi

printf '%s\n' "bootstrap complete: $STAGE0 -> $STAGE1"