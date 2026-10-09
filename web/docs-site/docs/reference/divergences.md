# Where laterite and python-ags4 differ

laterite is an **independent** implementation of the AGS4 rules, calibrated against the incumbent
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) on its own test corpus
(see [Cross-surface parity](../concepts/cross-surface-parity.md)). Two independent implementations of
one specification will disagree, so every disagreement is written down rather than smoothed over.

**27 of them change what you see.** They are not all the same kind of thing, which is why this
page is grouped by what actually happened rather than filed under one heading: some are deliberate
differences from python-ags4, some are places the two agree and the *spec* is the outlier, and some are
laterite's own false negatives that the comparison caught and closed.

This is the user-facing list. The full catalogue lives in `OBSERVATIONS.md` in the repo, and adds the
internal NOTE/SPEC entries along with the records since resolved. This page is generated from the
same source, so a record cannot be resolved there and stay live here.

## Where laterite differs from python-ags4

| # | What you see |
|---|---|
| **O-2** | Rule 6 is a **no-op** in python-ags4 (its body is `return ags_errors`). laterite implements the embedded-CR check. |
| **O-8** | python-ags4's `rule_7_2` can raise `IndexError` on duplicate headings; laterite bounds-guards it and reports. |
| **O-12** | The DT/datetime validity engine is `chrono`-based and stays lenient on semantics for UNIT shapes the AGS dictionary does not define, where pandas would still attempt a parse. |
| **O-30** | The dictionary edition is selected from `TRAN_AGS`. An AGS 3.x file is refused outright, where python-ags4 silently validates it against 4.1.1. |
| **O-34** | A non-AGS4 file surfaces as a clean `NotAgs4` error, reconciled against python-ags4's "missing mandatory groups". |
| **O-37** | The native parser is **lenient** where python-ags4 raises hard (duplicate GROUP, ragged rows): findings first, never a crash. |
| **O-41** | Rows before the first GROUP are reported as Rule 2 findings, not a parser crash. |
| **O-42** | `TRAN_AGS="4.0"` resolves to **4.0.4**, the newest 4.0 patch; python-ags4's static map picks the oldest and over-reports Rule 10c on `PMTL`. |
| **O-53** | A **blank** `TRAN_AGS` is reported once, as the Rule 10b error; which dictionary the verdict then fell back to is stated on the report itself rather than as a second finding. |
| **O-45** | An unrecognised `TRAN_AGS` edition → a laterite **WARNING** (Related to Rule 14), shown by default and **not fatal**. |
| **O-49** | A numeric TYPE's count, the `n` in `nDP`/`nSF`/`nSCI`, is clamped to **30**. Read uncapped, a crafted `9999999999SF` drives python-ags4 into a ~10 GB string. |
| **O-50** | A 0DP value outside `i64` **converts to Null**; python-ags4's conversion keeps full precision. Both validators flag the cell, so the difference is in conversion, not validation. |

## Where both depart from the written spec

| # | What you see |
|---|---|
| **O-1** | Rule 1 downgrades extended ASCII (128–255) to an **FYI** rather than a hard error, matching python-ags4's relaxation of the spec's "entirely ASCII". |
| **O-54** | A field holding concatenated abbreviations is only split when TRAN_RCON is populated. The specification names "+" as the default when it is absent; neither laterite nor python-ags4 applies it, so such a field is reported as one undefined abbreviation instead of being split into its parts. |
| **O-32** | Non-UTF-8 input is decoded **lossily** (python's `errors="replace"`), not refused. |

## Where laterite changed to match python-ags4

| # | What you see |
|---|---|
| **O-31** | Rule 8 flags an empty `DT` UNIT. This was laterite's own false negative, found by the comparison and closed to match python-ags4. |
| **O-33** | DT/datetime validity is bounded to pandas' Timestamp range. The false negative was again laterite's own (the year `0018` was accepted), closed to match python-ags4. |

## Checks laterite adds

| # | What you see |
|---|---|
| **O-43** | A self-declared but non-standard `PA` abbreviation → a laterite **FYI** (Related to Rule 16). |
| **O-44** | Structural validation of a file-level `DICT` group → a laterite **WARNING** (Related to Rule 18). |
| **O-51** | A custom-dictionary **overlay** that redefines the standard schema is reported as a **WARNING** when it changes row identity (re-parent, KEY demotion) and an **FYI** otherwise. |
| **O-52** | A child row whose parent-KEY cells are all empty gets a **WARNING** saying the parentage check was declined, rather than silently producing nothing. python-ags4's Rule 10c also merges the UNIT/TYPE descriptor rows as data, so a child whose TYPE row types a KEY column differently from its parent's is **rejected** there on its TYPE line; laterite does not check descriptor rows under Rule 10c and surfaces the departure as an FYI (Related to Rule 8). |
| **O-56** | A group that keys on a heading owned outside its declared parent chain gets an **FYI** naming the group that could have been its parent. Rule 10c is structurally unable to check that link, so a reference through it can be an orphan on a file that validates clean. |
| **O-57** | A `PA` value that fails Rule 16 only because of whitespace around a defined code (`" D"` where `"D"` is defined) → a laterite **FYI** (Related to Rule 16) naming the code, and `fix` trims it. |
| **O-58** | Two ABBR codes under one heading that differ only by letter case (`"Undisturbed"` / `"UNDISTURBED"`) → a laterite **FYI** (Related to Rule 16) naming both, since some importers treat them as one code; under 4.2, whose spec makes ABBR codes case-insensitive, a **WARNING** (Related to Rule 16) instead. |
| **O-59** | A standard heading whose TYPE row differs from the dictionary edition's type (`LNMC_MC` declared `1DP` where 4.1.1 says `X`) → a laterite **FYI** (Related to Rule 8), since files typed differently clash once combined. Native surfaces only: On a **KEY** heading it is a **WARNING**, shown by default: that is the shape python-ags4's Rule 10c rejects on the child's TYPE line (O-52). `laterite.compat` emits neither. |
| **O-60** | A DICT row for a group or heading the dictionary edition already defines → a laterite **FYI** (Related to Rule 18) listing what it changes, or a **WARNING** (Related to Rule 18) when it drops KEY from a standard KEY heading or re-parents a standard group. The standard definition applies either way. `build_ags4` and `merge` drop such rows with `dict_rows="prune"`. |
| **O-61** | An undefined `PA` value close to a code the file declares or the dictionary lists (`"UNDISTURBD"` for `"UNDISTURBED"`) → a laterite **FYI** (Related to Rule 16) naming up to three likely codes. A suggestion only; nothing is rewritten. |

!!! tip "Reading the tiers"
    Whether a difference surfaces as an **error**, **warning** or **FYI** follows
    laterite's [severity tiers](../concepts/severity-tiers.md).
