---
type: insight
title: "A numeric TYPE's count (n in nDP/nSF/nSCI) is an uncapped format width — a crafted \"9999999999SF\" OOMs python-ags4; laterite now clamps to 30"
status: ratified
tags: [insight]
gap_kind: rust-vs-python
severity: high
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-08-typed-values]
proposes_observation: true
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: D
related: [rule-08-typed-values, nDP, nSF, nSCI, O-49, O-50, 0dp-integer-conversion-precision-loss, python-ags4]
sources: [spec-4.2]
---
# The TYPE count is attacker-controlled, and it was a format width

## Claim
> [!variance] The `n` in an AGS4 numeric TYPE (`"3SF"`, `"3DP"`, `"3SCI"`) is
> parsed straight off the file's own TYPE row with **no upper bound**, then fed
> into a format WIDTH wherever a value is rendered to its expected form — Rule
> 8's grammar check, the fixes engine, and the XLSX writer all do this. No
> edition caps it; every real AGS4 numeric TYPE is single-digit (0–6). A
> crafted or corrupt TYPE like `"9999999999SF"` is a valid *parse* and reaches
> the formatter on raw bytes, before any rule has vetted the TYPE — the
> classic "attacker-controlled width" shape.

## Evidence
Probed 2026-07-20 (`ags-wiki/.bootstrap/probes/probe_sf_count_dos.py` →
`ags-wiki/.bootstrap/probes/sf-count-dos.md`):

- **python-ags4** `ext:ags-python-library:python_ags4/AGS4.py::_format_SF`
  (and its DP/SCI siblings): `i = int(TYPE.strip('SF')) - 1 -
  floor(log10|v|)` at arbitrary-precision Python-int width, then
  `f"{v:.{i}f}"`. Run through the *real* upstream function at escalating
  counts, `len(output)` grows linearly and uncapped —
  `10000000SF` → a 10,000,001-char string in 1 ms. The crafted
  `9999999999SF` computes `i ≈ 9,999,999,998` → a **~10 GB** string request →
  MemoryError/DoS.
- **laterite** was *also* vulnerable pre-#610: `repo:rust-packages/laterite-ags4-types/src/lib.rs::format_nsf`
  did `(n as i32)`, which **wraps** `9999999999` to a *positive*
  `1_410_065_407` → a ~1.4 GB requested width → OOM; `format_ndp`/`format_nsci`
  fed a bare `n: usize` straight into `{:.n$}` with the same unbounded read.
  `repo:rust-packages/laterite-ags4-excel/src/lib.rs`'s `NumericFormat::format`
  (`Dp`/`Sci` arms) and `format_sf` shared the identical shape.
- Post-#610: `L.validate(<crafted 9999999999SF file>)` returns in **1 ms**,
  bounded, 9 findings — no OOM, no hang.

## Why it matters
This is a genuine, exploitable DoS reachable from **untrusted file content**
alone — the formatter runs on parsed bytes before Rule 8 has vetted the TYPE,
so a hostile `.ags` file can wedge any caller that renders a value to its
expected form (Rule 8's grammar check, the fixes engine, python-ags4's own
XLSX export path). It is a *shared* latent defect, not a value-divergence —
both engines mis-render the same crafted input — which is why the fix is
bounded-output rather than a semantics change: clamping only bites at counts
> 30, a ceiling no legitimate AGS TYPE reaches (f64 itself carries only ~17
significant digits), so every real value renders byte-identically before and
after.

A **separate, opposite-direction** issue on the same formatter family: the
`0DP`-integer path casts `f as i64`, which **saturates** at `±i64::MAX` where
python-ags4's `f"{float(s):.0f}"` keeps full precision (`1E30` →
`9223372036854775807` vs `1000000000000000019884624838656`). That is a real
value *divergence* (we lose precision, python doesn't), the inverse of this
page's DoS shape — unbounded OUTPUT here, unbounded INPUT into a bounded
`i64` store there. It was hardened separately in #611, range-guarding to
Null instead of saturating; see the sibling [[O-50]] /
[[0dp-integer-conversion-precision-loss]] for the full account — not fixed
by this page's clamp.

## OBSERVATIONS entry — **ratified as [[O-49]]**
> [!note] Written into `repo:OBSERVATIONS.md#o-49` (5-field house style).
> **Our decision** (#610): clamp the count to `MAX_NUMERIC_COUNT = 30` at all
> six sites (`laterite-ags4-types` nDP/nSF/nSCI + `laterite-ags4-excel` Dp/Sci/
> `format_sf`) before it reaches a format width. Regression tests
> `repo:rust-packages/laterite-ags4-types/src/lib.rs::nsf_count_is_clamped_so_a_crafted_type_cannot_dos`
> and
> `repo:rust-packages/laterite-ags4-excel/src/lib.rs::format_sf_count_is_clamped_so_a_crafted_type_cannot_dos`
> assert a crafted/`usize::MAX` count stays bounded *and* that legitimate
> counts render unchanged. **Upstream-reportable: [YES]** — python-ags4's
> `_format_SF`/`_format_DP`/`_format_SCI` OOM on a crafted numeric-TYPE count;
> a malformed/hostile file DoSes any caller that reformats a value (Rule 8
> fixes, XLSX export). Candidate upstream report, not yet filed.

## Related
[[rule-08-typed-values]] · [[nDP]] · [[nSF]] · [[nSCI]] · [[O-49]] · [[O-50]] · [[0dp-integer-conversion-precision-loss]] · [[python-ags4]]
