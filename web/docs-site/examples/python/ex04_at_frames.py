# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.10.1"]
# ///
"""Docs example — run it with `uv run ex04_at_frames.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing,
and the fixture arm that makes its repo-relative path resolve outside a
checkout.
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
# what this shows: materialise a borehole's full record set as a dict of polars frames via .at(..).frames()
import laterite

ags = laterite.read("examples/sample_site.ags")
frames = ags.at("LOCA", ["BH01"]).frames()

# frames is a dict {group_code: polars frame}; pull one out by code (NOT q["SAMP"])
print(sorted(frames))
print(frames["SAMP"].height)

assert isinstance(frames, dict) and "SAMP" in frames and frames["SAMP"].height >= 1
# --8<-- [end:code]
