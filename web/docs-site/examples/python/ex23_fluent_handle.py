# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.10.1"]
# ///
"""Docs example — run it with `uv run ex23_fluent_handle.py`, from anywhere.

Everything above the `[start:code]` marker is machinery the page does not
show: the PEP 723 header that makes the file self-installing, the fixture arm
that makes its repo-relative path resolve outside a checkout, and a temp
working directory.

The temp dir is not tidiness. This example ENDS IN A WRITE, and every gate that
runs it does so from the repo root — so without the chdir, `checked.ags` lands
in the working tree on every test run, every doc-output regeneration and every
nightly. The fixture is copied in first so the path inside the shown code still
resolves, which is the same trick `tests/test_docs_snippets.py` uses on the
snippets that write.
"""

import os
import shutil
import tempfile
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

_TMP = Path(tempfile.mkdtemp(prefix="laterite-docs-"))
(_TMP / "examples").mkdir()
shutil.copy(_FIXTURE, _TMP / "examples" / "sample_site.ags")
os.chdir(_TMP)

# --8<-- [start:code]
# what this shows: read → validate → save on one handle, no intermediate variables.
import laterite

saved = laterite.read("examples/sample_site.ags").validate().save("checked.ags")
print(saved, saved.exists())

# Every step returns the same Ags4File, which is what makes the chain a chain —
# `validate()` hands back a handle, not a report.
assert isinstance(
    laterite.read("examples/sample_site.ags").validate(), laterite.Ags4File
)
# --8<-- [end:code]
