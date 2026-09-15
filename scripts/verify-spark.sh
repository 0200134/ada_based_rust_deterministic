#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if command -v gnatprove >/dev/null 2>&1; then
    PROVE_CMD="gnatprove"
elif command -v alr >/dev/null 2>&1 && alr exec -- gnatprove --version >/dev/null 2>&1; then
    PROVE_CMD="alr exec -- gnatprove"
else
    printf '%s\n' "formal verification unavailable: run ./scripts/setup-spark.sh" >&2
    exit 3
fi

if command -v gprbuild >/dev/null 2>&1; then
    BUILD_CMD="gprbuild"
elif command -v alr >/dev/null 2>&1 && alr exec -- gprbuild --version >/dev/null 2>&1; then
    BUILD_CMD="alr exec -- gprbuild"
else
    printf '%s\n' "formal verification unavailable: GPRbuild is missing; run ./scripts/setup-spark.sh" >&2
    exit 3
fi

printf '%s\n' "SPARK build: checking Ada project"
$BUILD_CMD -P spark/ada_based_runtime_verification.gpr -q

printf '%s\n' "SPARK proof: running runtime and Ada IR contracts at proof level 2"
$PROVE_CMD -P spark/ada_based_runtime_verification.gpr --level=2 --checks-as-errors=on

printf '%s\n' "SPARK formal verification complete"