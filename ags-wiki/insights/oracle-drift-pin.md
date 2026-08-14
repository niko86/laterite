---
type: insight
title: "The parity oracle (python-ags4) was a floor, not a pin — silent drift invalidates every reconcile arm"
status: confirmed
tags: [insight]
gap_kind: rust-vs-python
severity: high
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: false
feeds_strategy: [strat-parity-matrix]
feeds_ags5_req: []
discovered_phase: D
related: [parity-model, observations-coverage-map, rust-vs-python-ags4-parity, vendored-authority-faithful]
sources: []
---
# Oracle drift — the biggest dogfooding blind spot

## Claim
> [!divergence] The entire Rust↔python divergence catalogue
> ([[observations-coverage-map|O-1..O-34]]) and all
> `repo:rust-packages/laterite-ags4-corpus-qa/src/parity.rs` `reconcile`/
> `classify` arms (O-2/O-3/O-26/O-27/O-30/O-34) are encoded against
> python-ags4 **1.2.0 source behaviour** (rule_6 no-op, rule_20
> on-disk, default `rename_duplicate_headers` shielding O-8, …). It was
> pinned `python-ags4>=1.2.0` — a **floor, not a pin** — and
> `--selfcheck` only checked importability. A silent minor bump (or a
> different machine resolving newer) would make a reconcile arm wrong
> with **no test failing**. Every quantitative OBSERVATIONS figure was
> also unfalsifiable from the repo (private network-share corpus).

## Evidence
- Was `repo:pyproject.toml` `"python-ags4>=1.2.0"`; `--selfcheck`
  emitted only `{"ok":true}` (`tools/py_ags4_check_json.py`).
- `reconcile()` arms are literal python-source assumptions:
  `repo:rust-packages/laterite-ags4-corpus-qa/src/parity.rs` (O-2 Rule 6 no-op,
  O-3 Rule 5↔4, O-26 triple-19b, O-27 on-disk).

## What was done (this campaign)
> [!note] **Confirmed + fixed.**
> 1. `pyproject.toml` → `python-ags4==1.2.0` (exact, commented). At the
>    time of this campaign, the *library* `packages/ags5-ags4` kept <!-- retired: ags5-ags4 -->
>    `>=1.2.0` deliberately — a shipped lib must not hard-pin its deps;
>    the oracle guarantee was the dev pin + lock + the runtime assertion
>    below, not the library contract. (`ags5-ags4` was later deleted <!-- retired: ags5-ags4 -->
>    entirely in the F2c arc — this loose-pin rationale is historical.)
> 2. `tools/py_ags4_check_json.py --selfcheck` now emits
>    `{"ok":true,"python_ags4":"<ver>"}`.
> 3. `parity.rs` const `EXPECTED_PYAGS4="1.2.0"`; the `--selfcheck`
>    probe reads the version and prints a **loud stderr warning** on
>    drift (still runs — parity is optional QA; the warning is the
>    signal to re-probe + bump). Silent → loud.

## Why it matters
A clean-room equivalence claim that silently rests on one unpinned
oracle version is not a claim. The pin + assertion convert "trust me"
into "fails loudly if the ground moves" — and it is the seed of the
AGS5 requirement req-reproducible-conformance-corpus (ship a
versioned conformance corpus + pinned reference oracle).

## Update (2026-07-17) — #558 extended the pin to the data it protects
> [!note] This page's fix pinned the *version string* and made drift loud
> via `--selfcheck`; it never checked whether the five vendored `.ags`
> dictionaries — themselves derived from that pinned python-ags4 — still
> matched what `PROVENANCE.md` claims they are. `tests/test_vendored_authority_faithful.py`
> now checks them byte-for-byte against the installed oracle (plus the file
> set, the `fallback_edition` behaviour, and all four hand-written `1.2.0`
> claims across the tree), and `repo:tools/check_upstream_pin.py` (a new
> `upstream-pin` job on `parity.yml`'s scheduled cron — weekly since 2026-07-24,
> monthly when this was written <!-- cadence: historical=monthly --> — never
> a PR) adds the <!-- cadence: parity -->
> (that "weekly since" is itself now gated — see
> [[stated-cadences-faithful]])
> direction nothing checked before: noticing when PyPI moves *past* the
> pin. See [[vendored-authority-faithful]] for the full account — including
> the mutation-test proof of the gap and the honest limit that this still
> cannot show python-ags4 itself matches the AGS spec, only that we match
> python-ags4.

## Related
[[parity-model]] · [[observations-coverage-map]] · [[rust-vs-python-ags4-parity]] · req-reproducible-conformance-corpus · [[vendored-authority-faithful]]
