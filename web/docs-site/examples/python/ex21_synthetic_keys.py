# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite[pyarrow]==0.12.0"]
# ///
"""Docs example — run it with `uv run ex21_synthetic_keys.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing, and the fixture
arm that makes its repo-relative path resolve outside a checkout.

Prints the COLUMN LISTS rather than the frames: which columns appear is the
whole lesson here, and two full frames would bury it.

`[pyarrow]` is here for the closing `rel.pl()`, not for anything laterite does:
the relation `.sql()` hands back is DuckDB's, and its `.pl()` imports pyarrow.
See `ex06_sql_join.py`, which carries the same extra for the same reason.
"""

import urllib.request
from pathlib import Path

_FIXTURE = Path("examples/sample_site.ags")
_RAW = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"
if not _FIXTURE.exists():
    # Cold only for a reader running this outside the repo: in a checkout (and in
    # CI, cwd = repo root) the file is already there and this arm never executes,
    # so the gates stay offline.
    _FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    _FIXTURE.write_bytes(urllib.request.urlopen(_RAW, timeout=30).read())

# --8<-- [start:code]
# what this shows: frames hide the synthetic keys; ask for them per call or per handle.
import laterite as L

ags = L.read("examples/sample_site.ags")

print(ags["LOCA"].columns)  # AGS columns only
print(ags.table("LOCA", keys=True).columns)  # + _id / _parent_id
print(L.read("examples/sample_site.ags", keys=True)["LOCA"].columns)  # handle-wide

# The join needs no opt-in either way — the engine always has the keys, whatever
# the frames were asked to show.
rel = ags.sql(
    "SELECT s.SAMP_ID, l.LOCA_ID FROM SAMP s JOIN LOCA l ON s._parent_id = l._id"
)
print(rel.pl().height)

assert "_id" not in ags["LOCA"].columns
assert "_id" in ags.table("LOCA", keys=True).columns
# --8<-- [end:code]
