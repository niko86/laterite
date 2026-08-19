# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.11.0"]
# ///
"""Docs example — run it with `uv run ex08_certify.py`, from anywhere.

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
# what this shows: the certify fast-path — a fresh .ags.idx cert lets validate() skip the rule engine.
import shutil
import tempfile
from pathlib import Path

import laterite

with tempfile.TemporaryDirectory() as tmp:
    tmp_path = str(Path(tmp) / "site.ags")
    shutil.copy("examples/sample_site.ags", tmp_path)

    # certify() runs the validation itself and mints <path>.ags.idx for an error-clean file.
    idx = laterite.read(tmp_path).certify()

    # re-reading with the fresh cert lets validate() answer without running the rule engine.
    ags = laterite.read(tmp_path, index=str(idx)).validate(warnings=False)

    print(ags.report.certified, ags.report.resolution)
    # `certified` says the ENGINE was skipped; `resolution` still says which dictionary judged it.
    assert ags.report.certified
# --8<-- [end:code]
