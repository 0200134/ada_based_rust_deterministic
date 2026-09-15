#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

expression='if 1 < 2 then 7 else 9'
output=$(cargo run --locked --bin rust-native-pipeline -- --expr "$expression")
printf '%s\n' "$output"
printf '%s\n' "$output" | grep -F 'formal: passed' >/dev/null
printf '%s\n' "$output" | grep -F 'ir: verified' >/dev/null
printf '%s\n' "$output" | grep -F 'result: 7' >/dev/null
printf '%s\n' 'Rust-native pipeline: passed'
