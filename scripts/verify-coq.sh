#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v coqc >/dev/null 2>&1; then
    printf '%s\n' "Coq verification unavailable: coqc is not installed" >&2
    exit 3
fi

OUTPUT_DIR="target/formal/coq"
mkdir -p "$OUTPUT_DIR"
printf '%s\n' "Coq proof: compiling reference model"
coqc -Q proofs/coq AdaBasedRust -output-dir "$OUTPUT_DIR" proofs/coq/Backend.v
printf '%s\n' "Coq verification complete"