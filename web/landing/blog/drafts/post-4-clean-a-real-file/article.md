# What is wrong with the AGS file you were sent?

_Post 4 of 5. Scripts: `check.py` and `clean_recipe.py`, beside this draft._

The file is a BGS export of twelve Crossrail boreholes at Farringdon: the
same one as post 1. It reads fine, and post 1 drew a log from it. It is also
a long way from valid AGS4.

## Check

```python
import laterite
import polars as pl

ags = laterite.read("your_file.ags")
report = ags.validate().report
print(f"{report.errors} errors, {report.warnings} warnings")
# 1488 errors, 13 warnings
```

A file of 1,393 lines with 1,488 errors. Before anyone panics, count them by
rule. The findings are a polars frame:

```python
by_rule = (
    report.findings.group_by("rule")
    .agg(pl.len().alias("n"), pl.col("desc").first().alias("for_example"))
    .sort("n", descending=True)
)
```

| Rule | Findings | For example                                                        |
| ---- | -------- | ------------------------------------------------------------------ |
| 2a   | 1,394    | Line is not terminated by CR LF                                    |
| 8    | 24       | Value "1992-12-04" in LOCA_STAR does not match its unit dd/mm/yyyy |
| 10a  | 23       | Duplicate KEY field combination: LOCA_TYPE\|CP                     |
| 10b  | 19       | Empty REQUIRED field: UNIT_DESC                                    |
| 19b  | 9        | Heading "BGS_PROJ_ID" is not group prefix + up to 4 characters     |
| 9    | 5        | Heading "BGS_PROJ_ID" is not in the dictionary or the file's DICT  |
| 16   | 5        | Abbreviation "HEADING" is not defined in ABBR                      |
| 19a  | 5        | Heading "BGS_PROJ_ID" is more than 9 characters long               |
| 7    | 4        | Heading order cannot be checked                                    |

Now it is a different file. Ninety-four per cent of the findings are one
fault: Unix line endings where the format wants Windows ones. The other 94
are eight kinds of thing, and several of those are the same fault seen by
more than one rule.

## Fix what is mechanical

```python
fixed = ags.fix()
for change in fixed.fix_report.applied:
    print("fixed:", change["label"])
# fixed: Normalise all line endings to CRLF (Rule 2a)

print(fixed.validate().report.errors)
# 94
```

`.fix()` returns a new handle and leaves the original alone. By default it
applies only safe repairs: ones that cannot change the meaning of the data.
There is a second tier, `fix(risky=True)`, for repairs that involve a guess.
On this file it finds nothing more to do. The 94 that remain are not
mechanical. Each needs someone to decide something.

## Clean the rest by hand

`clean_recipe.py` does it in seven more steps. The pattern is the same each
time: take the groups as frames, edit one with polars, rebuild, validate,
watch the count fall.

| Step                                           | Errors left |
| ---------------------------------------------- | ----------- |
| As sent                                        | 1,488       |
| 1. `.fix()`                                    | 94          |
| 2. Drop repeated rows in ABBR, BKFL, WSTG      | 75          |
| 3. Correct the declared date unit              | 51          |
| 4. Drop unit rows that are unused and blank    | 32          |
| 5. Declare the TRAN delimiter and concatenator | 31          |
| 6. Define the codes and units that are used    | 24          |
| 7. Rename and declare the columns BGS added    | 1           |
| 8. Remove one stale DICT row                   | 0           |

Three of those are worth reading closely, because the judgement is the point.

**Step 3: fix the declaration, not the data.** The dates are written
`2009-02-18` and the UNIT row says `dd/mm/yyyy`. There are two ways to make
them agree. The values are unambiguous ISO dates, so the declaration is the
thing that is wrong:

```python
for heading in ("LOCA_STAR", "LOCA_ENDD"):
    units["LOCA"][heading] = "yyyy-mm-dd"
```

Two lines clear 24 findings, and no date is touched.

**Step 2: which duplicate do you keep?** BKFL and WSTG repeat whole rows, and
dropping the copy is safe. ABBR is different: it defines some codes twice,
the second time with an extra remark. That is a choice, and the script makes
it where you can see it:

```python
frames["ABBR"] = frames["ABBR"].unique(
    subset=["ABBR_HDNG", "ABBR_CODE"], keep="first", maintain_order=True
)
for group in ("BKFL", "WSTG"):
    frames[group] = frames[group].unique(maintain_order=True)
```

**Step 6: do not invent what the source does not say.** `GEOL_LEG` uses code
`407` and ABBR never defines it. The validator wants a description. I do not
know what BGS meant by 407, so the script writes "Undefined in the source"
and says so in a comment. A file that validates because someone made up a
description is worse than one that fails.

Rebuilding uses `build_ags4`, with one thing to know: hand it the file's own
units and types, or it fills them in from the standard dictionary and
overwrites declarations you meant to keep.

```python
frames = {g: fixed[g] for g in fixed.groups}
units = {g: dict(zip(fixed.headings(g), fixed.units(g), strict=True)) for g in fixed.groups}
types = {g: dict(zip(fixed.headings(g), fixed.types(g), strict=True)) for g in fixed.groups}

built = laterite.build_ags4(frames, units=units, types=types, mode="report")
clean = laterite.read(built.text)
```

## What changed, and what did not

Every finding was in the file's bookkeeping: UNIT, ABBR, DICT, TRAN, a unit
declaration, five column names. The boreholes, strata, samples and SPTs come
out with exactly the values they went in with. That is the test of a clean:
the validator is satisfied and the ground has not moved.

## What a valid file is not

Valid means the file obeys the format. It does not mean the data is right.
Borehole F1 in this file started on 4 December 1992 and finished on 15 April 1992. No AGS rule objects to that, and neither does this recipe.

The recipe is also for this file only. It leaves a finding or three on the
neighbouring Crossrail files and fails outright on other projects, whose
groups differ. Cleaning is specific. What carries over is the method: count
by rule, fix what is mechanical, then decide the rest one kind at a time.

## What laterite should do for you, and does not yet

Writing this recipe showed me four places where laterite made me do work it
could have done. They are open issues now:

- Dates in ISO form under a `dd/mm/yyyy` unit: a fix should offer to correct
  the declaration.
- Rows that are identical in every field: dropping them is a safe fix.
- An undeclared concatenator when the data plainly uses one (`CP+RC`).
- Editing one group and writing the file back should not need the
  units-and-types dance above.

Not everything should be automatic. Steps 6 and 7 need a person. But the
dull ones should be one call.

## No file to hand?

[laterite-quarry](https://github.com/niko86/laterite-quarry) fetches this
file:

```
uvx --from laterite-quarry quarry bgs pull farringdon --bbox -0.108,51.518,-0.102,51.522
```

The pull arrives in three pieces. The file used here is the first,
`bgs/farringdon/cell000/202002061142130.ags`.

_Contains British Geological Survey materials ©NERC 2026. Run on laterite 0.17.1._
