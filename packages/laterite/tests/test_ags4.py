"""Round-trip smoke tests for the AGS4 codec.

Runs against a committed synthetic multi-group fixture
(`tools/gen_synthetic_fixture.py` — deterministic, no real project data), so
this exercises real-file round-trip fidelity everywhere including CI. (The old
23 MB `examples/output/large.ags` was git-ignored and absent in CI, so this
test silently skipped there.)
"""

from __future__ import annotations

from pathlib import Path

import pytest

# Committed synthetic fixture: 9 groups across the
# LOCA->SAMP->TREG->TRET->TREL chain plus GEOL/CORE/LLPL. Always present,
# so the round-trip runs in CI.
_FIXTURE = (
    Path(__file__).resolve().parents[3]
    / "rust-packages" / "laterite-ags4-validator" / "tests" / "fixtures"
    / "synthetic_multigroup.ags"
)


@pytest.fixture(scope="module")
def large_ags() -> Path:
    return _FIXTURE


def test_ags4_to_db_then_db_to_ags4_preserves_row_counts(
    large_ags: Path, tmp_path: Path,
) -> None:
    """Every group's row count survives the AGS4 -> .ags5db -> AGS4 round-trip."""
    from laterite.ags5db import convert, export
    from python_ags4 import AGS4

    db = tmp_path / "round.ags5db"
    out_ags = tmp_path / "out.ags"
    convert(large_ags, db)
    export(db, out_ags)

    orig_tables, _ = AGS4.AGS4_to_dataframe(str(large_ags))
    out_tables, _ = AGS4.AGS4_to_dataframe(str(out_ags))

    # Both files should have exactly the same set of group codes
    assert set(orig_tables.keys()) == set(out_tables.keys())

    # And each group should have the same data row count (drop the UNIT/TYPE rows)
    diffs = []
    for code in orig_tables:
        orig_rows = max(0, len(orig_tables[code]) - 2)
        out_rows = max(0, len(out_tables[code]) - 2)
        if orig_rows != out_rows:
            diffs.append((code, orig_rows, out_rows))

    assert diffs == [], f"row count diffs after round-trip: {diffs}"


# `test_ags4_passthrough_registers_unknown_groups` retired with F2c-4.
# That test exercised the Python-side `ags5_models.register` mutation
# triggered by `ags5_ags4.read_ags4` when it encountered an unknown
# group. ags5-models retires in this stage; the passthrough story is
# now end-to-end Rust + laterite.dynamic. The dynamic-class-on-read
# path is covered by `packages/laterite/tests/test_typed_graph.py`'s
# passthrough round-trip tests.
