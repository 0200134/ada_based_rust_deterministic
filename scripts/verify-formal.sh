#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

required=${FORMAL_VERIFICATION:-auto}
missing=0

./scripts/verify-rust-formal.sh

run_prover() {
    name=$1
    command_name=$2
    script=$3
    if command -v "$command_name" >/dev/null 2>&1; then
        "$ROOT_DIR/scripts/$script"
    else
        printf '%s\n' "$name verification skipped: $command_name is unavailable"
        missing=1
    fi
}

run_prover "SPARK" gnatprove verify-spark.sh
run_prover "Coq" coqc verify-coq.sh
run_prover "Lean" lean verify-lean.sh
run_prover "Isabelle/Sledgehammer" isabelle verify-isabelle.sh

if [ "$required" = "required" ] && [ "$missing" -ne 0 ]; then
    printf '%s\n' "formal verification failed: one or more required provers are unavailable" >&2
    exit 3
fi

if [ "$missing" -ne 0 ]; then
    printf '%s\n' "formal verification incomplete: install missing provers or set FORMAL_VERIFICATION=required"
else
    printf '%s\n' "formal verification complete: Rust checker, SPARK, Coq, Lean, and Isabelle/Sledgehammer"
fi