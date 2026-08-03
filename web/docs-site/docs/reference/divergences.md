# Known divergences from python-ags4

laterite is a **clean-room** re-implementation of the AGS4 rules, validated
against the incumbent [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)
on its own test corpus (see [Cross-surface parity](../concepts/cross-surface-parity.md)).
Where the two _deliberately_ differ, the difference is catalogued as an **O-N**
observation. This is the user-facing list; the full catalogue (including internal
NOTE/SPEC entries) lives in `OBSERVATIONS.md` in the repo.

Every one of these is intentional and tested — the compliance harness reconciles
them so they never read as a regression.

## Encoding & edition

| #                             | Divergence                                                                                                |
| ----------------------------- | --------------------------------------------------------------------------------------------------------- |
| **O-32**                      | Non-UTF-8 input is decoded **lossily** (like python's `errors="replace"`), not refused.                   |
| **O-1**                       | Rule 1's "entirely ASCII" is treated as an FYI for extended-ASCII, not a hard error.                      |
| **O-30**                      | Edition is selected from `TRAN_AGS`; some deliberate differences from python's mapping.                   |
| **O-42**                      | `TRAN_AGS="4.0"` resolves to 4.0.4 (superset-safe); python's static 4.0→4.0.3 over-reports Rule 10c.      |
| **O-10 / O-20 / O-25 / O-28** | Dictionary selection is `TRAN_AGS`-driven and self-contained; the external `--dict` override is deferred. |

## Rule attribution & value checks

| #               | Divergence                                                                                                  |
| --------------- | ----------------------------------------------------------------------------------------------------------- |
| **O-2**         | Rule 6 is a **no-op** in python-ags4; laterite implements the embedded-CR check.                            |
| **O-8**         | python-ags4's `rule_7_2` can raise `IndexError` on duplicate headings; laterite handles it.                 |
| **O-12 / O-33** | The DT/datetime validity engine differs from pandas (and is bounded to pandas' Timestamp range for parity). |
| **O-31**        | Rule 8 flags an empty `DT` UNIT (closing a python-parity gap).                                              |

## Leniency & structure

| #        | Divergence                                                                                                                   |
| -------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **O-37** | The native parser is **lenient** where python-ags4 raises hard (duplicate GROUP, ragged rows) — findings-first, never crash. |
| **O-41** | Rows before the first GROUP are reported as Rule 2 findings, not a parser crash.                                             |
| **O-34** | A non-AGS4 file surfaces as a clean `NotAgs4` error, reconciled against python's "missing mandatory groups".                 |

## Checks laterite adds

| #        | Divergence                                                                                                               |
| -------- | ------------------------------------------------------------------------------------------------------------------------ |
| **O-43** | A self-declared but non-standard `PA` abbreviation → a laterite FYI (Related to Rule 16). python-ags4 has no such check. |
| **O-44** | Structural validation of a file-level `DICT` group → a laterite WARNING (Related to Rule 18).                            |
| **O-45** | An unrecognised `TRAN_AGS` edition → a laterite WARNING (Related to Rule 14), shown by default.                          |

!!! tip "Reading the tiers"
    Whether a divergence surfaces as an **error**, **warning**, or **FYI** follows
    laterite's [severity tiers](../concepts/severity-tiers.md).
