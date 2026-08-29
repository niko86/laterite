# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.12.0"]
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

SRC = "examples/sample_site_edition.ags"

# Default: errors + warnings. The unrecognised-edition warning shows — and the
# file still PASSES, because only errors decide the verdict.
shown = laterite.read(SRC).validate().report
print(shown.count, shown.warnings, shown.is_valid)

# --no-warnings' twin: the warning is hidden. The verdict never moved.
hidden = laterite.read(SRC).validate(warnings=False).report
print(hidden.count, hidden.warnings, hidden.is_valid)

# The other dial: same report, opposite verdict.
strict = laterite.read(SRC).validate(warnings_as_errors=True).report
print(strict.count, strict.warnings, strict.is_valid)

assert (shown.count, shown.is_valid) == (1, True)
assert (hidden.count, hidden.is_valid) == (0, True)
assert (strict.count, strict.is_valid) == (1, False)
# --8<-- [end:code]
