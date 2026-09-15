# Formal Models

These files are reference models for the deterministic expression semantics
and constant-folding transformation used by the Rust frontend/backend:

- `coq/Backend.v`: Coq model
- `lean/Backend.lean`: Lean model
- `isabelle/Backend.thy`: Isabelle/HOL model with Sledgehammer proof methods
- `../src/formal.rs`: executable Rust bounded checker for the same semantics

Each model defines literal, add, subtract, and multiply expressions, evaluation,
and a fold operation, then proves that folding preserves evaluation. They are intentionally small
and dependency-light. They do not yet constitute a machine-checked refinement
proof of Rust source code. That requires a specified extraction or translation
boundary and a proof that the Rust IR and runtime preserve these models.

Run all available provers with:

```text
./scripts/verify-formal.sh
```

Require every prover to be installed and successful with:

```text
FORMAL_VERIFICATION=required ./scripts/verify-formal.sh
```