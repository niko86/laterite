# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.10.1"]
# ///
"""Docs example — run it with `uv run ex12_transport.py`, from anywhere.

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
# what this shows: zstd transport round-trip — pack a file to .zst, unpack it, prove byte-identical + smaller.
import shutil
import tempfile
from pathlib import Path

from laterite.transport import pack, unpack

with tempfile.TemporaryDirectory() as tmp:
    tmp = Path(tmp)
    src = tmp / "site.ags"
    shutil.copy("examples/sample_site.ags", src)

    packed = pack(src, dest=tmp / "site.ags.zst")
    restored = unpack(packed, dest=tmp / "restored.ags")

    original_bytes = src.read_bytes()
    restored_bytes = restored.read_bytes()
    original_size = len(original_bytes)
    compressed_size = packed.stat().st_size

    print(f"original:   {original_size} bytes")
    print(f"compressed: {compressed_size} bytes")
    print(f"round-trip byte-identical: {restored_bytes == original_bytes}")

    assert restored_bytes == original_bytes
    assert compressed_size < original_size
# --8<-- [end:code]
