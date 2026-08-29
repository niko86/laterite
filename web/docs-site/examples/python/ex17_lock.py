# /// script
# requires-python = ">=3.12"
# dependencies = ["laterite==0.12.0"]
# ///
"""Docs example — run it with `uv run ex17_lock.py`, from anywhere.

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
# what this shows: age-encrypted transport round-trip — lock a file with a passphrase, unlock it, prove byte-identical.
import shutil
import tempfile
from pathlib import Path

from laterite.transport import lock, unlock

with tempfile.TemporaryDirectory() as tmp:
    tmp = Path(tmp)
    src = tmp / "site.ags"
    shutil.copy("examples/sample_site.ags", src)

    # lock = zstd pack + age passphrase encrypt (scrypt KDF + ChaCha20-Poly1305).
    # Omit dest and it writes <src>.zst.age alongside the source.
    sealed = lock(
        src, password="correct horse battery staple", dest=tmp / "site.ags.zst.age"
    )
    restored = unlock(
        sealed, password="correct horse battery staple", dest=tmp / "restored.ags"
    )

    print(f"sealed: {sealed.name}")
    print(f"round-trip byte-identical: {restored.read_bytes() == src.read_bytes()}")

    # Encryption is transparent to the payload: unlock restores the exact bytes.
    assert restored.read_bytes() == src.read_bytes()
    assert sealed.suffixes[-2:] == [".zst", ".age"]
# --8<-- [end:code]
