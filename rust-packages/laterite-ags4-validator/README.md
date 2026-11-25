# laterite-ags4-validator

A clean-room Rust validator for the AGS4 geotechnical transfer format —
the same capability [python-ags4](https://gitlab.com/ags-data-format-wg/ags-python-library)'s
`AGS4.check_file()` provides, with no Python at runtime.

**Status: V0–V8 + dictionary auto-selection shipped.** Parsing, the
five bundled standard dictionaries (build-time `phf` codegen, zero
startup cost) with per-file `TRAN_AGS` edition auto-selection, the
public API (`check_file` / `check_file_with_dict` / `verdict::Verdict` /
`is_clean` / `CheckOptions`), the `lat` CLI, and the full numbered-rule set
(Rules 1–20) are implemented, regression-tested, and dogfooded
against real-world deliveries + cross-checked vs `python-ags4`.
Deliberate divergences from python are logged in
[`OBSERVATIONS.md`](https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md).
For end-user CLI usage run `lat --readme`, or see
[`README-cli.md`](https://github.com/niko86/laterite/blob/main/rust-packages/laterite-cli/README-cli.md).

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> **Engine crate, not a door.** `laterite-ags4-validator` is machinery inside the laterite
> toolchain, reshaped whenever the toolchain needs it. The Rust door is
> [`laterite`](https://crates.io/crates/laterite); depend on this one directly
> only if that suits you, and expect it to move.

## Install it

```bash
cargo add laterite-ags4-validator
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->

## Licence & clean-room boundary (important)

This crate is **MIT** and is **not** derived from python-ags4 (LGPL-3.0).
The AGS4 numbered rules are a functional standard (not copyrightable);
python-ags4 source may be read to understand *which* rules exist and
*how* it interprets ambiguous spec wording, but its code — structure,
control flow, algorithms — must never be copied. Each `src/rules/*`
module carries a clean-room header. Primary source is the published
AGS 4.1 specification, not code.

The bundled standard dictionaries under `data/` are ©AGS reference
data, redistributed as a documented repository-owner decision — see
[`data/PROVENANCE.md`](data/PROVENANCE.md).

## Observations log

Discrepancies, ambiguities, and apparent defects found in python-ags4
or the AGS4 spec while porting are recorded in
[`OBSERVATIONS.md`](https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md) — both to justify our deliberate
deviations and to feed upstream issue reports to the AGS Data Format
Working Group. Every phase appends to it.

## The rule set

The engine reports findings under the rule ids below, all implemented and
shipped. This is the engine's own catalogue of checks, not a restatement of the
specification: the specification's Rules 12, 16a and 18a have no entry of their
own here. The checks live in `src/rules/` (line_format, structure, naming,
dictionary, typed_values, relational, groups, references), dispatched from
`src/lib.rs`. The rule text is the AGS4 specification's CSV file rules
(§4.1.1).

`lat rules` prints the same catalogue with each rule's severity and whether it
is fixable; `lat rules --json` adds a description of what each one checks.

| Rule | Covers |
|---|---|
| 1 | Character Set |
| 2 | Data Groups and Data Rows |
| 2a | Line Termination |
| 2b | Group Header Structure |
| 3 | Data Descriptors |
| 4 | Field Count and GROUP Row Format |
| 5 | Quoted Fields |
| 6 | Embedded Carriage Returns |
| 7 | Heading Order and Duplicates |
| 8 | Typed Values |
| 9 | Dictionary Headings |
| 10a | KEY Field Uniqueness |
| 10b | REQUIRED Field Non-Emptiness |
| 10c | Parent-Child Linkage |
| 11a | Record-Link Delimiter |
| 11b | Record-Link Concatenator |
| 11c | Record-Link Resolution |
| 13 | PROJ Group Presence and Row Count |
| 14 | TRAN Group Presence and Row Count |
| 15 | Unit Definitions |
| 16 | Abbreviation Definitions |
| 17 | Data Type Definitions |
| 18 | DICT Group for Non-Standard Names |
| 19 | GROUP Name Format |
| 19a | HEADING Name Length and Character Set |
| 19b | HEADING Name Structure and Cross-Group Borrowing |
| 20 | File Attachments |

## Test corpus

The python-ags4 fixtures at
<https://gitlab.com/ags-data-format-wg/ags-python-library/-/tree/main/tests/test_files>
are the regression suite: each `4.1-rule<N>-<variant>.ags` has a matching
`.check` JSON capturing the expected `check_file` output. Parity against them is
not an aspiration but a **required merge check** — [`check_parity.py`](https://github.com/niko86/laterite/blob/main/tools/check_parity.py)
enforces the failing set by identity against
[`parity-known-failures.json`](https://github.com/niko86/laterite/blob/main/parity-known-failures.json), so a regression
and an intentional divergence are told apart rather than counted. The `"Rule N"`
vs `"AGS Format Rule N"` key-prefix difference is a deliberate, recorded variance.
