# sel4_cspace Verification Workspace

This workspace now follows the `ostd` crate organization more closely.

## Layout

- `code/sel4_cspace`
  - The main CSpace crate.
  - Contains the copied implementation in `src/`.
  - Contains the current verification model in `specs/`.
- `proofs/sel4_cspace`
  - Optional scratch area for bridge lemmas and early proof experiments.
  - Not part of the main workspace for now.
- `l4v-master`
  - Reference material from the seL4/l4v proof stack.
- `vostd-main`
  - Local reference project used to mirror Verus-oriented organization.

## Intended Direction

The near-term goal is to make `code/sel4_cspace` look like `ostd`:

1. `src/` remains the implementation-facing side.
2. `specs/` becomes the in-crate home of abstract models and contracts.
3. Once the Verus toolchain is wired in, proofs should gradually move from
   pure spec scaffolding toward executable verified code that can live close to
   the implementation.
