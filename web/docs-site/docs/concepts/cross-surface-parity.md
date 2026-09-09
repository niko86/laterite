# Cross-surface parity

Four surfaces run "the same" AGS4 validator. That's a strong claim, so it's
**tested**, not asserted.

## The premise

[One engine, many doors](one-engine-many-doors.md): the surfaces are thin
bindings over one Rust core, so a finding you get in Python is produced by the
exact code that produces it in the browser.

The DuckDB extension is the read-only exception: it runs no rules, so it has no
findings to compare. Its agreement is proven on what it actually does: that its
`read_ags` produces the same content-addressed key set as the core reader.

## How it's proven

A **compliance harness** turns "the same engine" into a checkable property. Over a
real corpus of AGS4 files, it runs every read surface and reduces each to the same
currency (the set of `"AGS Format Rule N"` labels that fired), then asserts:

1. **The laterite surfaces agree exactly.** rust, python-laterite, node and wasm
   must report a **byte-identical** rule floor on every file. A mismatch is a
   binding or serialization bug, and it fails the check.
2. **python-ags4 agrees, modulo the documented divergences.** The incumbent
   `python-ags4` is compared too; the handful of deliberate, catalogued
   differences reconcile, and only an _unexplained_ difference is flagged.

This runs two ways: a **per-PR gate** (the four in-repo surfaces, on every change
that could move a finding) and a **full-matrix report** (all six, including the
DuckDB extension built from source), which is run on-demand.
<!-- cadence: compliance -->
<!-- cadence: compliance-report -->

## Why it matters to you

It means you can **choose a surface for its ergonomics, not its correctness**.
Validate in a notebook, in a CI shell, in the browser, or in a SQL query. The
verdict is the same one, and there's a gate standing over the codebase to keep it
that way.

## Related

[The .ags.idx certificate](certificate-lifecycle.md) · the [capability
matrix](../surfaces/index.md#what-each-door-can-do)
