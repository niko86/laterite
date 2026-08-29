# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite[compat]==0.12.0"]
# ///
"""Docs example — run it with `uv run ex11_compat.py`, from anywhere.

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
# EX11 — the python-ags4 drop-in: laterite.compat is a faithful AGS4_to_dataframe shim.
from laterite import compat as AGS4

result = AGS4.AGS4_to_dataframe("examples/sample_site.ags")

# python-ags4 returns a (tables, headings) 2-tuple; tables maps group -> pandas DataFrame.
print(type(result), list(result[0])[:5])
print(result[0]["LOCA"].shape)

assert isinstance(result, tuple) and len(result) == 2 and "LOCA" in result[0]
# --8<-- [end:code]
