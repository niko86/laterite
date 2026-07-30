"""`laterite.build_ags4` — the data→AGS4 door (frames → valid AGS4).

Exercises the native `emit_ags4_from_arrow` binding through the public
Python entry point: the DuckDB-bridge boundary (pandas *and* polars,
pyarrow-free), dictionary UNIT/TYPE fill, the three validity modes, and a
build→read round-trip."""

from __future__ import annotations

import laterite
import pandas as pd
import polars as pl
import pytest

# A complete transmission stamp. All five, because TRAN_PROD/RECV/STAT are
# REQUIRED by the dictionary: issue+date alone mint a TRAN that then reports
# Rule 10b on the empty cells. The old placeholder wrote "TBC" into all three,
# which is precisely how it silenced both Rule 14 and Rule 10b at once.
_TRAN = {
    "tran": laterite.TranStamp(
        issue="1",
        date="2026-07-30",
        producer="Acme Ground Engineering",
        recipient="Client Ltd",
        status="FINAL",
    )
}


def _proj() -> pd.DataFrame:
    return pd.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["Demo project"]})


def _group_rows(text: str, code: str) -> dict[str, list[str]]:
    """Reparse `text` and return the group's UNIT/TYPE/HEADING rows as cell lists —
    a structural assertion (the parser agrees these are the UNIT/TYPE rows) rather
    than a substring that a coincidental match elsewhere could satisfy."""
    from laterite import _laterite_native as _native

    g = _native.parse_primitives(text=text)["groups"][code]
    return {"headings": g["headings"], "types": g["types"], "units": g["units"]}


def test_emit_fills_unit_and_type_from_dict():
    # Columns are the AGS headings; UNIT/TYPE come from the 4.1.1 dict. Assert via a
    # reparse of the emitted file, not a substring: the parser must agree LOCA's
    # TYPE row is [ID, 2DP] and its UNIT row is ["", m] against the declared headings.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    rows = _group_rows(res.text, "LOCA")
    assert rows["headings"] == ["LOCA_ID", "LOCA_GL"]
    assert rows["types"] == ["ID", "2DP"]
    assert rows["units"] == ["", "m"]


def test_build_units_types_override():
    # LOCA_XTRA is a custom heading the dictionary doesn't know (default TYPE X);
    # the heading-keyed override (#294 F#9) gives it a real UNIT/TYPE, while the
    # standard headings still fill from the dict. Reparse so the override lands in
    # the right column against LOCA_XTRA, not merely somewhere in the text.
    loca = pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_GL": [1.0], "LOCA_XTRA": ["9"]})
    res = laterite.build_ags4(
        {"PROJ": _proj(), "LOCA": loca},
        mode="report",
        units={"LOCA": {"LOCA_XTRA": "kPa"}},
        types={"LOCA": {"LOCA_XTRA": "3DP"}},
    )
    rows = _group_rows(res.text, "LOCA")
    assert rows["headings"] == ["LOCA_ID", "LOCA_GL", "LOCA_XTRA"]
    assert rows["types"] == [
        "ID",
        "2DP",
        "3DP",
    ]  # LOCA_GL from dict, LOCA_XTRA overridden
    assert rows["units"] == ["", "m", "kPa"]


@pytest.mark.parametrize(
    "bad",
    [
        {"units": {"NOPE": {"X": "m"}}},  # unknown group
        {"types": {"LOCA": {"NOSUCH": "3DP"}}},  # unknown heading
    ],
)
def test_build_units_types_reject_unknown(bad):
    loca = pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_GL": [1.0]})
    with pytest.raises(ValueError, match=r"^build_ags4 "):
        laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, **bad)


def test_typed_float_is_canonical_by_construction():
    # A native float under a 2DP heading formats to "12.30" with no fixing.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert '"DATA","BH01","12.30"' in res.text
    assert res.fixes_applied == 0


def test_build_result_applied_ledger():
    # A string "1.0" under a 2DP heading is a safe Rule 8 reformat AutoFix applies
    # during the build — so `applied` carries its record (#294 F#7), the same
    # {kind,label,rule,line,risk} shape FixResult.applied uses, and fixes_applied
    # is exactly its length.
    loca = pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_GL": ["1.0"]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="autofix")
    assert res.fixes_applied == len(res.applied) >= 1
    fix = res.applied[0]
    assert set(fix) >= {"kind", "label", "rule", "line", "risk"}
    assert fix["rule"] == "AGS Format Rule 8"
    assert fix["risk"] == "safe"
    # report mode touches nothing -> empty ledger.
    rep = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="report")
    assert rep.applied == [] and rep.fixes_applied == 0


def test_polars_backend_works_pyarrow_free():
    # polars frames cross the same DuckDB bridge; no pyarrow needed.
    loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.3, 13.0]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert '"DATA","BH01","12.30"' in res.text
    assert '"DATA","BH02","13.00"' in res.text


def test_autofix_pads_a_string_numeric():
    # A string "12.3" under 2DP is non-compliant; AutoFix's safe fix pads it.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="autofix")
    assert res.fixes_applied >= 1
    assert '"12.30"' in res.text
    assert '"12.3"' not in res.text


def test_report_mode_keeps_strings_verbatim():
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="report")
    assert '"12.3"' in res.text
    assert res.fixes_applied == 0
    # The non-compliant cell is still surfaced as a finding.
    assert any("Rule 8" in f.get("rule", "") for f in res.findings)


def test_strict_mode_raises_on_invalid():
    # No PROJ / TRAN → error-severity rules → Strict refuses.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    with pytest.raises(RuntimeError, match="strict mode rejected"):
        laterite.build_ags4({"LOCA": loca}, mode="strict")


def test_edition_is_selectable():
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    # A different edition still resolves + emits (smoke; dict differs internally).
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, dict_version="4.2")
    assert '"DATA","BH01","12.30"' in res.text


def test_unknown_edition_and_mode_raise():
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    with pytest.raises(RuntimeError, match="unknown edition"):
        laterite.build_ags4({"LOCA": loca}, dict_version="9.9")
    with pytest.raises(RuntimeError, match="unknown mode"):
        laterite.build_ags4({"LOCA": loca}, mode="banana")


def test_round_trips_through_read():
    loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.3, 13.0]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    back = laterite.read(text=res.text)
    # Exactly what was built comes back — synthesis is opt-in, so the round trip
    # is no longer entangled with minted groups. Metadata catalogs are absent,
    # which is what the Rule 14/15/17 findings below say.
    assert sorted(back.groups) == ["LOCA", "PROJ"]
    assert {f["rule"] for f in res.findings} >= {
        "AGS Format Rule 14",
        "AGS Format Rule 15",
        "AGS Format Rule 17",
    }
    df = back["LOCA"]
    assert df["LOCA_ID"].to_list() == ["BH01", "BH02"]
    assert df["LOCA_GL"].to_list() == [12.3, 13.0]


def test_emit_result_write(tmp_path):
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    out = res.save(tmp_path / "out.ags")
    assert out.read_bytes() == res.bytes
    assert out.read_bytes().startswith(b'"GROUP","PROJ"')


def test_accepts_ordered_pairs_too():
    # A list of (code, frame) pairs preserves order without a Mapping.
    res = laterite.build_ags4(
        [("PROJ", _proj()), ("LOCA", pd.DataFrame({"LOCA_ID": ["BH01"]}))]
    )
    assert res.text.index('"GROUP","PROJ"') < res.text.index('"GROUP","LOCA"')


# --- typed-graph door (#214: parity with laterite-node's buildAgs4) --------
#
# build_ags4 also accepts a typed PROJ tree, walked depth-first the same way
# laterite-node's `walkTree` does. Before #214 a typed graph raised
# `TypeError: 'PROJ' object is not iterable` (it fell through to `list(groups)`);
# Node already accepted one via `instanceof AgsGroup`.


def test_typed_graph_root_builds():
    # The #214 repro: a hand-built PROJ tree used to raise TypeError.
    proj = laterite.groups.PROJ(proj_id="P1", proj_name="Demo project")
    proj.locas.append(laterite.groups.LOCA(loca_id="BH01", loca_gl=12.34))
    proj.locas.append(laterite.groups.LOCA(loca_id="BH02", loca_gl=8.0))
    res = laterite.build_ags4(proj)
    assert res.text.startswith('"GROUP","PROJ"')
    assert '"GROUP","LOCA"' in res.text


def test_typed_graph_native_floats_canonicalise():
    # Native floats carried on the typed node format to their 2DP heading with
    # no fixing — the same born-typed guarantee the frames door gives.
    proj = laterite.groups.PROJ(proj_id="P1")
    proj.locas.append(laterite.groups.LOCA(loca_id="BH01", loca_gl=12.34))
    proj.locas.append(laterite.groups.LOCA(loca_id="BH02", loca_gl=8.0))
    res = laterite.build_ags4(proj)
    assert '"DATA","BH01"' in res.text
    assert '"12.34"' in res.text
    assert '"8.00"' in res.text  # 8.0 padded to 2DP


def test_typed_graph_round_trips_through_read():
    proj = laterite.groups.PROJ(proj_id="P1", proj_name="Demo project")
    proj.locas.append(laterite.groups.LOCA(loca_id="BH01", loca_gl=12.34))
    proj.locas.append(laterite.groups.LOCA(loca_id="BH02", loca_gl=8.0))
    back = laterite.read(data=laterite.build_ags4(proj).bytes)
    assert sorted(back.groups) == ["LOCA", "PROJ"]
    loca = back["LOCA"]
    assert loca["LOCA_ID"].to_list() == ["BH01", "BH02"]
    assert loca["LOCA_GL"].to_list() == [12.34, 8.0]


def test_typed_graph_recurses_deeply():
    # The walk follows the registry's parent→child links to any depth:
    # PROJ → LOCA → SAMP → LLPL (the four-level chain in the dictionary).
    from laterite.groups import LLPL, LOCA, PROJ, SAMP

    proj = PROJ(proj_id="P1", proj_name="Deep tree")
    loca = LOCA(loca_id="BH01")
    samp = SAMP(loca_id="BH01", samp_id="S1", samp_top=1.5)
    samp.llpls.append(LLPL(loca_id="BH01", samp_id="S1", spec_ref="BS1377"))
    loca.samps.append(samp)
    proj.locas.append(loca)

    back = laterite.read(data=laterite.build_ags4(proj).bytes)
    # exactly the four walked groups — the depth-first walk is what is under test
    assert sorted(back.groups) == ["LLPL", "LOCA", "PROJ", "SAMP"]


def test_typed_graph_round_trips_via_read_typed(tmp_path):
    # read_typed builds a PROJ tree; build_ags4 walks it back out. read_typed
    # recovers only the PROJ-rooted subtree (parity-defining coverage — exactly
    # Node's walk; the root-metadata groups have parent None, so aren't in the
    # tree), and build_ags4's autofix then re-synthesizes TRAN/UNIT/TYPE.
    from laterite.ags4 import read_typed

    src = tmp_path / "in.ags"
    src.write_bytes(
        laterite.build_ags4(
            {
                "PROJ": _proj(),
                "LOCA": pl.DataFrame({"LOCA_ID": ["BH01", "BH02"]}),
            }
        ).bytes
    )
    proj = read_typed(src)
    back = laterite.read(data=laterite.build_ags4(proj).bytes)
    assert sorted(back.groups) == ["LOCA", "PROJ"]
    assert back["LOCA"]["LOCA_ID"].to_list() == ["BH01", "BH02"]


def test_typed_graph_childless_root_builds():
    res = laterite.build_ags4(laterite.groups.PROJ(proj_id="P1", proj_name="Solo"))
    back = laterite.read(data=res.bytes)
    assert sorted(back.groups) == ["PROJ"]
    assert back["PROJ"]["PROJ_ID"].to_list() == ["P1"]


def test_synthesise_metadata_mints_the_missing_catalogs_when_asked():
    # Opted in AND with the transmission stated, build_ags4 mints the mandatory
    # UNIT/TYPE catalogs (from the data) plus a TRAN carrying the caller's own
    # values, so a data-only build is valid in one call.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4(
        {"PROJ": _proj(), "LOCA": loca},
        synthesise_metadata=True,
        **_TRAN,
    )
    assert {"TRAN", "UNIT", "TYPE"}.issubset(laterite.read(data=res.bytes).groups)
    assert not res.findings  # fully valid, no Rule 14/15/17
    # The caller's values, never a placeholder — "TBC"/"1900-01-01" satisfied
    # Rule 14 while asserting a transmission that never happened.
    assert "Acme Ground Engineering" in res.text
    assert "TBC" not in res.text and "1900-01-01" not in res.text

    # Report mode never synthesises, opted in or not — it reports, it doesn't fix.
    rep = laterite.build_ags4(
        {"PROJ": _proj(), "LOCA": loca}, mode="report", synthesise_metadata=True
    )
    assert "TRAN" not in laterite.read(data=rep.bytes).groups
    assert rep.findings


def test_synthesis_is_off_by_default():
    # The behaviour change: the same build without the flag mints nothing and
    # surfaces the gaps instead, so the caller sees what they declined.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert not {"TRAN", "UNIT", "TYPE"} & set(laterite.read(data=res.bytes).groups)
    rules = {f["rule"] for f in res.findings}
    assert any("Rule 14" in r for r in rules)
    assert any("Rule 15" in r for r in rules)
    assert any("Rule 17" in r for r in rules)


def test_synthesise_metadata_mints_abbr_for_pa_codes():
    # When the data uses a PA picklist code (LOCA_TYPE is a PA heading), opted-in
    # synthesis also mints ABBR (Rule 16) defining that code.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_TYPE": ["TP"]})
    res = laterite.build_ags4(
        {"PROJ": _proj(), "LOCA": loca}, synthesise_metadata=True, **_TRAN
    )
    assert "ABBR" in laterite.read(data=res.bytes).groups
    assert '"DATA","LOCA_TYPE","TP"' in res.text
    assert not res.findings  # fully valid, incl. Rule 16


def test_typed_graph_emits_only_set_columns():
    # The typed-graph door emits only the headings you SET (like the frames
    # door), not the full union schema — so a sparse node builds clean at the
    # default edition instead of dragging in ~45 blank columns (whose unset
    # edition-specific / PA headings would trip Rule 9 / 16).
    proj = laterite.groups.PROJ(proj_id="LAT-DEMO", proj_name="Demo")
    proj.locas.append(laterite.groups.LOCA(loca_id="BH01", loca_gl=12.50))
    # Synthesis on + a stamped TRAN: a clean baseline is this test's
    # precondition — the claim is that PRUNING doesn't trip Rule 9/16, which
    # needs the metadata gaps closed so they don't drown the signal.
    res = laterite.build_ags4(proj, synthesise_metadata=True, **_TRAN)
    hdr = next(
        ln for ln in res.text.splitlines() if ln.startswith('"HEADING","LOCA_ID"')
    )
    assert hdr == '"HEADING","LOCA_ID","LOCA_GL"'  # only the two set columns, in order
    assert not res.findings  # valid at the default 4.1.1

    # No data loss: a deliberately-set heading survives the prune (here a
    # 4.2-only one — flagged at 4.1.1, clean at 4.2, value kept either way).
    p2 = laterite.groups.PROJ(proj_id="P", proj_name="x")
    p2.locas.append(
        laterite.groups.LOCA(loca_id="BH01", loca_gl=12.5, loca_vssl="MV Demo")
    )
    assert "LOCA_VSSL" in laterite.build_ags4(p2).text
    assert not laterite.build_ags4(
        p2, dict_version="4.2", synthesise_metadata=True, **_TRAN
    ).findings


def test_typed_graph_non_group_child_raises():
    # A foreign object hung off a child accessor is caught with a clear message,
    # not a confusing downstream failure (mirrors Node's walkTree guard).
    proj = laterite.groups.PROJ(proj_id="P1")
    proj.locas.append(object())  # type: ignore[arg-type]
    with pytest.raises(TypeError, match="not a known typed AGS group instance"):
        laterite.build_ags4(proj)


def test_typed_graph_passthrough_root_builds():
    # A laterite.dynamic passthrough class (a custom group) can be the root —
    # a Python-only superset of Node's walk (Node has no passthrough surface).
    from laterite import dynamic

    dynamic.clear_cache()
    cls = dynamic.get_or_register(
        "XCUS", [{"name": "XCUS_ID", "type": "ID"}, {"name": "XCUS_VAL", "type": "X"}]
    )
    res = laterite.build_ags4(cls(xcus_id="A", xcus_val="hello"))
    assert '"GROUP","XCUS"' in res.text
    assert '"hello"' in res.text


def test_typed_graph_custom_group_child_survives_round_trip(tmp_path):
    # Regression: read_typed hangs a custom group off its parent via setattr
    # (an undeclared `<code>s` accessor the registry doesn't know about). The
    # walk must still carry it, or read_typed → build_ags4 silently loses the
    # group and its data.
    from laterite import dynamic
    from laterite.ags4 import read_typed

    dynamic.clear_cache()
    src = (
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
        '"DATA","P1"\r\n\r\n'
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
        '"DATA","BH01"\r\n\r\n'
        '"GROUP","MYGP"\r\n"HEADING","LOCA_ID","MYGP_VAL"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\n"DATA","BH01","custom-data-here"\r\n'
    )
    p = tmp_path / "custom.ags"
    p.write_text(src)
    proj = read_typed(p)
    back = laterite.read(data=laterite.build_ags4(proj).bytes)
    assert "MYGP" in back.groups
    assert back["MYGP"]["MYGP_VAL"].to_list() == ["custom-data-here"]
