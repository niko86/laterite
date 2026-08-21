---
type: decision
title: "The .ags.idx certificate consumer — certify · read(index=) · validate-skip (v1, SUPERSEDED)"
status: superseded
superseded_by: [cert-trust-v2]
tags: [design, decision]
decided: 2026-06-20
supersedes: []
from_gap: []
related: [dec-duckdb-perf-architecture, crate-map, laterite-ags4-core, surface-census, data-single-source-audit, laterite-cli, laterite-node, cert-trust-v2]
sources: []
---

# The `.ags.idx` certificate consumer

> [!warning] **SUPERSEDED by [[cert-trust-v2]] (2026-07-14).** This page records the
> **v1** certificate model *as it was*, and is kept for the history — not as guidance.
> Its central policy was **unsound**, in five separate ways, and the code it describes no
> longer exists. Read [[cert-trust-v2]] for the model that shipped.
>
> What v1 got wrong, in its own terms:
>
> - **`check_files` in the stamp.** A certificate recorded that Rule 20's *on-disk* half
>   had run — and a later request read that record and skipped the check. Delete the
>   `FILE/` tree in between and the file still reported clean. The certified bytes had not
>   moved, and they were never the thing in question. v2 has **no field to say it with**:
>   a certificate may speak only for computations that are a pure function of the certified
>   bytes, and the directory beside them is not one.
> - **`profile_covers` compared a forced edition and an auto-resolved one apart** — two
>   separable fields for one indivisible fact. v2 has `EditionInput::{Auto, Forced}`.
> - **The stamp's `warnings`/`fyi` counts were never measured.** The mint took them as
>   *default arguments* (`warnings=0, fyi=0`) and nothing ever passed them, so every
>   certificate this library produced claimed to have measured zero warnings without having
>   looked — and a `validate(warnings=True)` read the zero and skipped the engine. v2's
>   mint **takes no counts**: it runs every tier and records `TierCoverage::{NotMeasured,
>   Measured{count}}`, so "I never looked" and "I looked and found none" are different
>   facts.
> - **`validator_version` was a hand-bumped semver.** Edit a rule, forget the bump, and
>   every certificate from the old engine kept claiming to be current. v2 stamps a
>   build-time SHA-256 over the rule sources and the bundled dictionary.
> - **"No automation, no hidden responsibility" pushed the verdict onto the caller.**
>   `certify` vouching for a prior `.validate()` sounded like restraint; in practice it made
>   the certificate's contents an *assertion by the caller*, and the callers disagreed —
>   five hand-written trust conjunctions across the surfaces, four of which could report a
>   file clean that was not. v2's `certify` **runs the validation itself** and refuses only
>   **errors**.
>
> The v1 *index* half — byte offsets, size/SHA freshness, the remote ETag shortcut — was
> sound and survives intact in v2 (with the multi-span fix from laterite-dev#512).

## Context

[[dec-duckdb-perf-architecture]] built the `.ags.idx` **sidecar** in core
(`repo:rust-packages/laterite-ags4-core/src/index.rs`): a `Sidecar` is two things
at once — a **byte-offset index** (where each group's section lives) *and* a
**validity certificate** (a positive assertion "this exact file validated clean,
here is the proof"). The perf page consumed the *index* half (the DuckDB
extension's lazy single-group read + remote range-GET). This page is the
certificate half's **first real consumer**, in the Python `laterite` library: a
fresh certificate lets `.validate()` **skip the rule engine**.

Core owns the format and can *read* a cert validator-free, but **cannot mint** one
— validation lives in the validator crate, above core. So minting is an opt-in
action of a validator-aware layer; the Python binding (`laterite-py`, which already
links both core and the validator) is that layer.

## Decision

A three-verb lifecycle on `Ags4File`, exposed over a thin PyO3 `Sidecar` pyclass
(`assemble` / `from_json` / `is_fresh_for` / `index` + provenance getters):

- **`read(p).validate().certify(path=None)`** — *mint*. `certify` writes the cert
  beside the file (default `<source>.idx`, e.g. `delivery.ags` → `delivery.ags.idx`).
  It indexes the **original source bytes** (not the spec re-emit) and stamps the
  validation (validator id + version + UTC `checked_at`).
- **`read(p, index=cert)`** — *consume*. Explicit opt-in (no autodiscovery). The
  cert is loaded and freshness-checked (format version + size + SHA-256) against the
  source bytes; a fresh cert is carried on the handle.
- **`read(p, index=cert).validate()`** — *skip*. The engine is skipped (and `report`
  is a synthesised clean report from `Report.from_cert`, `resolution == "certified"`)
  only when the carried cert is **fresh** (bytes), minted by the **same engine**
  (`checker_matches` — validator + engine version + compat), and its **profile covers**
  the request (`profile_covers` — ran ≥ the requested `check_files`, same edition
  forcing). A `warnings`/`fyi` request never skips (the cert stores advisory *counts*,
  not the findings to replay).

Two owner-settled rules shape the edges:

- **`certify` never auto-validates.** It *vouches for* a prior clean `.validate()`,
  reading the handle's `report`; it raises if `.validate()` was not called, or if it
  found findings. The validation result lives on the handle, so there is no
  "validation object" to pass and no standalone `certify()` function — it is a method
  only. Keeps `certify`'s responsibility narrow (mint, don't re-derive).
- **A stale cert fails fast at `read()`.** An explicit `index=` *asserts* the cert is
  for this file, so a mismatch raises `StaleCertError` at read time — never a silent
  fall-back to re-validation (which would mask a file that drifted under the cert).
  Asking `.validate()` for **more** than the cert vouches for (a stronger profile, or
  a different checker) re-validates rather than trusting a verdict today's rules might
  not reproduce.

## The cert format (locked at version 1, grows by optional fields)

A multi-lens design review settled the format: it **locks at `version: 1`** and grows
only by `serde(default)` optional fields — old certs always deserialise, and the cert
is a regenerable cache, so no future feature forces a re-mint. Beyond the byte index +
size/SHA freshness, the format carries:

- **Provenance / safe skip.** `validator_version` is the validation *engine* version
  (not the binding's), so a cert is comparable across surfaces (a cert minted by the
  DuckDB extension is trusted by Python and vice versa); `compat` records the
  python-ags4 version when validated through `laterite.compat`. A clean verdict is
  trusted only when this checker identity still matches (`checker_matches`).
- **Check profile.** `check_files` (ran Rule 20's on-disk half — a real *error*) and
  `edition_forced` (forced vs `TRAN_AGS`-auto edition — possibly a different
  dictionary). `profile_covers` enforces "cert profile ≥ request", closing a
  false-clean hole where a default-minted cert would wrongly satisfy a stricter request.
- **Remote freshness.** `file.etag` + `file.last_modified` (optional) let a *remote*
  reader confirm freshness with a single HEAD instead of downloading the whole object
  to re-hash (which would defeat the ranged-GET the cert exists for). SHA-256 **stays
  the compulsory** strong, portable check; ETag is a cheap shortcut layered on top —
  the pure, I/O-free `is_fresh_for_remote` only ever *grants* trust on a match and
  *downgrades* to the SHA path otherwise (it can never make a stale cert look fresh).
  Network I/O stays at the call site; core never touches it.

## Why

- **The cert is coherent because the index and the verdict share one parser.** As of
  #168 Phase 4 the byte offsets come from the shared parse leaf
  (`laterite_ags4_parse`) source-true byte walk, and `.validate()` parses via the
  validator's `parse` module — itself a thin adapter over that same leaf (Phase 2). So
  an index and the verdict it certifies are produced by *one* parser, not two held in
  sync. (The old **parser-parity gate** `parse_parity.rs` — which guarded the leaf against
  the legacy csv `ags4_codec` — was retired at Phase 7 once both parsers became the one
  leaf, down to `repo:rust-packages/laterite-ags4-validator/tests/from_shared_trim.rs`,
  which keeps only the real trim-asymmetry guard, fork 1.) The offsets are now the **true GROUP
  line-starts** — the csv reader recorded the preceding `\n` for CRLF and absorbed
  leading blank lines; see `repo:OBSERVATIONS.md#o-40` ([[O-40]]).
- **No automation, no hidden responsibility** (the owner's framing): `certify` doesn't
  validate, `read(index=)` doesn't silently swallow a mismatch. Each step does one
  thing and surfaces what it can't do.
- **Mint over the original source bytes, not the re-emit** — so the index points into
  the actual on-disk file (what a remote range-GET / sliced read needs) and the cert
  certifies the bytes that were validated. UTF-8 only (the byte index rejects other
  encodings, matching the parity gate's scope).

## Consequences

- The sidecar now has consumers on **both** its axes: the index half (DuckDB ext,
  [[dec-duckdb-perf-architecture]]) and the certificate half (this page). They share
  one core format.
- `resolution` gains a non-edition value, `"certified"` — documented on `Report`.
- **Advisory counts are errors-only by default.** The canonical mint path runs the
  default (errors-only) `.validate()`, so the stamp records `warnings=0, fyi=0`; the
  cert certifies the *error-clean* property. (Follow-on: thread measured advisory
  counts through if a future `.validate(warnings=True)` path wants them stamped.)
- Same-content, different-path is fine (the cert keys on bytes, not path); a
  same-size in-place edit is caught by the SHA (unlike the perf cache's `(path,size)`
  key — a strictly stronger check here).

## Known divergence — CLOSED by the v2 trust model

v1 left the binary and the libraries honouring `--index` **differently**: `lat`'s own
`try_certified_skip` skipped whenever the cert was fresh + checker-matching +
profile-covering, regardless of `--no-warnings`/`--show-fyi`, while Python and Node
refused to skip at all for a warnings/FYI request (their certs stored *counts*, not
findings to replay). Same verdict, different work — and a declaration-level census
structurally cannot see it, because it compares what each launcher *advertises*, not what
it *does*.

There is now **one** decision, in `laterite-ags4-trust`, for every surface: a certificate
may stand in for a tier iff it **measured** that tier and found it **empty**. So an
errors-only request against a clean cert skips everywhere; a `--show-warnings` request
skips iff the cert measured zero warnings; and a cert that counted a warning cannot answer
for it (it knows there is something to say, but not what — `TierNotClean`). The five
conjunctions are gone, and with them the divergence. See [[cert-trust-v2]] and
[[data-single-source-audit]] row 6.

## Implementation status

| Piece | Where | State |
|---|---|---|
| Parser-parity gate | `repo:rust-packages/laterite-ags4-validator/tests/from_shared_trim.rs` | **done** — the `parse_parity.rs` gate (PR #183) was retired at #168 Phase 7 once the two parsers converged on the shared leaf; its one meaningful assertion (the interior-quoted-whitespace trim) survives here |
| `Sidecar` PyO3 pyclass | `repo:rust-packages/laterite-py/src/lib.rs` | **done** — assemble/from_json/is_fresh_for/index + getters |
| `certify` / `read(index=)` / validate-skip | `repo:packages/laterite/python/laterite/__init__.py` | **done** (PR #184) — `test_certificate.py` |
| provenance + profile + remote-freshness format | `repo:rust-packages/laterite-ags4-core/src/index.rs` | **done** — engine-version + `compat` + `check_files`/`edition_forced` + `etag`/`last_modified`; `checker_matches`/`profile_covers`/`is_fresh_for_remote` |
| DuckDB `certify_ags` + `read_ags` slice + version-aware `validate_ags` | `niko86/laterite-duckdb` | **done** (PR #3, merged), then **removed** in the 0.7.0 read-only rework — `validate_ags`/`certify_ags` dropped; the extension now only consumes an externally-minted `.ags.idx` (validate-and-mint stays a CLI/library operation). `read_ags`'s **size**-gated sliced read survives; same `laterite_ags4`+engine-VERSION checker identity as the Python wheel | <!-- historical -->

**Deferred (add as `serde(default)` optional fields when a consumer needs them):**
per-group row-count cardinality hint (graduates when `read_ags` slices off the full
parse), a `producer` binding string, advisory *findings* replay (not just counts),
hash-agility, signing. **Rejected:** replacing SHA with ETag, a baked freshness tier
enum or stored "trusted" bool, network I/O in core, per-group schema duplication, a
version bump, `deny_unknown_fields`, required non-Option fields.
| `StaleCertError` | `repo:packages/laterite/python/laterite/_errors.py` | **done** |

## Related
[[dec-duckdb-perf-architecture]] · [[crate-map]] · [[laterite-ags4-core]] · [[surface-census]] · [[data-single-source-audit]] · [[laterite-cli]] · [[laterite-node]]
