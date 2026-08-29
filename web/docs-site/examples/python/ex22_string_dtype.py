# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite[compat,pyarrow]==0.12.0"]
# ///
"""Docs example — run it with `uv run ex22_string_dtype.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing, and the fixture
arm that makes its repo-relative path resolve outside a checkout.

The header asks for `[compat,pyarrow]` rather than bare `laterite`, because
that IS the page's subject: `string_dtype="string"` needs the pyarrow
accelerator and raises an actionable error without it.

Sets a PROCESS-WIDE default at the end, deliberately — that is what the page
documents. It is contained here because every docs example runs as its own
subprocess; the in-process snippet gate restores it explicitly (#328).
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
# what this shows: the two pandas string dtypes compat can hand you, and how to pick.
from laterite import compat as AGS4

# object dtype (numpy) — today's python-ags4 baseline, the default
tables, _ = AGS4.AGS4_to_dataframe("examples/sample_site.ags")
print(tables["LOCA"]["LOCA_ID"].dtype)

# string dtype (pandas' Arrow-backed str) — needs [compat,pyarrow]
tables, _ = AGS4.AGS4_to_dataframe("examples/sample_site.ags", string_dtype="string")
print(tables["LOCA"]["LOCA_ID"].dtype)

# …or set it once, for the process (or export LATERITE_COMPAT_STRING_DTYPE).
AGS4.set_string_dtype("string")
tables, _ = AGS4.AGS4_to_dataframe("examples/sample_site.ags")
print(tables["LOCA"]["LOCA_ID"].dtype)
# --8<-- [end:code]
