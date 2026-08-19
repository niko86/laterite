# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — born-typed reads. `uv run ex01_read_typed.py` from anywhere.

Everything above the `[start:code]` marker is machinery the page does not show:
the PEP 723 header that makes the file self-installing, and the fixture arm that
makes its repo-relative path resolve outside a checkout. The pin is a CLAIM —
this example is known to run at that laterite — and `test_version_faithful.py`
holds it to the shipped version, so it cannot quietly rot into a lie.
"""

import urllib.request
from pathlib import Path

_FIXTURE = Path("examples/sample_site.ags")
_RAW = "https://raw.githubusercontent.com/niko86/laterite/main/examples/sample_site.ags"
if not _FIXTURE.exists():
    # Cold only for a reader running this outside the repo: in a checkout (and in
    # CI, cwd = repo root) the file is already there and this arm never executes,
    # so the gates stay offline. Fetching it — rather than rewriting the example
    # to an absolute path — is what keeps the text on the page the text you would
    # actually type.
    _FIXTURE.parent.mkdir(parents=True, exist_ok=True)
    _FIXTURE.write_bytes(urllib.request.urlopen(_RAW, timeout=30).read())

# --8<-- [start:code]
import laterite

# Read an AGS4 file. `read` takes a path, or text=… / data=… (the three doors).
ags = laterite.read("examples/sample_site.ags")

# A group comes back as a born-typed polars frame — the dtype *is* the TYPE row.
loca = ags["LOCA"]
print(loca.select("LOCA_ID", "LOCA_NATE", "LOCA_GL").head(2))
print({h: str(loca[h].dtype) for h in ("LOCA_ID", "LOCA_NATE", "LOCA_GL")})

assert str(loca["LOCA_GL"].dtype) == "Float64"  # 2DP  → Float64 (no manual cast)
assert str(loca["LOCA_NATE"].dtype) == "Float64"  # 2DP → Float64
assert str(loca["LOCA_ID"].dtype) == "String"  # ID → String
# --8<-- [end:code]
