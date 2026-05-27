"""F2b-6a: catch dictionary-edit drift in the committed `.pyi` stubs.

The `_laterite_native.pyi` stub file is generated from
`rust-packages/ags5-core/data/ags5_dictionary.json` by
`tools/generate_pyi.py`. If the dictionary is edited (a new heading,
a renamed group, a precision change) the stub falls out of sync with
the compiled Rust extension, which means IDE autocomplete starts
showing the wrong fields.

This test re-runs the generator and asserts byte-equality with the
committed file, so the CI gate fails loud rather than letting the
mismatch ship.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

# Test lives under packages/laterite/tests/ — repo root is three
# parents up (tests → laterite → packages → root).
_REPO_ROOT = Path(__file__).resolve().parents[3]
_PYI_PATH = (
    _REPO_ROOT
    / "packages"
    / "laterite"
    / "python"
    / "laterite"
    / "_laterite_native.pyi"
)
_GENERATOR = _REPO_ROOT / "tools" / "generate_pyi.py"


def _load_generator():
    """Import `tools/generate_pyi.py` as a module (the file isn't on
    `sys.path` by default)."""
    spec = importlib.util.spec_from_file_location("_gen_pyi", _GENERATOR)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["_gen_pyi"] = mod
    spec.loader.exec_module(mod)
    return mod


def test_pyi_file_in_sync_with_generator() -> None:
    """`tools/generate_pyi.py` should regenerate the committed
    `.pyi` byte-for-byte. Drift means the dictionary moved and someone
    forgot to re-run the generator."""
    gen = _load_generator()
    expected = gen.generate()
    actual = _PYI_PATH.read_text(encoding="utf-8")
    assert actual == expected, (
        f"{_PYI_PATH} is out of date — regenerate with "
        "`uv run python tools/generate_pyi.py`"
    )
