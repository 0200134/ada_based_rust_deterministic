#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v lean >/dev/null 2>&1; then
    printf '%s\n' "Lean verification unavailable: lean is not installed" >&2
    exit 3
fi

printf '%s\n' "Lean proof: checking reference model"
lean proofs/lean/Backend.lean
printf '%s\n' "Lean verification complete"