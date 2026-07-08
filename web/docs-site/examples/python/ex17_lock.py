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
    sealed = lock(src, password="correct horse battery staple", dest=tmp / "site.ags.zst.age")
    restored = unlock(sealed, password="correct horse battery staple", dest=tmp / "restored.ags")

    print(f"sealed: {sealed.name}")
    print(f"round-trip byte-identical: {restored.read_bytes() == src.read_bytes()}")

    # Encryption is transparent to the payload: unlock restores the exact bytes.
    assert restored.read_bytes() == src.read_bytes()
    assert sealed.suffixes[-2:] == [".zst", ".age"]
