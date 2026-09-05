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


def test_build_out_writes_the_judged_file_and_returns_no_bytes(tmp_path):
    # The to-disk rider (#855): out= lands the judged document at the path and
    # hands back a BuildSaved — path + verdict, deliberately no bytes held.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    dest = tmp_path / "built.ags"
    saved = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, out=dest)
    assert isinstance(saved, laterite.BuildSaved)
    assert saved.path == dest
    assert not hasattr(saved, "bytes")
    # The file on disk is byte-identical to what the bytes-carrying door
    # returns for the same input — the rider changes where, never what.
    plain = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert dest.read_bytes() == plain.bytes
    assert saved.findings == plain.findings
    assert saved.fixes_applied == plain.fixes_applied
    # No staging debris left beside the destination.
    assert [p.name for p in tmp_path.iterdir()] == ["built.ags"]


def test_build_out_strict_failure_writes_nothing(tmp_path):
    # Build-and-judge survives the trip to disk: a strict refusal raises with
    # the destination untouched — never a file of unjudged bytes.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    dest = tmp_path / "refused.ags"
    with pytest.raises(RuntimeError, match="strict mode rejected"):
        laterite.build_ags4({"LOCA": loca}, mode="strict", out=dest)
    assert list(tmp_path.iterdir()) == []


def test_build_out_replaces_an_existing_file_atomically(tmp_path):
    # os.replace semantics: an existing destination is overwritten in one
    # move, and the autofix rewrite happened before the path existed.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})  # string → padded
    dest = tmp_path / "built.ags"
    dest.write_bytes(b"stale previous content")
    saved = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, out=dest)
    text = dest.read_text()
    assert '"12.30"' in text and "stale" not in text
    assert saved.fixes_applied >= 1


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


def _data_rows(text: str, code: str) -> list[list[str]]:
    """The group's DATA rows, reparsed — structural, like `_group_rows`."""
    from laterite import _laterite_native as _native

    rows = _native.parse_primitives(text=text)["groups"][code]["rows"]
    return [r["values"] for r in rows]


def test_a_dt_cell_is_written_at_its_headings_declared_precision():
    """#695. A `DT` column declares its precision in its UNIT and Rule 8 judges
    the cell against it, but a typed datetime carries no precision — a date-only
    cell read from disk comes back as midnight. Rendering that as Arrow does
    (`2021-08-09T00:00:00`) fails the `yyyy-mm-dd` unit the heading itself
    declares.

    All three cases live in one file so the rule is visible as one rule: the
    DECLARED precision decides, not the value's own zeros.
    """
    import datetime as dt

    loca = pl.DataFrame(
        # LOCA_STAR declares `yyyy-mm-dd`.
        {"LOCA_ID": ["BH01"], "LOCA_STAR": [dt.datetime(2021, 8, 9, 0, 0)]}
    )
    mond = pl.DataFrame(
        # MOND_DTIM declares `yyyy-mm-ddThh:mm:ss`.
        {
            "LOCA_ID": ["BH01", "BH01"],
            "MOND_DTIM": [
                dt.datetime(2021, 8, 9, 0, 0),
                dt.datetime(2021, 8, 9, 14, 30),
            ],
        }
    )
    res = laterite.build_ags4(
        {"PROJ": _proj(), "LOCA": loca, "MOND": mond}, dict_version="4.2", mode="report"
    )

    assert _data_rows(res.text, "LOCA")[0][1] == "2021-08-09", (
        "midnight under a date-only unit must be written date-only"
    )
    dtim = [r[1] for r in _data_rows(res.text, "MOND")]
    assert dtim[0] == "2021-08-09T00:00:00", (
        "the SAME instant keeps its time under a unit that declares seconds — "
        "truncating here would break a file that is Rule 8 clean today"
    )
    assert dtim[1] == "2021-08-09T14:30:00", (
        "a real time is never trimmed to fit; a lossy render is refused so the "
        "mismatch reaches the caller as a finding instead"
    )
    assert not [f for f in res.findings if f["rule"] == "AGS Format Rule 8"], (
        "no DT cell should contradict its own heading"
    )


def test_read_build_round_trip_of_a_date_only_dt_needs_no_fix(tmp_path):
    """#695 as reported: a clean file read back and re-emitted was clean only
    through `autofix`, which repaired what we had just mis-written. `strict`
    raised. The round trip is now clean in every mode, and byte-identical.
    """
    src = tmp_path / "base.ags"
    res = laterite.build_ags4(
        {
            "PROJ": _proj(),
            "LOCA": pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_STAR": ["2021-08-09"]}),
        },
        dict_version="4.2",
        synthesise_metadata=True,
        **_TRAN,
    )
    src.write_bytes(res.text.encode("utf-8"))

    ags = laterite.read(src)
    assert ags["LOCA"]["LOCA_STAR"].to_list() == [
        __import__("datetime").datetime(2021, 8, 9, 0, 0)
    ], "read() still promotes a date-only DT to midnight — that half is deliberate"

    frames = [(code, ags[code]) for code in ags.groups]
    for mode in ("strict", "report", "autofix"):
        out = laterite.build_ags4(frames, dict_version="4.2", mode=mode)
        assert not [f for f in out.findings if f["rule"] == "AGS Format Rule 8"], mode
        assert out.fixes_applied == 0, f"{mode}: nothing should need fixing"
        assert _data_rows(out.text, "LOCA")[0][1] == "2021-08-09"


# --- build_ags4_unchecked (#858): the no-verdict door -----------------------


def test_unchecked_bytes_equal_the_judged_report_bytes():
    # The whole contract: same fills, same formatting, same order as
    # build_ags4(mode="report") — the only thing removed is the verdict.
    # Held on a clean build AND a dirty one; asserting report actually
    # OBJECTED to the dirty one keeps the identity falsifiable.
    clean = {
        "PROJ": _proj(),
        "LOCA": pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]}),
    }
    judged = laterite.build_ags4(clean, mode="report")
    assert laterite.build_ags4_unchecked(clean) == judged.bytes

    # No PROJ, a non-canonical string under the 2DP dict fill.
    dirty = {"LOCA": pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})}
    judged = laterite.build_ags4(dirty, mode="report")
    assert judged.findings, "the dirty fixture must draw findings"
    assert laterite.build_ags4_unchecked(dirty) == judged.bytes


def test_unchecked_returns_plain_bytes_not_a_build_result():
    # Deliberately NOT a BuildResult: an empty `findings` would read as
    # "judged clean", and nothing here judged anything. The absent field IS
    # the statement.
    raw = laterite.build_ags4_unchecked({"PROJ": _proj()})
    assert type(raw) is bytes


def test_unchecked_accepts_edition_units_and_types():
    # The data-shaping knobs survive; only the judge-coupled ones
    # (mode / synthesise_metadata / tran) are gone.
    loca = pl.DataFrame({"LOCA_ID": ["BH1"], "LOCA_GL": [1.0], "LOCA_XTRA": ["9"]})
    raw = laterite.build_ags4_unchecked(
        {"PROJ": _proj(), "LOCA": loca},
        dict_version="4.2",
        units={"LOCA": {"LOCA_XTRA": "kPa"}},
        types={"LOCA": {"LOCA_XTRA": "3DP"}},
    )
    rows = _group_rows(raw.decode("utf-8"), "LOCA")
    assert rows["types"] == ["ID", "2DP", "3DP"]
    assert rows["units"] == ["", "m", "kPa"]
    with pytest.raises(TypeError):
        laterite.build_ags4_unchecked({"PROJ": _proj()}, mode="report")
    with pytest.raises(ValueError, match=r"^build_ags4_unchecked "):
        laterite.build_ags4_unchecked({"PROJ": _proj()}, units={"NOPE": {"X": "m"}})


def test_unchecked_out_stages_the_write(tmp_path):
    # Same staged write as build_ags4(out=) — a temp file beside the
    # destination, os.replace'd into place — WITHOUT the verdict gate:
    # unvalidated bytes landing on disk is the feature being asked for.
    frames = {"PROJ": _proj()}
    dest = tmp_path / "delivery.ags"
    ret = laterite.build_ags4_unchecked(frames, out=dest)
    assert ret == dest
    assert dest.read_bytes() == laterite.build_ags4_unchecked(frames)
    leftovers = [p for p in tmp_path.iterdir() if p != dest]
    assert not leftovers, f"staging must clean up: {leftovers}"


def test_out_failure_keeps_its_oserror_subclass(tmp_path):
    # The staged write's error contract (#938): a missing destination
    # directory raises FileNotFoundError — the subclass, not a flattened
    # base OSError — exactly what the old in-Python dance raised before the
    # shared native door took over. The node twin of this pin is
    # p2.test.ts's /ENOENT/ match.
    frames = {"PROJ": _proj()}
    with pytest.raises(FileNotFoundError):
        laterite.build_ags4_unchecked(frames, out=tmp_path / "missing" / "x.ags")
    assert not list(tmp_path.iterdir()), "a failed write must not litter"


def test_unchecked_out_writes_even_a_dirty_file(tmp_path):
    # THE difference from build_ags4(out=): nothing judges, so a file
    # build_ags4(mode="strict") refuses still lands — the caller chose
    # unchecked, and the docstring is the consent form.
    dirty = {"LOCA": pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})}
    dest = tmp_path / "dirty.ags"
    with pytest.raises(RuntimeError):
        laterite.build_ags4(dirty, mode="strict", out=dest)
    assert not dest.exists(), "the judged door must not have written"
    laterite.build_ags4_unchecked(dirty, out=dest)
    assert dest.exists(), "the unchecked door writes what the caller chose"
