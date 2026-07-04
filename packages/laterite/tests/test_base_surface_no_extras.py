"""Regression guard: the **base** ``laterite`` surface works on a base-only
install and never reaches *into* an optional extra or the now-decoupled
experimental AGS5 package (#111, #177).

The original report: a base ``pip install laterite`` user called
``from laterite.ags4 import read_typed`` and got
``ModuleNotFoundError: laterite.ags5db requires the 'ags5' extra`` — a base
namespace silently depending on the heavy AGS5 (DuckDB) wheel. The
emit path had the same disease (it round-tripped frames through DuckDB,
whose polars ingest pulls ``pyarrow`` — a ``[compat]`` dep).

Since #177 the experimental ``.ags5db`` surface is fully decoupled — its code
moved to the dormant ``ags5/`` holding folder, there is no ``[ags5]`` extra,
and ``laterite.ags5db`` no longer ships. This test still guards that the base
never re-acquires that dependency, alongside the live ``[compat]`` extras.

We can't uninstall the extras in the dev workspace (everything's installed),
so we run a **subprocess** with a ``sys.meta_path`` finder that makes the
decoupled/optional top-levels (``laterite_ags5`` = the AGS5 package;
``pandas`` / ``pyarrow`` = ``[compat]``) un-importable — a faithful base-only
simulation. Base ``duckdb`` / ``polars`` stay (they ARE base deps). The
subprocess exercises the whole documented base surface and asserts:

* every base call works,
* no blocked package is imported as a side effect, and
* the decoupled AGS5 surface (``laterite.ags5db``) is absent from the base.

If this test fails, a base feature has started depending on an extra or the
decoupled AGS5 package again.
"""

from __future__ import annotations

import subprocess
import sys
import textwrap

# Runs inside a fresh interpreter with the extras blocked BEFORE laterite is
# imported. Exits non-zero (with a diagnostic dump) on any base-surface break.
_EXERCISE = textwrap.dedent(
    '''
    import sys

    BLOCKED = {"laterite_ags5", "pandas", "pyarrow"}  # decoupled AGS5 pkg + [compat]

    class _BlockExtras:
        def find_spec(self, name, path=None, target=None):
            if name.split(".")[0] in BLOCKED:
                raise ModuleNotFoundError(
                    f"[base-only sim] {name!r} is an optional extra, not installed"
                )
            return None

    sys.meta_path.insert(0, _BlockExtras())

    import tempfile
    from pathlib import Path

    TINY = (
        '"GROUP","PROJ"\\n"HEADING","PROJ_ID","PROJ_NAME"\\n"UNIT","",""\\n'
        '"TYPE","ID","X"\\n"DATA","P1","demo"\\n\\n'
        '"GROUP","LOCA"\\n"HEADING","LOCA_ID","LOCA_TYPE","LOCA_GL"\\n'
        '"UNIT","","","m"\\n"TYPE","ID","PA","2DP"\\n'
        '"DATA","BH01","CP","12.50"\\n"DATA","BH02","CP","8.75"\\n'
    )
    d = Path(tempfile.mkdtemp())
    ags = d / "t.ags"
    ags.write_text(TINY)

    import laterite          # must not import any blocked extra
    import polars as pl      # base dep

    def t_read():
        f = laterite.read(ags)
        assert "LOCA" in f.groups and len(f["LOCA"]) == 2
    def t_sql():
        laterite.read(ags).sql("SELECT COUNT(*) AS n FROM LOCA").fetchall()
    def t_connection():
        assert laterite.read(ags).connection is not None
    def t_save():
        assert laterite.read(ags).save(d / "o.ags").exists()
    def t_validate():
        laterite.validate(ags)
    def t_emit():
        res = laterite.build_ags4(
            {"PROJ": pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["demo"]})}
        )
        assert res.bytes
    def t_read_typed():
        from laterite.ags4 import read_typed
        proj = read_typed(ags)
        assert proj.proj_id == "P1" and len(proj.locas) == 2
    def t_transport():
        from laterite import transport
        z = transport.pack(ags); assert z.exists()
        assert transport.unpack(z).exists()
        a = transport.lock(ags, password="pw"); assert a.exists()
        transport.unlock(a, password="pw")
    def t_registry():
        from laterite.registry import GROUPS, child_groups
        assert "LOCA" in GROUPS and any(c.code == "LOCA" for c in child_groups("PROJ"))
    def t_ags_types():
        from laterite.ags_types import canonical_type, parse_value
        assert parse_value("12.50", "2DP") == 12.5 and canonical_type("2DP")
    def t_typed_classes():
        from laterite.groups import LOCA, PROJ
        p = PROJ(proj_id="P1", locas=[LOCA(loca_id="BH01")])
        assert p.proj_id == "P1" and p.locas[0].loca_id == "BH01"
    def t_dict_for():
        laterite.dict_for(text=TINY)
    def t_dynamic():
        from laterite import dynamic
        cls = dynamic.get_or_register("ZZTS", [{"name": "ZZTS_REF", "type": "X"}])
        assert cls(zzts_ref="R1").zzts_ref == "R1"

    CASES = [
        ("read", t_read), ("sql", t_sql), ("connection", t_connection),
        ("save", t_save), ("validate", t_validate), ("build_ags4", t_emit),
        ("ags4.read_typed", t_read_typed), ("transport", t_transport),
        ("registry", t_registry), ("ags_types", t_ags_types),
        ("typed_classes", t_typed_classes), ("dict_for", t_dict_for),
        ("dynamic", t_dynamic),
    ]
    fails = []
    for name, fn in CASES:
        try:
            fn()
        except Exception as e:
            fails.append(f"{name}: {type(e).__name__}: {e}")

    leaked = sorted(m for m in sys.modules if m.split(".")[0] in BLOCKED)

    # The decoupled AGS5 surface must be ABSENT from the base (#177): no
    # `laterite.ags5db` module ships, so importing it fails.
    gate_ok = False
    try:
        from laterite.ags5db import read_db  # noqa: F401
    except ModuleNotFoundError as e:
        gate_ok = "ags5" in str(e).lower()

    if fails or leaked or not gate_ok:
        print("BASE-SURFACE FAILURES:", fails)
        print("LEAKED PACKAGES:", leaked)
        print("ags5 absent from base:", gate_ok)
        sys.exit(1)
    sys.exit(0)
    '''
)


def test_base_surface_works_without_optional_extras() -> None:
    """Full base API works under a simulated base-only install (no [ags5]/[compat])."""
    proc = subprocess.run(
        [sys.executable, "-c", _EXERCISE],
        capture_output=True,
        text=True,
        timeout=300,
    )
    assert proc.returncode == 0, (
        "base laterite reached into an optional extra (regression of #111):\n"
        f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
    )
