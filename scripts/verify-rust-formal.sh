#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

printf '%s\n' "Rust formal checker: running proof-kernel tests"
cargo test --locked formal

printf '%s\n' "Rust formal checker: running reference suite"
cargo run --locked --bin rust-formal-check
cargo run --locked --bin rust-formal-check -- --expr '2 + 3 * (5 - 1)'

printf '%s\n' "Rust formal verification complete"