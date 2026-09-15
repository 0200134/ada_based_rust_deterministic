#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

printf '%s\n' "Rust-native execution check"
result=$(cargo run --locked --bin rust-run -- --expr '2 + 3 * (5 - 1)')
if [ "$result" != "14" ]; then
    printf '%s\n' "unexpected result: $result" >&2
    exit 1
fi
printf '%s\n' "Rust-native execution passed: $result"