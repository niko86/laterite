"""Stage F2b-1: typed-graph engine smoke tests.

Exercises the Rust-side `#[pyclass]` classes codegen'd from
`ags5_dictionary.json` at build time. F2b-1 lands scalars only; child
fields (`Py<PyList>`), `walk()`, passthrough, and read/write_db arrive
in F2b-2 onward.

Tests focus on the contract that distinguishes a typed-graph class
from a generic Python object: kwarg constructor with field validation,
typed-attribute round-trip, the 92-class count from the registry, and
the AGS heading → field-name lowering convention.
"""

from __future__ import annotations

import pytest


def test_proj_construct_and_attribute_round_trip() -> None:
    """Scalar fields round-trip through the `#[pyo3(get, set)]` accessor."""
    from laterite._laterite_native import PROJ

    p = PROJ(proj_id="P1", proj_name="Test")
    assert p.proj_id == "P1"
    assert p.proj_name == "Test"
    # Unset fields default to None — every field is Optional.
    assert p.proj_memo is None


def test_typed_scalars() -> None:
    """Numeric (1DP, 2DP) AGS types map to Python float on the round-trip."""
    from laterite._laterite_native import LOCA

    loca = LOCA(loca_id="BH01", loca_type="CP", loca_gl=10.5)
    assert loca.loca_gl == 10.5
    assert isinstance(loca.loca_gl, float)
    assert loca.loca_id == "BH01"


def test_setter_updates_attribute() -> None:
    """`#[pyo3(get, set)]` means attribute assignment works post-construction."""
    from laterite._laterite_native import PROJ

    p = PROJ()
    assert p.proj_id is None
    p.proj_id = "new"
    assert p.proj_id == "new"


def test_unknown_kwarg_rejected() -> None:
    """The constructor is keyword-only with a fixed field set; unknown
    kwargs raise TypeError (this is the spec-correctness guard
    msgspec.Struct gave us for free; we keep it on the Rust side)."""
    from laterite._laterite_native import PROJ

    with pytest.raises(TypeError, match="unexpected keyword argument"):
        PROJ(not_a_field="x")  # type: ignore[call-arg]


def test_positional_args_rejected() -> None:
    """All fields are keyword-only — positional construction is a typo."""
    from laterite._laterite_native import PROJ

    with pytest.raises(TypeError, match="positional"):
        PROJ("P1")  # type: ignore[misc]


def test_all_92_groups_registered() -> None:
    """Every AGS group in the registry has a codegen'd class."""
    import laterite._laterite_native as native
    from laterite.registry import GROUPS

    missing = [code for code in GROUPS if not hasattr(native, code)]
    assert missing == [], f"groups without a typed class: {missing[:10]}"
    # 92 known plus any future additions — the registry is authoritative.
    assert len(GROUPS) == 92, f"unexpected registry size: {len(GROUPS)}"


def test_heading_name_to_field_name_lowering() -> None:
    """AGS headings are uppercase (LOCA_ID, SAMP_TOP); fields are lower
    (loca_id, samp_top). Same convention as ags5_models._modelgen."""
    from laterite._laterite_native import SAMP

    s = SAMP(
        loca_id="BH01",
        samp_top=5.0,
        samp_ref="S1",
        samp_type="U",
        samp_id="SAMP001",
    )
    assert s.loca_id == "BH01"
    assert s.samp_top == 5.0
    assert s.samp_id == "SAMP001"


def test_int_typed_field() -> None:
    """`0DP` (integer) AGS types round-trip as Python int."""
    from laterite._laterite_native import CORE

    # CORE_RQD is 0DP per the dictionary.
    c = CORE(loca_id="BH01", core_top=0.0, core_base=5.0, core_rqd=85)
    assert c.core_rqd == 85
    assert isinstance(c.core_rqd, int)


# --- F2b-2: Py<PyList> children + walk() -----------------------------


def test_child_field_defaults_to_empty_list() -> None:
    """Every parent group has a child field per direct child code; the
    default value is an empty list (each instance gets its own list —
    no shared-default-list bug)."""
    from laterite._laterite_native import PROJ

    p1 = PROJ(proj_id="A")
    p2 = PROJ(proj_id="B")
    assert isinstance(p1.locas, list)
    assert p1.locas == []
    # Independent lists — mutating one does not affect the other.
    p1.locas.append("placeholder")
    assert p2.locas == []


def test_child_list_identity_is_stable() -> None:
    """The `Py<PyList>` returned from each attribute access is the
    same Python list object. That's what makes `proj.locas.append(...)`
    work end-to-end."""
    from laterite._laterite_native import PROJ

    p = PROJ()
    assert p.locas is p.locas


def test_child_list_accepts_list_at_construction() -> None:
    """The constructor's child kwargs accept any iterable that PyO3
    can extract into `Py<PyList>` — a Python list is the natural form."""
    from laterite._laterite_native import LOCA, PROJ

    l1 = LOCA(loca_id="BH01", loca_type="CP")
    l2 = LOCA(loca_id="BH02", loca_type="CP")
    p = PROJ(proj_id="P", locas=[l1, l2])
    assert len(p.locas) == 2
    assert p.locas[0].loca_id == "BH01"
    assert p.locas[1].loca_id == "BH02"


def test_walk_direct_children() -> None:
    """`PROJ.walk('LOCA')` returns every LOCA at depth 1."""
    from laterite._laterite_native import LOCA, PROJ

    p = PROJ(proj_id="P", locas=[
        LOCA(loca_id="BH01", loca_type="CP"),
        LOCA(loca_id="BH02", loca_type="CP"),
    ])
    locas = p.walk("LOCA")
    assert len(locas) == 2
    assert [loc.loca_id for loc in locas] == ["BH01", "BH02"]


def test_walk_deep_descendants() -> None:
    """`PROJ.walk('TREL')` descends PROJ→LOCA→SAMP→TREG→TRET→TREL
    (5 levels) and returns every leaf instance."""
    from laterite._laterite_native import LOCA, PROJ, SAMP, TREG, TREL, TRET

    samp_keys = dict(loca_id="BH01", samp_top=5.0, samp_ref="S",
                     samp_type="U", samp_id="X")
    spec_keys = {**samp_keys, "spec_ref": "R", "spec_dpth": 5.0}
    trels = [
        TREL(**spec_keys, tret_tesn="1", trel_mnum=i, trel_cell=350.0 + i)
        for i in range(3)
    ]
    tret = TRET(**spec_keys, tret_tesn="1", trels=trels)
    treg = TREG(**spec_keys, treg_type="CU", trets=[tret])
    samp = SAMP(**samp_keys, tregs=[treg])
    loca = LOCA(loca_id="BH01", loca_type="CP", samps=[samp])
    p = PROJ(proj_id="P", locas=[loca])

    found = p.walk("TREL")
    assert len(found) == 3
    assert sorted(t.trel_mnum for t in found) == [0, 1, 2]


def test_walk_case_insensitive() -> None:
    """Code matching uses `eq_ignore_ascii_case` so `walk('trel')`,
    `walk('TREL')`, and `walk('Trel')` all match."""
    from laterite._laterite_native import LOCA, PROJ

    p = PROJ(proj_id="P", locas=[LOCA(loca_id="BH01", loca_type="CP")])
    assert len(p.walk("LOCA")) == 1
    assert len(p.walk("loca")) == 1
    assert len(p.walk("Loca")) == 1


def test_walk_unknown_code_returns_empty() -> None:
    """A code that isn't anywhere in the subtree returns []."""
    from laterite._laterite_native import LOCA, PROJ

    p = PROJ(proj_id="P", locas=[LOCA(loca_id="BH01", loca_type="CP")])
    assert p.walk("NOPE") == []


def test_walk_on_leaf_returns_empty() -> None:
    """Leaf groups (no children — e.g. TREL) walk to []."""
    from laterite._laterite_native import TREL

    leaf = TREL(loca_id="BH01", samp_top=5.0, samp_ref="S",
                samp_type="U", samp_id="X", spec_ref="R",
                spec_dpth=5.0, tret_tesn="1", trel_mnum=1)
    assert leaf.walk("TREL") == []
    assert leaf.walk("NOPE") == []


# --- F2b-4: read_db ------------------------------------------------


def _tiny_proj_and_write(tmp_path):
    """Build a small typed graph + return the written path."""
    from pathlib import Path

    from laterite import LOCA, PROJ, SAMP, TREG, TREL, TRET
    from laterite.ags5db import write_db as write_ags5db

    samp_keys = dict(loca_id="BH01", samp_top=5.0, samp_ref="S",
                     samp_type="U", samp_id="X")
    spec_keys = {**samp_keys, "spec_ref": "R", "spec_dpth": 5.0}
    trels = [
        TREL(**spec_keys, tret_tesn="1", trel_mnum=i, trel_cell=350.0 + i)
        for i in range(3)
    ]
    tret = TRET(**spec_keys, tret_tesn="1", trels=trels)
    treg = TREG(**spec_keys, treg_type="CU", trets=[tret])
    samp = SAMP(**samp_keys, tregs=[treg])
    loca = LOCA(loca_id="BH01", loca_type="CP", loca_gl=10.5, samps=[samp])
    proj = PROJ(proj_id="P1", proj_name="test", locas=[loca])
    db_path = Path(tmp_path) / "tiny.ags5db"
    write_ags5db(proj, db_path)
    return db_path


def test_read_db_round_trip_scalar_values(tmp_path) -> None:
    """Every scalar field round-trips through write_ags5db → read_db."""
    from laterite.ags5db import read_db

    db_path = _tiny_proj_and_write(tmp_path)
    proj = read_db(db_path)

    assert type(proj).__name__ == "PROJ"
    assert proj.proj_id == "P1"
    assert proj.proj_name == "test"


def test_read_db_round_trip_tree_structure(tmp_path) -> None:
    """read_db produces the full PROJ → LOCA → SAMP → TREG → TRET → TREL
    tree with each parent linked to its children via the typed list."""
    from laterite.ags5db import read_db

    db_path = _tiny_proj_and_write(tmp_path)
    proj = read_db(db_path)

    assert len(proj.locas) == 1
    loca = proj.locas[0]
    assert loca.loca_id == "BH01"
    assert loca.loca_gl == 10.5

    assert len(loca.samps) == 1
    samp = loca.samps[0]
    assert samp.samp_top == 5.0

    treg = samp.tregs[0]
    assert treg.treg_type == "CU"
    tret = treg.trets[0]
    trels = sorted(tret.trels, key=lambda t: t.trel_mnum)
    assert [t.trel_mnum for t in trels] == [0, 1, 2]
    assert [t.trel_cell for t in trels] == [350.0, 351.0, 352.0]


def test_read_db_walk_reaches_leaves(tmp_path) -> None:
    """walk('TREL') from the PROJ root finds every TREL across the tree."""
    from laterite.ags5db import read_db

    db_path = _tiny_proj_and_write(tmp_path)
    proj = read_db(db_path)
    assert len(proj.walk("TREL")) == 3
    assert len(proj.walk("LOCA")) == 1


def test_read_db_passthrough_attaches_dynamic_class(tmp_path) -> None:
    """A custom group (QQTS) ingested from AGS4 lands in the typed PROJ
    via a Python-side dynamic class registered by `laterite.dynamic`."""
    from pathlib import Path

    from laterite import ags5db, dynamic
    from laterite.ags5db import read_db

    # Clear the dynamic cache so we have a clean registration namespace.
    dynamic.clear_cache()

    ags = (
        '"GROUP","PROJ"\n'
        '"HEADING","PROJ_ID","PROJ_NAME"\n'
        '"UNIT","",""\n'
        '"TYPE","ID","X"\n'
        '"DATA","P1","test"\n'
        '\n'
        '"GROUP","LOCA"\n'
        '"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"\n'
        '"UNIT","","","m"\n'
        '"TYPE","ID","PA","2DP"\n'
        '"DATA","BH01","CP","10.50"\n'
        '\n'
        '"GROUP","QQTS"\n'
        '"HEADING","LOCA_ID","QQTS_REF","QQTS_VAL"\n'
        '"UNIT","","","kPa"\n'
        '"TYPE","ID","X","1DP"\n'
        '"DATA","BH01","R1","100.0"\n'
        '"DATA","BH01","R2","200.0"\n'
    )
    ags_path = Path(tmp_path) / "custom.ags"
    db_path = Path(tmp_path) / "custom.ags5db"
    ags_path.write_text(ags)
    ags5db.convert(ags_path, db_path)

    proj = read_db(db_path)
    loca = proj.locas[0]

    # QQTS rows attached via setattr — accessible via getattr.
    qqtss = getattr(loca, "qqtss", None)
    assert qqtss is not None
    assert len(qqtss) == 2
    assert type(qqtss[0]).__name__ == "QQTS"
    assert type(qqtss[0]).__module__ == "laterite.dynamic"
    assert qqtss[0].qqts_ref == "R1"
    assert qqtss[1].qqts_ref == "R2"

    # Importable after registration.
    from laterite.dynamic import QQTS  # noqa: PLC0415
    assert isinstance(qqtss[0], QQTS)


def test_read_db_dynamic_cache_is_stable(tmp_path) -> None:
    """A second read of the same file reuses the cached class (identity
    holds across calls — key invariant for isinstance to work)."""
    from pathlib import Path

    from laterite import ags5db, dynamic
    from laterite.ags5db import read_db

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
        '"GROUP","WIDG"\n'
        '"HEADING","LOCA_ID","WIDG_REF"\n'
        '"UNIT","",""\n'
        '"TYPE","ID","X"\n'
        '"DATA","BH01","R1"\n'
    )
    ags_path = Path(tmp_path) / "widg.ags"
    db_path = Path(tmp_path) / "widg.ags5db"
    ags_path.write_text(ags)
    ags5db.convert(ags_path, db_path)

    p1 = read_db(db_path)
    p2 = read_db(db_path)
    w1 = p1.locas[0].widgs[0]
    w2 = p2.locas[0].widgs[0]
    assert type(w1) is type(w2)


def test_read_db_missing_file_raises(tmp_path) -> None:
    """A path that doesn't exist surfaces FileNotFoundError, not a
    silent empty PROJ."""
    import pytest
    from laterite.ags5db import read_db

    with pytest.raises(FileNotFoundError):
        read_db(tmp_path / "nope.ags5db")


# --- F2b-5a: write_db (standard groups) ----------------------------


def _typed_proj_for_write():
    """Build a small PROJ tree using the compiled #[pyclass] types
    (NOT the msgspec ones) — exercises the typed-graph engine's own
    write path end-to-end without depending on ags5_models."""
    from laterite._laterite_native import (
        GEOL,
        LOCA,
        PROJ,
        SAMP,
        TREG,
        TREL,
        TRET,
    )

    samp_keys = dict(loca_id="BH01", samp_top=5.0, samp_ref="S",
                     samp_type="U", samp_id="X")
    spec_keys = {**samp_keys, "spec_ref": "R", "spec_dpth": 5.0}
    trels = [
        TREL(**spec_keys, tret_tesn="1", trel_mnum=i, trel_cell=350.0 + i)
        for i in range(3)
    ]
    tret = TRET(**spec_keys, tret_tesn="1", trels=trels)
    treg = TREG(**spec_keys, treg_type="CU", trets=[tret])
    samp = SAMP(**samp_keys, tregs=[treg])
    geol = GEOL(loca_id="BH01", geol_top=0.0, geol_base=5.0,
                geol_desc="CLAY")
    loca = LOCA(loca_id="BH01", loca_type="CP", loca_gl=10.5,
                samps=[samp], geols=[geol])
    return PROJ(proj_id="WRITE_RT", proj_name="round trip", locas=[loca])


def test_write_db_produces_a_file(tmp_path) -> None:
    """The simplest contract: write_db creates a .ags5db at the
    requested path."""
    from laterite.ags5db import write_db

    proj = _typed_proj_for_write()
    db_path = tmp_path / "wrt.ags5db"
    write_db(proj, db_path)

    assert db_path.exists()
    assert db_path.stat().st_size > 0


def test_write_db_then_read_db_round_trips_tree(tmp_path) -> None:
    """The full round-trip contract: write_db → read_db reproduces
    the same tree shape with every scalar preserved."""
    from laterite.ags5db import read_db, write_db

    src = _typed_proj_for_write()
    db_path = tmp_path / "rt.ags5db"
    write_db(src, db_path)
    back = read_db(db_path)

    assert back.proj_id == src.proj_id
    assert back.proj_name == src.proj_name
    assert len(back.locas) == len(src.locas)
    loca_b = back.locas[0]
    loca_s = src.locas[0]
    assert loca_b.loca_id == loca_s.loca_id
    assert loca_b.loca_gl == loca_s.loca_gl
    assert len(loca_b.samps) == 1
    assert len(loca_b.geols) == 1
    # Deep — TREL leaves preserved.
    trels = sorted(back.walk("TREL"), key=lambda t: t.trel_mnum)
    assert [t.trel_mnum for t in trels] == [0, 1, 2]
    assert [t.trel_cell for t in trels] == [350.0, 351.0, 352.0]


def test_write_db_overwrites_existing_file(tmp_path) -> None:
    """write_db replaces a pre-existing file at the path rather than
    appending or erroring."""
    from laterite._laterite_native import PROJ
    from laterite.ags5db import read_db, write_db

    db_path = tmp_path / "ow.ags5db"
    write_db(_typed_proj_for_write(), db_path)
    first_size = db_path.stat().st_size

    # Now overwrite with a different PROJ (no LOCAs).
    write_db(PROJ(proj_id="EMPTY"), db_path)
    back = read_db(db_path)
    assert back.proj_id == "EMPTY"
    assert back.locas == []
    # Smaller after overwrite (fewer rows + fewer descendant tables
    # populated).
    assert db_path.stat().st_size <= first_size


def test_write_db_passthrough_round_trips_via_session_registry(tmp_path) -> None:
    """F2b-5b: dynamic / passthrough classes attached to a standard
    parent flow through write_db via a session-extended registry —
    their descriptor lands in the file's `_spec_*` and rows round-trip
    on a fresh read."""
    from laterite import dynamic
    from laterite._laterite_native import LOCA, PROJ
    from laterite.ags5db import read_db, write_db

    dynamic.clear_cache()
    Widg = dynamic.get_or_register(
        "WDGT", [
            {"name": "LOCA_ID", "type": "ID"},
            {"name": "WDGT_REF", "type": "X"},
            {"name": "WDGT_VAL", "type": "1DP"},
        ],
    )
    loca = LOCA(loca_id="BH01", loca_type="CP")
    loca.wdgts = [
        Widg(loca_id="BH01", wdgt_ref="R1", wdgt_val=100.0),
        Widg(loca_id="BH01", wdgt_ref="R2", wdgt_val=200.0),
    ]
    proj = PROJ(proj_id="WPSS", locas=[loca])

    db_path = tmp_path / "pass.ags5db"
    write_db(proj, db_path)

    # Fresh process-equivalent: clear the dynamic cache so the read
    # path has to rebuild WDGT from the file's `_spec_*` tables (proves
    # the descriptor really landed in the file, not just in memory).
    dynamic.clear_cache()
    back = read_db(db_path)
    assert back.proj_id == "WPSS"
    assert len(back.locas) == 1
    widgs = getattr(back.locas[0], "wdgts", [])
    assert len(widgs) == 2
    assert sorted(w.wdgt_ref for w in widgs) == ["R1", "R2"]
    assert sorted(w.wdgt_val for w in widgs) == [100.0, 200.0]


def test_write_db_passthrough_from_ags4_round_trip(tmp_path) -> None:
    """End-to-end: AGS4 with a custom group → convert → read_db →
    write_db → read_db preserves the passthrough rows."""
    from laterite import ags5db, dynamic
    from laterite.ags5db import read_db, write_db

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
        '"GROUP","XCTG"\n'
        '"HEADING","LOCA_ID","XCTG_REF","XCTG_VAL"\n'
        '"UNIT","","","kPa"\n'
        '"TYPE","ID","X","1DP"\n'
        '"DATA","BH01","R1","123.4"\n'
    )
    ags_path = tmp_path / "src.ags"
    src_db = tmp_path / "src.ags5db"
    rewrite_db = tmp_path / "rewrite.ags5db"
    ags_path.write_text(ags)

    ags5db.convert(ags_path, src_db)
    proj = read_db(src_db)
    write_db(proj, rewrite_db)

    dynamic.clear_cache()
    back = read_db(rewrite_db)
    xctgs = getattr(back.locas[0], "xctgs", [])
    assert len(xctgs) == 1
    assert xctgs[0].xctg_ref == "R1"
    assert xctgs[0].xctg_val == 123.4
