# Cross-surface parity

Five surfaces run "the same" AGS4 validator. That's a strong claim — so it's
**tested**, not asserted.

## One engine, many doors

Python, Node, the browser (wasm), DuckDB, and the `lat` CLI don't each
re-implement the AGS4 rules. They're thin bindings over **one** clean-room Rust
core. A finding you get in Python is produced by the exact code that produces it
in the browser — the only thing that differs is the door.

## How it's proven

A **compliance harness** turns "the same engine" into a checkable property. Over a
real corpus of AGS4 files, it runs every read surface and reduces each to the same
currency — the set of `"AGS Format Rule N"` labels that fired — then asserts:

1. **The laterite surfaces agree exactly.** rust, python-laterite, node, wasm, and
   the DuckDB extension must report a **byte-identical** rule floor on every file.
   A mismatch is a binding or serialization bug, and it fails the check.
2. **python-ags4 agrees, modulo the documented divergences.** The incumbent
   `python-ags4` is compared too; the handful of deliberate, catalogued
   differences reconcile, and only an _unexplained_ difference is flagged.

This runs two ways: a **per-PR gate** (the four in-repo surfaces, on every change
that could move a finding) and a **monthly full-matrix report** (all six,
including the DuckDB extension built from source).

## Why it matters to you

It means you can **choose a surface for its ergonomics, not its correctness**.
Validate in a notebook, in a CI shell, in the browser, or in a SQL query — the
verdict is the same one, and there's a gate standing over the codebase to keep it
that way.

## Related

[The .ags.idx certificate](certificate-lifecycle.md) · the [capability
matrix](../surfaces/index.md#what-each-door-can-do)
