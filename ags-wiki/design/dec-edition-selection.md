---
type: decision
title: "TRAN_AGS-driven edition selection: the deliberate divergences from python-ags4"
status: accepted
tags: [design, decision, validator, dictionary]
decided: "2026-05-16"
supersedes: []
from_gap: []
related: [dec-dictionary-single-source, rust-vs-python-ags4-parity, laterite-ags4-validator, edition-resolution]
sources: []
---

# TRAN_AGS-driven edition selection: the deliberate divergences from python-ags4

The design rationale behind observation **O-30**. The observation itself states
the *what* + the one *because*; this page holds the fuller reasoning (corpus
evidence, rejected alternatives, provenance tracking) that would otherwise bloat
the catalogue entry.

## Context

We bundle all five AGS4 editions python-ags4 ships (4.0.3 / 4.0.4 / 4.1 / 4.1.1 /
4.2) and `resolve_dict_version` picks one per file from its `TRAN_AGS`, with an
explicit `--dict-version` overriding. python-ags4's `pick_standard_dictionary`
instead uses a fixed exact-string map with `LATEST_DICT_VERSION = "4.1.1"` as the
catch-all. Three of our resolutions deliberately diverge; the rest is
python-identical on purpose.

## The general rule

An **exact** bundled string wins (`4.1`→4.1, `4.1.1`→4.1.1, `4.2`→4.2 keep
python-parity); otherwise the **newest bundled patch of that `major.minor`**
(`4.0`→4.0.4, `4.1.5`→4.1.1, `4.2.7`→4.2). A file tagged `4.0` is best served by
the latest 4.0.x schema, not the oldest — AGS versioning is traditionally
`major.minor`, so the patch is a compatibility detail, and the newest patch is a
superset-safe read.

## The corpus evidence behind bare `"4"` → 4.0.4

A 12,503-file real dogfood run (`sandbox/`) showed **41% of files declare a bare
`TRAN_AGS = "4"`** (no usable minor: `"4"`, `"4."`, `"4.x"`), and the *same
producers'* `"4.0"` files already resolve to 4.0.4 — i.e. ~5,100 4.0-era files
were being mis-editioned to python's `4.1.1` fallback. "4" colloquially means
AGS4(.0); the original/most-common line is the safer, deterministic, per-file
choice. python has no bare-`4` key, so it → `4.1.1`.

## Rejected alternatives

Dynamic / statistical / findings-minimising selectors were considered and
rejected: they break per-file determinism and `--seed` reproducibility and the
python-parity premise, and "fewest findings" masks real defects. Edition
selection must be a pure function of one file's `TRAN_AGS`, nothing else.

## AGS 3.x → hard error, not silent fallback

python silently falls an AGS 3.x `TRAN_AGS` back to 4.1.1 and validates it against
an AGS4 schema. Nothing AGS3 is specced here, so we refuse. AGS3 is detected at
*parse* by its unambiguous signature (`**GROUP` / `<UNITS>` / `<CONT>`) and raised
as `UnsupportedEdition { found: "3.x (AGS3 format)" }` — a clear edition error,
not the misleading generic `NotAgs4("no GROUP rows found")` it used to fall
through to. The corpus-QA parity classifier folds "Rust refuses AGS3 + python
validated it" into `KNOWN_DIVERGENCE (O-30)` so the expected AGS3 divergences in
a real corpus leave the parity ACTION list instead of swamping it.

## Provenance tracking

`resolve_dict_version` returns `(DictVersion, DictResolution)`; the second value
(`forced` / `exact` / `guessed` / `fallback`) lets a batch run distinguish a
genuine `TRAN_AGS` edition from this fallback — the blind spot that O-31 surfaced
(fallback files were indistinguishable from genuine 4.1.1). Older 4.0.x
dictionaries are Latin-1 (cp1252); `build.rs` decodes them byte→char
(ISO-8859-1) — lossless, dependency-free, the same 0–255 tolerance O-1 documents.

See [[rust-vs-python-ags4-parity]] for how these resolutions land in the parity
baseline, and [[dec-dictionary-single-source]] for where the bundled editions
come from. [[edition-resolution]] covers the generated `DictVersion` set this
resolver draws on and the 2026-07-14 sweep that stopped other surfaces
hand-copying it.
