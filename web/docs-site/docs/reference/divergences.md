# Where laterite and python-ags4 differ

laterite is an **independent** implementation of the AGS4 rules, calibrated against the incumbent
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) on its own test corpus
(see [Cross-surface parity](../concepts/cross-surface-parity.md)). Two independent implementations of
one specification will disagree, so every disagreement is written down rather than smoothed over.

**20 of them change what you see.** They are not all the same kind of thing, which is why this
page is grouped by what actually happened rather than filed under one heading: some are deliberate
differences from python-ags4, some are places the two agree and the *spec* is the outlier, and some are
laterite's own false negatives that the comparison caught and closed.

This is the user-facing list. The full catalogue — including the internal NOTE/SPEC entries and the
records since resolved — lives in `OBSERVATIONS.md` in the repo, and this page is generated from the
same source, so a record cannot be resolved there and stay live here.

## Where laterite differs from python-ags4

| # | What you see |
|---|---|
| **O-2** | Rule 6 is a **no-op** in python-ags4 — its body is `return ags_errors`. laterite implements the embedded-CR check. |
| **O-8** | python-ags4's `rule_7_2` can raise `IndexError` on duplicate headings; laterite bounds-guards it and reports. |
| **O-12** | The DT/datetime validity engine is `chrono`-based and stays lenient on semantics for UNIT shapes the AGS dictionary does not define, where pandas would still attempt a parse. |
| **O-30** | The dictionary edition is selected from `TRAN_AGS`. An AGS 3.x file is refused outright, where python-ags4 silently validates it against 4.1.1. |
| **O-34** | A non-AGS4 file surfaces as a clean `NotAgs4` error, reconciled against python-ags4's "missing mandatory groups". |
| **O-37** | The native parser is **lenient** where python-ags4 raises hard (duplicate GROUP, ragged rows) — findings first, never a crash. |
| **O-41** | Rows before the first GROUP are reported as Rule 2 findings, not a parser crash. |
| **O-42** | `TRAN_AGS="4.0"` resolves to **4.0.4**, the newest 4.0 patch; python-ags4's static map picks the oldest and over-reports Rule 10c on `PMTL`. |
| **O-53** | A **blank** `TRAN_AGS` is reported once, as the Rule 10b error; which dictionary the verdict then fell back to is stated on the report itself rather than as a second finding. |
| **O-49** | A numeric TYPE's count — the `n` in `nDP`/`nSF`/`nSCI` — is clamped to **30**. Read uncapped, a crafted `9999999999SF` drives python-ags4 into a ~10 GB string. |
| **O-50** | A 0DP value outside `i64` **converts to Null**; python-ags4's conversion keeps full precision. Both validators flag the cell — the difference is in conversion, not validation. |

## Where both depart from the written spec

| # | What you see |
|---|---|
| **O-1** | Rule 1's "entirely ASCII" is an **FYI** for extended ASCII (128–255), not a hard error — matching python-ags4, which the spec text does not. |
| **O-32** | Non-UTF-8 input is decoded **lossily** (python's `errors="replace"`), not refused. |

## Where laterite changed to match python-ags4

| # | What you see |
|---|---|
| **O-31** | Rule 8 flags an empty `DT` UNIT. This was laterite's own false negative, found by the comparison and closed to match python-ags4. |
| **O-33** | DT/datetime validity is bounded to pandas' Timestamp range — again laterite's own false negative (the year `0018` was accepted), closed to match python-ags4. |

## Checks laterite adds

| # | What you see |
|---|---|
| **O-43** | A self-declared but non-standard `PA` abbreviation → a laterite **FYI** (Related to Rule 16). |
| **O-44** | Structural validation of a file-level `DICT` group → a laterite **WARNING** (Related to Rule 18). |
| **O-45** | An unrecognised `TRAN_AGS` edition → a laterite **WARNING** (Related to Rule 14), shown by default and **not fatal**. |
| **O-51** | A custom-dictionary **overlay** that redefines the standard schema is reported — a **WARNING** when it changes row identity (re-parent, KEY demotion), an **FYI** otherwise. |
| **O-52** | A child row whose parent-KEY cells are all empty gets a **WARNING** saying the parentage check was declined, rather than silently producing nothing. |

!!! tip "Reading the tiers"
    Whether a difference surfaces as an **error**, **warning** or **FYI** follows
    laterite's [severity tiers](../concepts/severity-tiers.md).
