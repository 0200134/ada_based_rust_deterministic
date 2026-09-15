#!/bin/sh

set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT_DIR"

if ! command -v alr >/dev/null 2>&1; then
    printf '%s\n' "SPARK setup error: Alire (alr) is required" >&2
    exit 2
fi

printf '%s\n' "SPARK setup: selecting GNAT and GPRbuild through Alire"
alr toolchain --select --local gnat_native gprbuild

printf '%s\n' "SPARK setup: resolving GNATprove and SPARK libraries"
alr update
alr with gnatprove=16.1.0 sparklib=16.1.0

printf '%s\n' "SPARK setup complete"
printf '%s\n' "Run verification with: alr exec -- ./scripts/verify-spark.sh"
