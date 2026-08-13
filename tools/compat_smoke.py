"""Smoke test for an isolated ``pip install laterite[compat]``: the drop-in
default is **pyarrow-free**. pyarrow must NOT be importable, and the round-trip
the README advertises — ``AGS4_to_dataframe`` then ``dataframe_to_AGS4`` — must
work, returning the python-ags4 shape with object-dtype columns and emitting a
file that reads back. That proves the DuckDB ``.df()`` fallback path (the only
environment that exercises it, since every workspace job installs pyarrow via
``--all-extras``).
Run by nightly.yml's wheel-smoke job in a clean ``[compat]`` venv.

**Both halves, deliberately.** Until 2026-08-12 this covered only reads, so the
pyarrow-free *write* path had never run anywhere — and the gate's name implied
otherwise, which is the failure mode laterite#295 catalogues. `dataframe_to_AGS4`
is the half that hands a frame across the FFI boundary, and it is the call that
faults intermittently *with* pyarrow present (laterite#294). So this leg is also
the discriminator for that bug: if it stays green here while #294 keeps firing in
pyarrow-bearing environments, that is evidence the fault needs pyarrow-produced
buffers; if it ever fires here, that hypothesis is dead and the fault is in the
frame handoff itself. Read the failure accordingly — a crash here is a finding,
not flake.
"""

import importlib.util
import io
import tempfile
from pathlib import Path

import pandas as pd
from laterite import compat as AGS4

# The module SHAPES have to survive packaging too, not just the flat namespace:
# a missing `__init__.py` in the built wheel would leave every dev-tree import
# test green and break `from laterite.compat.AGS4 import …` for real users.
from laterite.compat.AGS4 import AGS4Error  # noqa: F401
from laterite.compat.data import load_test_data
from laterite.compat.utils import get_DICT_table_from_json_file  # noqa: F401

# The invariant: `[compat]` alone does not pull pyarrow.
assert importlib.util.find_spec("pyarrow") is None, (
    "pyarrow is importable in a bare [compat] install — it must be an opt-in "
    "accelerator ([compat,pyarrow]/[all]/[pyarrow]), not a [compat] dependency."
)

src = (
    '"GROUP","PROJ"\r\n'
    '"HEADING","PROJ_ID"\r\n'
    '"UNIT",""\r\n'
    '"TYPE","ID"\r\n'
    '"DATA","P1"\r\n'
)
tables, _ = AGS4.AGS4_to_dataframe(io.StringIO(src))
proj = tables["PROJ"]
assert list(proj.columns) == ["HEADING", "PROJ_ID"], proj.columns
assert all(str(d) == "object" for d in proj.dtypes), list(proj.dtypes)

# The shipped sample must be readable through the same pyarrow-free path.
sample, sample_headings = load_test_data()
assert "LOCA" in sample, sorted(sample)

# …and writable back out through it. The 8-group sample is used rather than the
# one-column PROJ above because the emit walks every table it is handed, so a
# single-group round-trip would leave most of the path unexercised.
with tempfile.TemporaryDirectory() as tmp:
    out = Path(tmp) / "round-trip.ags"
    AGS4.dataframe_to_AGS4(sample, sample_headings, str(out))
    assert out.stat().st_size > 0, "dataframe_to_AGS4 wrote an empty file"
    reread, _ = AGS4.AGS4_to_dataframe(str(out))

# Round-trip identity at group level: what went out comes back. Not asserting
# byte-for-byte equality with the source — `emit` is spec-correct rather than
# input-preserving, which is a different guarantee (and one the Rust suite owns).
assert sorted(reread) == sorted(sample), (sorted(reread), sorted(sample))
print(
    f"[compat] OK — pandas {pd.__version__}, pyarrow-free, "
    "object-dtype drop-in via DuckDB verified, "
    f"round-trip re-read {len(reread)} groups"
)
