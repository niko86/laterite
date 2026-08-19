# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — run it with `uv run ex18_severity_tiers.py`, from anywhere.

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

# The page's subject is a file whose ONLY blemish is an out-of-range TRAN_AGS
# edition, so mint one from the clean fixture. Bytes, not text: the fixture is
# CRLF (AGS4 Rule 2a) and a read_text()/write_text() round trip would normalise
# it, burying the one interesting warning under a Rule 2a per line. The edition
# string is unique in the fixture, so this cannot hit a date or a DICT row.
_EDITION = Path("examples/sample_site_edition.ags")
_EDITION.write_bytes(_FIXTURE.read_bytes().replace(b',"4.1.1",', b',"4.9.9",'))

# --8<-- [start:code]
import laterite

# Default: errors + warnings. The unrecognised-edition warning shows.
print(laterite.read("examples/sample_site_edition.ags").validate().report.count)

# Errors only — the warning is gone, the verdict is clean.
print(
    laterite.read("examples/sample_site_edition.ags")
    .validate(warnings=False)
    .report.count
)

assert laterite.read("examples/sample_site_edition.ags").validate().report.count == 1
assert (
    laterite.read("examples/sample_site_edition.ags")
    .validate(warnings=False)
    .report.count
    == 0
)
# --8<-- [end:code]
