"""Every surface that sets a global allocator must keep mimalloc pinned to v2.

THE FAULT. mimalloc v3 co-resident with a SECOND mimalloc in one process hands
out overlapping memory (microsoft/mimalloc#1327). Our cdylib is loaded into
processes that already have one: pyarrow bundles mimalloc as its default pool,
and CPython 3.14 vendors its own. `import pyarrow; import laterite` was enough
to corrupt pyarrow's buffers with no laterite call at all.

IT HAS TWO FACES, and that is why this file exists rather than one more runtime
assertion. #294 is the loud one — corrupted buffers, a UnicodeDecodeError or a
SIGSEGV. #297 is the quiet one: `tests/test_docs_snippets.py` selected on its
own never terminated, burning CPU with no output, and was written up as a
separate ordering bug because it had a different shape. It was the same fault.
A corrupted free list can spin as easily as it can crash, and which one you get
depends on allocation order — which is precisely why the module was fine with
its siblings collected and not on its own.

WHY A MANIFEST GATE. The runtime guard
(`packages/laterite/tests/test_public_api_surface.py::test_pyarrow_buffers_survive_a_coresident_native_module`)
can only go red on pyarrow 24.0.0-25.0.0; Arrow fixed their side in 25.0.1, and
our lock has been past that since. It says so itself. So on today's environment
that guard passes no matter what the pin says, and the pin — the half we
actually control, and the half that protects a user whose pyarrow we do not
choose — has nothing watching it. This does, off the source, on any pyarrow.

The crates are DISCOVERED, not listed: a `#[global_allocator]` can only be set
by a final artifact, so a new binding or binary that adds one is covered here
the day it is added rather than the day someone remembers this file.
"""

from __future__ import annotations

import tomllib
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
CRATES = REPO / "rust-packages"

#: The feature that selects the v2 line. Without it libmimalloc-sys resolves
#: v3 from 0.1.47 on, so an omitted feature is not a neutral default — it is
#: the faulting configuration, reached by deleting three characters.
PIN = "v2"


def _sets_a_global_allocator(crate: Path) -> bool:
    return any(
        "#[global_allocator]" in f.read_text(encoding="utf-8")
        for f in (crate / "src").rglob("*.rs")
    )


ALLOCATOR_CRATES = sorted(
    c
    for c in CRATES.iterdir()
    if (c / "Cargo.toml").is_file() and _sets_a_global_allocator(c)
)


def test_the_discovery_found_the_surfaces() -> None:
    """A discovery that silently finds nothing is a green tick over no coverage."""
    assert ALLOCATOR_CRATES, (
        "no crate under rust-packages/ sets a #[global_allocator] — either the "
        "allocator was dropped everywhere, or this file's discovery has rotted."
    )


@pytest.mark.parametrize("crate", ALLOCATOR_CRATES, ids=lambda c: c.name)
def test_mimalloc_stays_on_v2(crate: Path) -> None:
    manifest = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))
    dep = manifest.get("dependencies", {}).get("mimalloc")
    if dep is None:
        pytest.skip(f"{crate.name} sets a global allocator, but not mimalloc's")
    features = dep.get("features", []) if isinstance(dep, dict) else []
    assert PIN in features, (
        f'{crate.name}\'s mimalloc dependency has lost `features = ["{PIN}"]`, '
        "which resolves v3 and reintroduces #294/#297: two co-resident mimallocs "
        "in one process overlap their heaps. Do not drop it because pyarrow "
        ">=25.0.1 fixed their side — we do not control which pyarrow a user "
        "installs, and CPython 3.14 vendors a second allocator regardless."
    )
