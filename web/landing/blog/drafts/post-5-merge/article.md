# Seventeen files in, one file out

_Post 5 of 5. Scripts: `merge_files.py` and `type_clash.py`, beside this draft._

![The site's soil units at its nine locations, west to east, from the merged file](site.png)

Fugro's borehole campaign for site III of the Hollandse Kust (zuid) wind farm
zone is published as seventeen AGS4 files, one per borehole. That is a
sensible way to issue data and an awkward way to use it. Posts 2 and 3 asked
questions about the whole site: all 53 Atterberg results, every stratum. For
that you want one file.

## Merge

```python
from pathlib import Path

import laterite
from laterite import TranStamp

files = sorted(p for p in Path("folder").iterdir() if p.suffix.lower() == ".ags")

stamp = TranStamp(
    issue="1",
    date="2026-11-28",
    producer="Example Ltd",
    recipient="Example Client",
    status="Final",
    description="All boreholes, merged",
)
result = laterite.merge(*files, tran=stamp)
merged = laterite.read(result.bytes)
```

`merge` takes any number of sources and returns the merged file as bytes,
along with a list of warnings. It does the parts that go wrong by hand: one
set of headings per group, and one UNIT, TYPE, ABBR and DICT table reconciled
from all the inputs. Rows are matched on their KEY headings, not their
position, so a record that arrives in two files is one record in the result.
Where the two disagree, the later file wins, and `result.revisions` lists
every row that happened to.

## Why the stamp

Every AGS4 file has exactly one TRAN row saying who issued it, to whom, and
when. A merged file is a new transmission, and none of the seventeen input
rows describes it. Leave the stamp out and laterite still merges, but it
tells you:

```
no merge TRAN stamp supplied; TRAN was reconciled like any other group …
```

Pass a `TranStamp` and the merged file says what it is: your issue, your
date, your name on it. The warnings list comes back empty.

## Check the counts

```python
for code in ("LOCA", "GEOL", "SAMP"):
    before = sum(f[code].height for f in map(laterite.read, files) if code in f.groups)
    print(f"{code}  {before:>4} in  {merged[code].height:>4} out")
```

```
LOCA    17 in    17 out
GEOL   114 in   114 out
SAMP   754 in   754 out
```

Seventeen boreholes, 114 strata, 754 samples, all present. A lower count
would mean two inputs held a record with the same key and merge kept one:
what you want when a later issue revises an earlier one, and worth knowing
either way.

## What merge does not do

It does not repair. Validate the result:

```python
report = merged.validate().report
print(f"{report.errors} errors, {report.warnings} warnings")
# 17 errors, 3 warnings
```

Seventeen errors, and they are one fault seventeen times: each input has a
single character outside the permitted set in its LOCA row, and each one
came through. Merging gathers what the inputs say. Cleaning is post 4.

## When two files disagree about a type

The Fugro files all came from one contractor, so they agree. Yours may not.
A common case on a phased project: one contractor declares depths to two
decimal places (`2DP`) and the next to one (`1DP`). `type_clash.py` builds two
such files and merges them three ways.

By default merge stops and tells you, because either answer changes
something:

```python
laterite.merge(phase_1, phase_2)
# MergeConflictError: type conflict in LOCA.LOCA_FDEP: files declared ["1DP", "2DP"]
```

Then you choose:

```python
laterite.merge(phase_1, phase_2, on_type_clash="promote")
# declared 2DP, read as [12.5, 8.5]

laterite.merge(phase_1, phase_2, on_type_clash="widen")
# declared X, read as ['12.50', '8.5']
```

- **`"promote"`** keeps the column a number, at the finer of the two
  precisions. `8.5` is written out as `8.50`. No digit changes, and the
  warnings list records that it happened. This is usually the one you want.
- **`"widen"`** falls back to text and leaves every value exactly as it was
  written. Nothing is touched, but the column is no longer a number, and you
  are back to casting (post 2).

Promote only works when every file declares decimal places. If one says
`2DP` and another says `X`, there is no number to keep, and the column
becomes text under either setting. Watch for this one, because it happens
quietly: the warnings list comes back empty.

```python
quiet = laterite.merge(phase_1, phase_3, on_type_clash="promote")
# declared X, read as ['12.50', 'about 9']
print([warning["kind"] for warning in quiet.warnings])
# []
```

So after any merge where you passed `on_type_clash`, compare the types that
came out with the types that went in. `type_clash.py` has a short function
for it:

```python
print(types_changed(phase_1, laterite.read(quiet.bytes)))
# ['LOCA_FDEP: 2DP -> X']
```

I have raised the missing warning as an issue on laterite. Until it is
settled, the check is yours to make.

A disagreement about a **unit** is different: merge always stops, whatever
you pass. Metres in one file and feet in another cannot be settled by
picking one, so that has to be fixed in the source.

## What you can do now

Everything from the earlier posts, across the whole site:

```python
merged.at("LOCA", ["HKZ3-BH04-SA"]).frames()   # post 1
merged["LLPL"]                                  # post 2: all 53 results
merged.sql("SELECT … FROM LLPL l JOIN GEOL g …")  # post 3
```

## The figure

The seventeen boreholes sit at nine locations: most were drilled two or three
times a few metres apart. Drawing all seventeen shows the same ground again
and again, so the figure keeps the deepest hole at each location and orders
them west to east. That choice is one query on the merged file (`figures.py`
has it in full):

```sql
SELECT LOCA_ID,
       row_number() OVER (
           PARTITION BY regexp_extract(LOCA_ID, 'BH([0-9]+)', 1)
           ORDER BY LOCA_FDEP DESC
       ) AS deepest_first
FROM LOCA
```

## No file to hand?

[laterite-quarry](https://github.com/niko86/laterite-quarry) fetches the
seventeen files:

```
uvx --from "laterite-quarry[rvo]" quarry rvo pull hkz-wfs3-bh
```

_Data: Netherlands Enterprise Agency (RVO) / Fugro. Run on laterite 0.17.1._
