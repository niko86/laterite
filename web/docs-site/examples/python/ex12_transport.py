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
