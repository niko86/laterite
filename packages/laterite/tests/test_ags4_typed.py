"""Stage F2b-4b: `laterite.ags4.read_typed` — direct AGS4 → typed PROJ.

Smoke-tests the one-shot helper. The heavy lifting is exercised by
F2b-4's read_db tests; here we just confirm the wrapper produces the
same tree shape from an AGS4 source as the explicit
``convert → read_db`` path would.
"""

from __future__ import annotations

from pathlib import Path

import pytest

_TINY_AGS = (
    '"GROUP","PROJ"\n'
    '"HEADING","PROJ_ID","PROJ_NAME"\n'
    '"UNIT","",""\n'
    '"TYPE","ID","X"\n'
    '"DATA","P1","tiny project"\n'
    '\n'
    '"GROUP","LOCA"\n'
    '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"\n'
    '"UNIT","","","m"\n'
    '"TYPE","ID","PA","2DP"\n'
    '"DATA","BH01","CP","12.50"\n'
    '"DATA","BH02","CP","8.75"\n'
)


def test_read_typed_basic_tree(tmp_path: Path) -> None:
    """A 2-LOCA AGS4 → typed PROJ with both LOCAs present + scalar values
    preserved."""
    from laterite.ags4 import read_typed

    ags4 = tmp_path / "tiny.ags"
    ags4.write_text(_TINY_AGS, encoding="utf-8")

    proj = read_typed(ags4)
    assert type(proj).__name__ == "PROJ"
    assert proj.proj_id == "P1"
    assert proj.proj_name == "tiny project"
    assert len(proj.locas) == 2
    locas = sorted(proj.locas, key=lambda loc: loc.loca_id)
    assert locas[0].loca_id == "BH01"
    assert locas[0].loca_gl == 12.50
    assert locas[1].loca_id == "BH02"
    assert locas[1].loca_gl == 8.75


def test_read_typed_passthrough_attaches_dynamic_class(tmp_path: Path) -> None:
    """A custom group flows through the dynamic factory the same way it
    does for read_db — verifying the wrapper doesn't bypass the policy."""
    from laterite import dynamic
    from laterite.ags4 import read_typed

    dynamic.clear_cache()

    ags = (
        '"GROUP","PROJ"\n'
        '"HEADING","PROJ_ID"\n'
        '"UNIT",""\n'
        '"TYPE","ID"\n'
        '"DATA","P1"\n'
        '\n'
        '"GROUP","LOCA"\n'
        '"HEADING","LOCA_ID","LOCA_TYPE"\n'
        '"UNIT","",""\n'
        '"TYPE","ID","PA"\n'
        '"DATA","BH01","CP"\n'
        '\n'
        '"GROUP","ZZTS"\n'
        '"HEADING","LOCA_ID","ZZTS_REF","ZZTS_VAL"\n'
        '"UNIT","","","kPa"\n'
        '"TYPE","ID","X","1DP"\n'
        '"DATA","BH01","R1","123.4"\n'
    )
    ags_path = tmp_path / "custom.ags"
    ags_path.write_text(ags)

    proj = read_typed(ags_path)
    loca = proj.locas[0]
    zztss = getattr(loca, "zztss", None)
    assert zztss is not None
    assert len(zztss) == 1
    assert zztss[0].zzts_ref == "R1"
    assert zztss[0].zzts_val == 123.4
    from laterite.dynamic import ZZTS  # noqa: PLC0415
    assert isinstance(zztss[0], ZZTS)


def test_read_typed_missing_source_raises(tmp_path: Path) -> None:
    """A path that doesn't exist surfaces an error, not a silent empty
    PROJ."""
    from laterite.ags4 import read_typed

    # Either a FileNotFoundError or the convert-side RuntimeError is
    # acceptable — both signal the missing source.
    with pytest.raises((FileNotFoundError, RuntimeError)):
        read_typed(tmp_path / "nope.ags")
