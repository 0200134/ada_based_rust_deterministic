#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v rustc >/dev/null 2>&1; then
    printf '%s\n' "bootstrap error: rustc is required as the stage-0 seed" >&2
    exit 2
fi

export SOURCE_DATE_EPOCH=0

compile_stage() {
    stage_dir=$1
    mkdir -p "$stage_dir/lib" "$stage_dir/bin"

    printf '%s\n' "compile library: $stage_dir"
    rustc \
        --crate-name ada_based_rust \
        --crate-type lib \
        --edition 2021 \
        --deny warnings \
        -C opt-level=2 \
        -C panic=abort \
        src/lib.rs \
        --out-dir "$stage_dir/lib"

    printf '%s\n' "compile executable: $stage_dir"
    rustc \
        --edition 2021 \
        --deny warnings \
        -C opt-level=2 \
        -C panic=abort \
        src/main.rs \
        --extern ada_based_rust="$stage_dir/lib/libada_based_rust.rlib" \
        -L dependency="$stage_dir/lib" \
        -o "$stage_dir/bin/ada-based-rust"

    printf '%s\n' "compile formal checker: $stage_dir"
    rustc \
        --edition 2021 \
        --deny warnings \
        -C opt-level=2 \
        -C panic=abort \
        src/bin/rust-formal-check.rs \
        --extern ada_based_rust="$stage_dir/lib/libada_based_rust.rlib" \
        -L dependency="$stage_dir/lib" \
        -o "$stage_dir/bin/rust-formal-check"

    printf '%s\n' "compile LLVM IR emitter: $stage_dir"
    rustc \
        --edition 2021 \
        --deny warnings \
        -C opt-level=2 \
        -C panic=abort \
        src/bin/rust-emit-ir.rs \
        --extern ada_based_rust="$stage_dir/lib/libada_based_rust.rlib" \
        -L dependency="$stage_dir/lib" \
        -o "$stage_dir/bin/rust-emit-ir"

    printf '%s\n' "compile Rust-native VM: $stage_dir"
    rustc \
        --edition 2021 \
        --deny warnings \
        -C opt-level=2 \
        -C panic=abort \
        src/bin/rust-run.rs \
        --extern ada_based_rust="$stage_dir/lib/libada_based_rust.rlib" \
        -L dependency="$stage_dir/lib" \
        -o "$stage_dir/bin/rust-run"
}

STAGE0="target/bootstrap/pure-rust/stage0"
STAGE1="target/bootstrap/pure-rust/stage1"

printf '%s\n' "pure-Rust bootstrap stage 0"
compile_stage "$STAGE0"

printf '%s\n' "pure-Rust bootstrap stage 1"
compile_stage "$STAGE1"

printf '%s\n' "pure-Rust bootstrap check"
"$STAGE1/bin/ada-based-rust" \
    --log-path "$STAGE1/bootstrap.log"

printf '%s\n' "pure-Rust formal check"
"$STAGE1/bin/rust-formal-check" --expr '2 + 3 * (5 - 1)'

printf '%s\n' "pure-Rust native execution check"
result=$("$STAGE1/bin/rust-run" --expr '2 + 3 * (5 - 1)')
if [ "$result" != "14" ]; then
    printf '%s\n' "pure-Rust error: unexpected VM result: $result" >&2
    exit 1
fi

if command -v clang >/dev/null 2>&1; then
    printf '%s\n' "pure-Rust LLVM check"
    "$STAGE1/bin/rust-emit-ir" --expr '2 + 3 * (5 - 1)' \
        > "$STAGE1/expression.ll"
    clang -x ir -c "$STAGE1/expression.ll" -o "$STAGE1/expression.o"
else
    printf '%s\n' "pure-Rust note: clang unavailable; LLVM check skipped"
fi

printf '%s\n' "pure-Rust bootstrap complete: $STAGE0 -> $STAGE1"