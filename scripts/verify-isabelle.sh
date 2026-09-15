#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v isabelle >/dev/null 2>&1; then
    printf '%s\n' "Isabelle verification unavailable: isabelle is not installed" >&2
    exit 3
fi

printf '%s\n' "Isabelle proof: building model with Sledgehammer obligations"
isabelle build -D proofs/isabelle Ada_Based_Rust
printf '%s\n' "Isabelle/Sledgehammer verification complete"