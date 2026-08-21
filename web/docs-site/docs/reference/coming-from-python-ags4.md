# Coming from python-ags4

You have code built on
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) and you
are deciding whether to move. This page answers the questions that decision turns
on: what the swap costs, what is deliberately not mirrored, which upstream version
you are getting, and what your CI will do afterwards.

For the task-shaped version — _how do I do X with the drop-in_ — see
[Drop-in for python-ags4](../cookbook/compat.md).

## One token changes

<!-- doc-code: skip — installs packages — a gate that ran it would rewrite its own environment -->
```bash
pip install laterite[compat]
```

```python
from python_ags4 import AGS4          # before
from laterite.compat import AGS4      # after
```

That is structural rather than approximate. `laterite.compat` is a **package**
whose submodule names mirror upstream's, so every import shape real python-ags4
code writes has a real equivalent instead of a flattened stand-in:

```python
from laterite.compat import AGS4               # the submodule
from laterite.compat.AGS4 import AGS4Error     # third-party code does this
from laterite.compat.check import get_TRAN_AGS
from laterite.compat.utils import get_DICT_table_from_json_file
from laterite.compat.data import load_test_data
```

`AGS4_to_dataframe` returns the same `(tables, headings)` 2-tuple of pandas
frames it always did. The import swap _is_ the migration — no call-site edits.
[The cookbook page](../cookbook/compat.md) has it running, and covers the pandas
dtype question and the `[compat]` extra.

## What maps, and what deliberately does not

Four submodules are mirrored: `AGS4`, `check`, `utils`, `data`.

**`ags4_cli` is not.** laterite ships [`lat`](../surfaces/cli.md) instead — a
standalone binary with its own JSON/NDJSON output shapes, not a command-level
mirror of upstream's Click CLI. A deliberate divergence, not a gap left open.

Everything else upstream exposes is either mirrored or listed **by name, with a
reason**, in `compat-surface-gaps.json` at the repo root. A scheduled CI job
(`check_dropin_surface.py`) compares the two public APIs callable-by-callable and
enforces that file by identity: a new gap fails, and so does an entry that has
stopped being a gap. It runs on a schedule rather than per-PR because the thing
it is watching for is _upstream_ moving — python-ags4 adding a public function no
existing test calls, which every other gate would sail straight past.

That file is the authority, which is why this page does not copy it: a copy would
be right until the next upstream release and quietly wrong afterwards.

## There will never be a `python_ags4` import name

Stated rather than left open, because it is the obvious next thing to ask for:
laterite will not ship a top-level `python_ags4` distribution. Not now, not at
1.0.

Two independent reasons, either sufficient.

Inside the wheel it would collide with the real library in
`site-packages/python_ags4/` — two distributions claiming one import path, with
install order deciding which one wins. That is a packaging hazard, not a feature.

As a separate distribution it would break this repo's own parity oracle. The
oracle installs the **genuine** python-ags4 and runs its test suite against
`laterite.compat` through a `sys.modules` shim. A laterite-published `python_ags4`
would fight the real one for the same directory, and the comparison that keeps
`compat` honest is the first thing it would take out.

## Which python-ags4 version you are getting

`compat` is calibrated against one specific upstream release, and it says which:

```python
from laterite import compat

compat.PYTHON_AGS4_COMPAT   # the upstream version this surface is pinned to
compat.__version__          # laterite's version + a PEP 440 local segment naming that pin
```

The local segment exists so the identity is honest in a log line or a bug report
— it names both packages at once. `test_version_faithful.py` asserts the two
agree, and the release stamper is configured to move the laterite prefix only, so
the upstream pin does not drift when laterite cuts a version.

## Exit codes, and what your CI will do

The short version: **through the drop-in nothing changes shape, and on the native
surface no warning fails a run.** Where the two tools genuinely disagree at error
level, that is [a catalogued divergence](divergences.md) rather than a surprise.

Two paths, different on purpose.

**Through `compat.check_file`.** It runs the informational tier and _not_ the
warning tier, because informational is what python-ags4 emits. An unrecognised
`TRAN_AGS`, for example, comes back as the same `FYI` key python-ags4 produces
rather than as a laterite warning. The returned dict carries python-ags4's keys
— rule keys plus `Metadata` / `Summary of data` / `General` — so `json.dumps` on
it matches. Nothing new appears in it.

**Through the native surface** — [`lat`](../surfaces/cli.md) or
`laterite.validate()` — you get laterite's own opinions, including a warning tier
python-ags4 has no equivalent of. Those warnings are **shown by default and do
not decide the verdict**: a file whose only blemish is a warning exits `0`. Only
errors fail a run. [Severity tiers](../concepts/severity-tiers.md) has the two
dials and the full table.

!!! tip "If you want warnings to fail CI, ask for it"

    `lat validate "$f" --warnings-as-errors` — or `warnings_as_errors=True` in
    Python — is the compiler's `-Werror`. `--no-warnings` is the opposite dial:
    errors only, on screen and in the verdict. The two contradict each other, so
    passing both is rejected rather than silently resolved.

## What differs, and why

laterite is a clean-room re-implementation. Two independent implementations of
one specification will disagree, so every disagreement is written down rather
than smoothed over — including the ones where laterite was the one in the wrong.

The full user-facing list is
[Where laterite and python-ags4 differ](divergences.md), generated from the same
source as the repo's catalogue so a record cannot be resolved in one and stay
live in the other. It groups them by what actually happened, which is the useful
distinction when you are deciding:

- **Deliberate differences** — laterite refuses an AGS 3.x file outright, where
  python-ags4 silently validates it against 4.1.1.
- **Where both depart from the written spec** — Rule 1's literal "entirely
  ASCII" is the clearest: laterite matches python-ags4, and the standard's text
  is the outlier.
- **Where laterite changed to match python-ags4** — its own false negatives,
  which the comparison caught and closed, kept on the record afterwards.
- **Checks laterite adds** — findings python-ags4 has no equivalent of. This is
  where the warning tier lives, which is what the section above is about.

For the rule-by-rule detail — parser strictness, encoding, error-handling
philosophy, the function-level API map and the residual parity-test failures —
[`COMPAT.md`](https://github.com/niko86/laterite/blob/main/COMPAT.md) in the repo
is the long form.

## When you are ready to stop being a migrant

The drop-in is a bridge, not the destination. [`laterite.read()`](../learn/read.md)
gives you born-typed frames directly — no `.cast()`, no `pd.to_numeric` — and the
base install is polars + duckdb with no pandas at all. See
[Dependency shape](../concepts/dependency-shape.md).
