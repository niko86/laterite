# Check an AGS file without installing anything

_An optional companion to the series. Every command here was run as written._

Every post in this series has a script, and every script starts the same way:
"first, install laterite". For a lot of people that is where it stops. Which
Python? Which environment? Will it break the other thing?

There is now a way round all of that. [uv](https://docs.astral.sh/uv/) is a
single program that fetches a tool, runs it, and leaves your machine as it
found it. Install uv once, and the rest of this page needs nothing else.

## One command, nothing installed

laterite ships with a command-line tool called `lat`. This checks a file:

```
uvx --from laterite lat validate your_file.ags
```

`uvx` means "run this tool once". The first time, it downloads laterite into
a cache, which takes a few seconds. After that it starts at once. Nothing is
added to your Python, your project or your PATH.

On the Crossrail file from posts 1 and 4:

```
crossrail.ags: 1501 finding(s) — dictionary 4.0.4 (guessed)
┌──────┬──────┬───────┬───────────────────────────────────────────────┐
│ Rule ┆ Line ┆ Group ┆ Description                                   │
╞══════╪══════╪═══════╪═══════════════════════════════════════════════╡
│ 10a  ┆ 236  ┆ ABBR  ┆ Duplicate KEY field combination: LOCA_TYPE|CP │
│ …    ┆ …    ┆ …     ┆ …                                             │
```

(1,501 is the 1,488 errors of post 4 plus 13 warnings.)

Why `--from laterite lat` and not just `uvx lat`? The package is called
`laterite` and the command it provides is called `lat`. `--from` says which
package to fetch; the word after it is the command to run.

## What `lat` can do

The same things the posts do in Python, one line each:

```
# What groups are in this file?
uvx --from laterite lat read your_file.ags

# One group, as a table. Add --csv to pipe it into a spreadsheet.
uvx --from laterite lat read your_file.ags LOCA
uvx --from laterite lat read your_file.ags LOCA --csv > loca.csv

# Apply the safe fixes and write a new file (post 4).
uvx --from laterite lat fix your_file.ags --fix-out fixed.ags

# Merge a folder of files into one (post 5).
uvx --from laterite lat merge folder/*.AGS --out merged.ags \
    --tran-issue 1 --tran-date 2026-11-28 --tran-status Final \
    --tran-producer "Example Ltd" --tran-recipient "Example Client"

# What changed between two issues of a file?
uvx --from laterite lat diff issue_1.ags issue_2.ags

# The rule list, with which ones have a fix.
uvx --from laterite lat rules
```

`fix` on the Crossrail file prints what it did and what is left:

```
applied 1 fix(es) [normalize_crlf] → fixed.ags
fixed.ags: 107 finding(s) remain (not mechanically fixable)
```

Add `--json` to `validate` or `fix` for output a script can read. `lat
<command> --help` lists the options for one command.

`lat validate` exits with status 1 when it finds anything, so it can gate a
delivery in a script or a CI job: a file that fails stops the pipeline.

## Use it every day: install the tool

If you find yourself typing the `uvx` line often, install the command for
good:

```
uv tool install laterite
lat validate your_file.ags
```

Now `lat` is on your PATH. It still lives in its own private environment, so
it cannot clash with anything else. `uv tool upgrade laterite` moves it to
the newest release, and `uv tool list` shows what you have installed.

## Run the series' scripts the same way

Each post's script says how to run it in its first lines:

```
uv run --with laterite one_borehole.py your_file.ags
```

`uv run --with laterite` means: run this script in a throwaway environment
that has laterite in it. No virtual environment to create, nothing to
activate, nothing to clean up afterwards.

You can go one step further and write the requirement into the script
itself, as a comment at the top:

```python
# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite"]
# ///
import sys

import laterite

ags = laterite.read(sys.argv[1])
print(ags["LOCA"].height, "holes in", len(ags.groups), "groups")
```

Then the script carries its own instructions, and anyone with uv can run it:

```
uv run count_holes.py your_file.ags
# 12 holes in 15 groups
```

That is a script you can email to a colleague.

## Pin a version when it matters

For a check that has to give the same answer next year, name the release:

```
uvx --from laterite==0.17.1 lat validate your_file.ags
```

## The same for the data fetcher

[laterite-quarry](https://github.com/niko86/laterite-quarry), which fetches
the public files this series uses, works the same way. Its command is
`quarry`:

```
uvx --from laterite-quarry quarry bgs pull farringdon --bbox -0.108,51.518,-0.102,51.522
uvx --from "laterite-quarry[rvo]" quarry rvo pull hkz-wfs3-bh
```

The `[rvo]` in the second line is an optional extra: the Dutch files arrive
as `.7z` archives, and the extra adds what is needed to open them.

## No uv?

Everything here also works the traditional way. `pip install laterite` gives
you the `lat` command and the Python library together.
