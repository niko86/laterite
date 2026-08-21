---
type: strategy
title: "Rule 20 on-disk: implemented as opt-in --check-files; O-27 retired"
status: confirmed
tags: [strategy]
targets: [rule-20-file-fset]
divergence_hypothesis: "see body"
probe_files: [ags-wiki/.bootstrap/probes/probe-o27-file-ondisk.ags]
expected_rust: "Rule 20 (with --check-files) — matches python"
expected_python: "Rule 20 (always-on on-disk)"
evidence: "probe-run + implementation — ags-wiki/.bootstrap/probes/RESULTS.md"
related: [rule-20-file-fset, O-27, parity-model, parity-cascade-unreconcilable, cert-trust-v2]
sources: []
---
# Rule 20 on-disk: implemented (opt-in), O-27 retired

## History
> [!divergence] The probe (`probe-o27-file-ondisk.ags`) showed
> `rust=clean` vs `py={Rule 20}` because the Rust validator did only
> the **data-level** Rule 20; python-ags4 also stats the filesystem.
> This was first papered over with a `reconcile()` O-27 arm (treating
> it as a "known divergence"). **The user did not knowingly make that
> scope decision**, so the divergence was *closed by implementing the
> check*, not reconciled away.

## What changed (this session)
> [!note] **CONFIRMED — implemented.**
> 1. Validator: `CheckOptions::check_files` (default **OFF** —
>    path-independent, what a library / `db-to-ags4 --validate`
>    needs). When on, `rule_20_on_disk` asserts the
>    sidecar `FILE/<FILE_FSET>/<FILE_NAME>` tree (`std::fs` only).
>    CLI: `lat validate --check-files`.
>    **(2026-07-14, [[cert-trust-v2]] PR 2)** `rule_20_on_disk` moved out of
>    `rules/references.rs` into a new `src/world.rs`, and a `check_files`
>    request with no source path (any bytes/text-modality caller, wasm
>    always) now **refuses** (`WorldCheckRequiresSourceError`) instead of the
>    prior silent no-op-and-report-clean — a related but distinct bug from
>    this page's opt-in-scope story: this one fires even when the caller
>    genuinely wants the check but has nothing to check it against.
> 2. corpus-qa `validate` enables it **by default** (`--no-check-files`
>    opts out) → Rust matches python-ags4's always-on stat → the
>    dogfood **AGREEs on Rule 20** directly.
> 3. The `reconcile()` O-27 arm + its unit test were **removed** —
>    O-27 is no longer a divergence.
> 4. `db-to-ags4` now reconstructs `FILE/<fset>/<name>` from stored
>    blobs (Rust `attachments.rs` + Python `blobs.py`, JOIN
>    `blob.parent_id = v_file.id`), so an exported delivery passes
>    `lat validate --check-files`.

## Expected vs observed (post-implementation)

| | Rust `lat validate --check-files` | python-ags4 |
|---|---|---|
| no `FILE/` sidecar | Rule 20 (on-disk) | Rule 20 (on-disk) → **AGREE** |
| tree present | clean | clean → **AGREE** |

## OBSERVATIONS revision — **ratified**
> [!spec] **[[O-27]] revised** in
> `repo:OBSERVATIONS.md#o-27`:
> `[VARIANCE] … intentionally out of scope` → `[NOTE] Rule 20 on-disk
> checks are implemented as opt-in (--check-files)`. The entry now
> records: data-level always + on-disk opt-in (default off for
> path-independence); corpus-qa dogfood enables it (the parity O-27
> reconcile arm + test were removed — no longer a divergence);
> `db-to-ags4` reconstructs the `FILE/<fset>/<name>` tree. The
> earlier "out of scope" framing was a self-imposed assumption, not a
> maintainer decision; implementing the check resolved it.

## Compliance re-application (2026-07-03, #169 5a)
> [!note] O-27 is retired **for corpus-qa** (which runs `check_files`
> **ON**, so Rust stats the filesystem too and the two AGREE on Rule
> 20 directly). The cross-surface **compliance** matrix must run
> `check_files` **OFF** — its duckdb surface (`validate_ags`, since <!-- retired: validate_ags -->
> removed in the extension's 2026-07-08 0.7.0 read-only rework — whether
> the compliance harness still runs a duckdb arm at all post-removal
> isn't verified here) had no
> on-disk stat — so under that harness python's always-on Rule 20
> fires where Rust stays silent, and O-27 re-emerges. A **signature-
> narrow O-27 arm was re-added** to `laterite-ags4-parity::reconcile`
> (`po.remove("AGS Format Rule 20") → O-27`): it is **inert under
> corpus-qa** (with `check_files` on there is no py-only Rule 20 to
> match) and only reconciles the compliance harness's expected
> difference. So the OBSERVATIONS entry stands as written — O-27 is a
> documented *opt-out*, not a live divergence — and the reconcile arm
> now expresses "known under the check_files-off harness."

## Related
[[rule-20-file-fset]] · [[O-27]] · [[parity-model]] · [[parity-cascade-unreconcilable]] · [[cert-trust-v2]]
