"""Copy the brand mark into the built site (#401).

The masthead lockup needs the laterite mark, and the tab needs the icon. Both
already exist once, at the repo root under `assets/` — which is where every
other surface reads them from, and where a redraw would land.

Generated at build time rather than committed into `docs/` for the reason the
group catalogue is: a second copy of a binary is a thing that goes stale
silently. Nothing renders a diff of two PNGs in review, so a mark updated at the
root and not here would sit wrong on the docs for as long as it took somebody to
notice by eye.

`mkdocs.yml` points `theme.logo` / `theme.favicon` at the paths written here.
Those are resolved as URLs at render time and never checked against the source
tree, so a rename at the root would produce a silently broken image rather than
a build error — hence the explicit existence check below, which turns it into
one.
"""

from __future__ import annotations

from pathlib import Path

import mkdocs_gen_files

# web/docs-site/scripts/gen_brand.py -> repo root
_ASSETS = Path(__file__).resolve().parents[3] / "assets"

# (source at the repo root, destination inside the built site)
_MARKS = [
    ("laterite.svg", "assets/laterite.svg"),
    ("laterite-icon-128.png", "assets/laterite-icon-128.png"),
]

for src_name, dest in _MARKS:
    src = _ASSETS / src_name
    if not src.is_file():
        raise FileNotFoundError(
            f"brand asset {src} is missing — mkdocs.yml points theme.logo / "
            f"theme.favicon at {dest}, and a missing source renders as a broken "
            f"image rather than failing the build."
        )
    with mkdocs_gen_files.open(dest, "wb") as fd:
        fd.write(src.read_bytes())
