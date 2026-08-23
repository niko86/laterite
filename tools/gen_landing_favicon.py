"""Generate the landing page's /favicon.ico from the shipped icon set.

The landing serves real PNG icons (byte-identical copies of the app's
`web/public/icons/` set — `web/landing/icons.test.ts` pins that identity, which
is those copies' provenance), but agents and older tooling probe `/favicon.ico`
blind, so the root wants a real ICO (#598). This script is that file's
provenance: a multi-size ICO resampled from the 512px brand asset — the
original mark, not an already-downsized derivative — committed alongside it
rather than regenerated at build time so the deploy stays a plain static copy.

Run: uv run --no-project --with pillow python tools/gen_landing_favicon.py
`--check` compares DECODED PIXELS, not encoded bytes: the sibling `gen_*`
gates byte-compare, but this output rides Pillow's PNG encoder, and a fresh
`--with pillow` on the runner is free to encode the same pixels differently —
byte-compare went red on its very first CI run for exactly that. Decoding is
lossless, so pixel equality still catches the drift that matters (the source
moved, the sizes changed) while the encoder stays free to vary.
"""

import io
import sys
from pathlib import Path

from PIL import IcoImagePlugin, Image

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "assets" / "laterite-icon-512.png"
OUT = ROOT / "web" / "landing" / "public" / "favicon.ico"
SIZES = [(16, 16), (32, 32), (48, 48)]


def render() -> bytes:
    buf = io.BytesIO()
    Image.open(SRC).convert("RGBA").save(buf, format="ICO", sizes=SIZES)
    return buf.getvalue()


def frames(ico: bytes) -> list[bytes]:
    """Each embedded size's raw RGBA pixels, smallest to largest."""
    out = []
    for size in SIZES:
        img = Image.open(io.BytesIO(ico))
        # Frame selection is the ICO plugin's own idiom: assigning `size`
        # picks the embedded image. The narrowing is load-bearing twice over —
        # only IcoImageFile types the setter (the base ImageFile's `size` is
        # read-only, which is what ty rightly rejected), and a non-ICO byte
        # stream should fail HERE, not decode as something else.
        if not isinstance(img, IcoImagePlugin.IcoImageFile):
            raise OSError(f"not an ICO stream (decoded as {type(img).__name__})")
        img.size = size
        out.append(img.convert("RGBA").tobytes())
    return out


def main() -> None:
    if "--check" in sys.argv[1:]:
        # One file in scope, and said out loud: this gate checks favicon.ico
        # only — the PNG icon copies are held to the app's set by
        # web/landing/icons.test.ts, not here.
        committed = OUT.read_bytes() if OUT.exists() else b""
        try:
            same = committed and frames(committed) == frames(render())
        except OSError:
            same = False
        if not same:
            print(
                f"gen_landing_favicon: {OUT.relative_to(ROOT)} does not match "
                f"a fresh render from {SRC.relative_to(ROOT)} — regenerate "
                "with: uv run --no-project --with pillow python "
                "tools/gen_landing_favicon.py"
            )
            raise SystemExit(1)
        print(
            f"gen_landing_favicon: OK — {OUT.relative_to(ROOT)} matches "
            f"{SRC.relative_to(ROOT)} pixel-for-pixel at "
            f"{'/'.join(str(w) for w, _ in SIZES)} (PNG icons are "
            "icons.test.ts's scope, not this gate's)"
        )
        return
    OUT.write_bytes(render())
    print(f"wrote {OUT.relative_to(ROOT)} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
