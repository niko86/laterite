# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.12.0"]
# ///
"""Docs example — run it with `uv run ex02_validate.py`, from anywhere.

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
import laterite

# Validate a file. read(...).validate() returns the Ags4File (so it chains);
# the Report is on the .report property.
ags = laterite.read("examples/sample_site.ags").validate()
r = ags.report
print(
    f"is_valid={r.is_valid} count={r.count} "
    f"dict_version={r.dict_version!r} resolution={r.resolution!r}"
)

assert r.is_valid is True
assert r.count == 0
assert r.dict_version == "4.1.1"  # auto-selected from TRAN_AGS
# --8<-- [end:code]
