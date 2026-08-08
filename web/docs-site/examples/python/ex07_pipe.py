# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.10.1"]
# ///
"""Docs example — run it with `uv run ex07_pipe.py`, from anywhere.

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
# what this shows: .pipe(fn, *args) splices your own step into the chain — fn receives the handle as its first arg — on both Ags4File and AgsQuery.
import laterite

ags = laterite.read("examples/sample_site.ags")

# On an Ags4File: fn(self, *args) — the file handle is passed first, your extra args follow.
out = ags.pipe(lambda a, n: a.groups[:n], 3)
print("first 3 group codes:", out)

# On an AgsQuery: same contract — the query handle is passed in, you return whatever you like.
height = ags.query("SELECT * FROM LOCA").pipe(lambda q: q.frame().height)
print("LOCA row count via pipe:", height)

# .pipe returns your function's result verbatim, and passes the object as the first argument.
assert out == ["PROJ", "TRAN", "UNIT"]
assert ags.pipe(lambda a: a is ags) is True
assert height == 14
assert ags.query("SELECT * FROM LOCA").pipe(lambda q: q is not None) is True
# --8<-- [end:code]
