# laterite-ags4-validator

A clean-room Rust validator for the AGS4 geotechnical transfer format —
the same capability [python-ags4](https://gitlab.com/ags-data-format-wg/ags-python-library)'s
`AGS4.check_file()` provides, with no Python at runtime.

**Status: V0–V8 + dictionary auto-selection shipped.** Parsing, the
five bundled standard dictionaries (build-time `phf` codegen, zero
startup cost) with per-file `TRAN_AGS` edition auto-selection, the
public API (`check_file` / `check_file_with_dict` / `is_valid` /
`CheckOptions`), the `lat` CLI, and the full numbered-rule set
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

Currently v0.9.0 — the engine crates move in lockstep on the workspace version.
<!-- END GENERATED: availability -->

## Licence & clean-room boundary (important)

This crate is **MIT** and is **not** derived from python-ags4 (LGPL-3.0).
The AGS4 numbered rules are a functional standard (not copyrightable);
python-ags4 source may be read to understand *which* rules exist and
*how* it interprets ambiguous spec wording, but its code — structure,
control flow, algorithms — must never be copied. Each `src/rules/*`
module carries a clean-room header. Primary source is the AGS4.1 spec
PDF (`reports/AGS 4_1.pdf`), not code.

The bundled standard dictionaries under `data/` are ©AGS reference
data, redistributed as a documented repository-owner decision — see
[`data/PROVENANCE.md`](data/PROVENANCE.md).

For the interim Python-based workflow, see

## Observations log

Discrepancies, ambiguities, and apparent defects found in python-ags4
or the AGS4 spec while porting are recorded in
[`OBSERVATIONS.md`](https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md) — both to justify our deliberate
deviations and to feed upstream issue reports to the AGS Data Format
Working Group. Every phase appends to it.

## The rule set

All 19 numbered rules plus the cross-file Rule 20 are implemented and shipped;
the table is a reference to what each one covers, not a plan. They live in
`src/rules/` (line_format, structure, naming, dictionary, typed_values,
relational, groups, references), dispatched from `src/lib.rs`. The rule text is
AGS4.1 §4.1.1 (Rules 1–18) and §4.1.4 (Rule 20).

| Rule | Family | Notes |
|---|---|---|
| 1 | Encoding / character set | ASCII (0–127) + extended (160–255). Encoding param defaults to UTF-8; cp1252 should be configurable. |
| 2 | Group structure | GROUP / HEADING / UNIT / TYPE / DATA rows in order; blank line between groups. |
| 2b | UNIT-immediately-above-TYPE | Sub-rule of 2; common to trip independently. |
| 3 | Tag column 1 | Must be a valid descriptor. |
| 4 | Quoting | Every cell `"..."` quoted; embedded `"` doubled to `""`. |
| 5 | Field count | Consistent across rows of a group. |
| 6 | Separator + line endings | Comma + CRLF. |
| 7 | Heading order | Spec-mandated dictionary order; custom headings after standard (Rule 9). |
| 8 | Typed values | DT / 2DP / YN / PA / PT etc. match TYPE + UNIT declarations. |
| 9 | Custom headings | Vendor extensions must come after standard headings. |
| 10a–c | KEYs + parent refs | Non-null KEYs; unique within group; child rows resolve to parent rows. |
| 11 | Record Links | `RL` values use `GROUP\|key1\|...` and resolve. |
| 12 | DICT cross-checks | Headings declared in DICT match data. |
| 13 | PROJ singleton | Exactly 1 row. |
| 14 | TRAN required | Transfer metadata group present. |
| 15 | UNIT defs | Every used unit string declared in UNIT group. |
| 16 | ABBR defs | Every PA / PT value declared in ABBR group. |
| 16b | ABBR completeness | Sub-rule: ABBR_HDNG covered for every PA/PT heading. |
| 17 | DICT entries | Non-standard headings have DICT rows. |
| 18 | FILE refs | `FILE_FSET` references resolve in FILE group. |
| 19 / 19a / 19b | Multi-group consistency | Dictionary alignment across groups in one file. |
| 20 | Cross-file | Same headings/types/units across multiple deliveries. |

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
