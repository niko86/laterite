"""laterite.ags5db conversion contract tests.

Exercises the in-process Rust ags5db engine (convert / export / query)
exposed through laterite._laterite_native: an AGS4 -> .ags5db -> AGS4
round trip preserves the group set, query primitives return the right
shapes, and a missing source raises carrying the binary's exit code.
`.agsx` retired in Stage F2a — see laterite_ags5x.ags4_to_agsx for the
surviving Python-only `.agsx` writer.
"""

from __future__ import annotations

from pathlib import Path

import laterite
import pytest
from laterite import ags5db

_FIX = Path(__file__).parents[3] / "rust-packages" / "laterite-ags4-validator" / "tests" / "fixtures"
_CLEAN = _FIX / "clean_minimal.ags"


def test_convert_creates_db(tmp_path: Path) -> None:
    db = tmp_path / "clean.ags5db"
    stats = ags5db.convert(_CLEAN, db)
    assert db.exists()
    assert stats["bytes"] > 0
    assert isinstance(stats["mode"], str)
    assert stats["attachments"] == 0  # clean_minimal has no FILE refs
    assert isinstance(stats["warnings"], list)


def test_round_trip_preserves_group_set(tmp_path: Path) -> None:
    db = tmp_path / "clean.ags5db"
    ags5db.convert(_CLEAN, db)

    out = tmp_path / "round.ags"
    stats = ags5db.export(db, out)
    assert out.exists()
    assert stats["groups_emitted"] >= 1
    assert stats["rows_emitted"] >= 1

    # The engine round-trips the group set (PROJ/TRAN/UNIT/TYPE here).
    original = set(laterite.read(_CLEAN).groups)
    restored = set(laterite.read(out).groups)
    assert restored == original


# `.agsx` ↔ `.ags5db` conversion tests retired in Stage F2a; `.agsx` is
# now a Python-only inspection helper via `laterite_ags5x.ags4_to_agsx`
# (covered by tests/test_ags4_to_agsx.py).


def test_export_missing_db_raises(tmp_path: Path) -> None:
    with pytest.raises(RuntimeError) as exc:
        ags5db.export(tmp_path / "nope.ags5db", tmp_path / "out.ags")
    # CliError surfaced with the binary's exit-code metadata.
    assert "ags5db error (exit" in str(exc.value)


def test_convert_into_src_named_dest_after_python_duckdb(tmp_path: Path) -> None:
    """Regression: `compact_db` previously hard-coded ATTACH AS `src`, which
    collided when the destination file was literally named `src.ags5db` and
    Python's duckdb wheel had any catalog open in the process — DuckDB's
    instance manager refused the duplicate name across bindings. Fixed by
    uniquifying the alias (`ags5db_compact_src_<uuid>`)."""
    import duckdb  # type: ignore[import-untyped]

    # Force Python's libduckdb into the process (the cross-binding trigger).
    duckdb.connect(":memory:").close()

    db = tmp_path / "src.ags5db"  # filename stem == old hard-coded alias.
    ags5db.convert(_CLEAN, db)
    assert db.exists() and db.stat().st_size > 0


# --- read-side query API --------------------------------------------

# A tiny hand-authored AGS4 with a numeric LOCA_GL (2DP) so count / sum /
# peek / sql have a numeric heading to work over. CRLF per AGS4 Rule 2a.
_NUMERIC_LINES = [
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID","PROJ_NAME"',
    '"UNIT","",""',
    '"TYPE","ID","X"',
    '"DATA","P1","query test"',
    "",
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_GL"',
    '"UNIT","","m"',
    '"TYPE","ID","2DP"',
    '"DATA","BH01","10.50"',
    '"DATA","BH02","20.25"',
    "",
]


@pytest.fixture(scope="module")
def numeric_db(tmp_path_factory: pytest.TempPathFactory) -> Path:
    # Module-scoped: every consumer (count/sum/sql/peek/query/list_blobs/
    # validate/info/groups) is read-only, so the ~15 of them share ONE
    # convert instead of re-running it per test. Biggest single cut to the
    # suite's convert count; build once into a shared temp dir.
    d = tmp_path_factory.mktemp("numeric_db")
    ags = d / "num.ags"
    ags.write_bytes(("\r\n".join(_NUMERIC_LINES)).encode("utf-8"))
    db = d / "num.ags5db"
    ags5db.convert(ags, db)
    return db


def test_count(numeric_db: Path) -> None:
    assert ags5db.count(numeric_db, "LOCA") == 2
    assert ags5db.count(numeric_db, "LOCA", where=["LOCA_ID=BH01"]) == 1
    assert ags5db.count(numeric_db, "LOCA", where=["LOCA_ID=ZZ"]) == 0


def test_sum(numeric_db: Path) -> None:
    total = ags5db.sum(numeric_db, "LOCA", "LOCA_GL")
    assert abs(total - 30.75) < 1e-9
    filtered = ags5db.sum(numeric_db, "LOCA", "LOCA_GL", where=["LOCA_ID=BH01"])
    assert abs(filtered - 10.50) < 1e-9


def test_sum_non_numeric_raises(numeric_db: Path) -> None:
    with pytest.raises(RuntimeError) as exc:
        ags5db.sum(numeric_db, "LOCA", "LOCA_ID")
    assert "ags5db error (exit 5)" in str(exc.value)


def test_sql(numeric_db: Path) -> None:
    out = ags5db.sql(numeric_db, "SELECT loca_id FROM v_loca ORDER BY loca_id")
    assert out["columns"] == ["loca_id"]
    assert [r["loca_id"] for r in out["records"]] == ["BH01", "BH02"]


def test_peek(numeric_db: Path) -> None:
    out = ags5db.peek(numeric_db, "LOCA")
    assert "loca_id" in out["columns"] and "loca_gl" in out["columns"]
    assert len(out["records"]) == 2

    picked = ags5db.peek(numeric_db, "LOCA", fields="loca_id")
    assert picked["columns"] == ["loca_id"]


# --- F2a-2: typed Predicate API + query (polars) + list_blobs -------


def test_predicate_class() -> None:
    p = ags5db.Predicate("trel_mnum", "<=", 5)
    assert p.to_where_string() == "trel_mnum<=5"
    with pytest.raises(ValueError, match="disallowed predicate op"):
        ags5db.Predicate("x", "bogus", 1)  # type: ignore[arg-type]


def test_count_accepts_predicate(numeric_db: Path) -> None:
    p = ags5db.Predicate("LOCA_ID", "=", "BH01")
    assert ags5db.count(numeric_db, "LOCA", where=p) == 1


def test_sum_accepts_predicate_list(numeric_db: Path) -> None:
    preds = [
        ags5db.Predicate("LOCA_GL", ">=", 10.0),
        ags5db.Predicate("LOCA_GL", "<", 20.0),
    ]
    total = ags5db.sum(numeric_db, "LOCA", "LOCA_GL", where=preds)
    # Only BH01 (10.50) matches both predicates (BH02=20.25 excluded by <20.0).
    assert abs(total - 10.50) < 1e-9


def test_query_returns_polars(numeric_db: Path) -> None:
    import polars as pl
    df = ags5db.query(numeric_db, "LOCA", fields=["loca_id", "loca_gl"])
    assert isinstance(df, pl.DataFrame)
    assert df.columns == ["loca_id", "loca_gl"]
    assert df.height == 2


def test_query_with_predicate(numeric_db: Path) -> None:
    df = ags5db.query(
        numeric_db,
        "LOCA",
        fields=["loca_id"],
        where=ags5db.Predicate("LOCA_ID", "=", "BH01"),
    )
    assert df.height == 1
    assert df["loca_id"][0] == "BH01"


def test_list_blobs_empty(numeric_db: Path) -> None:
    """The numeric fixture has no FILE attachments, so list_blobs returns []."""
    blobs = ags5db.list_blobs(numeric_db)
    assert blobs == []


def test_validate_clean_db_returns_empty(numeric_db: Path) -> None:
    """A clean DB with no PA/DT issues should validate cleanly."""
    findings = ags5db.validate(numeric_db)
    assert findings == []


def test_validate_catches_abbr_unknown(tmp_path: Path) -> None:
    """A PA value not declared in the file's ABBR group is flagged."""
    # AGS4 fixture with a LOCA_TYPE = "INVALID" but only "CP" in ABBR.
    # LOCA_TYPE is PA-typed; only ABBR-declared codes should pass.
    lines = [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","abbr test"',
        "",
        '"GROUP","ABBR"',
        '"HEADING","ABBR_HDNG","ABBR_CODE","ABBR_DESC"',
        '"UNIT","","",""',
        '"TYPE","X","X","X"',
        '"DATA","LOCA_TYPE","CP","Cable Percussion"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_TYPE"',
        '"UNIT","",""',
        '"TYPE","ID","PA"',
        '"DATA","BH01","CP"',
        '"DATA","BH02","INVALID"',
        "",
    ]
    ags = tmp_path / "abbr.ags"
    ags.write_bytes(("\r\n".join(lines)).encode("utf-8"))
    db = tmp_path / "abbr.ags5db"
    ags5db.convert(ags, db)

    findings = ags5db.validate(db)
    abbr_findings = [f for f in findings if f.code == "abbr_unknown"]
    assert len(abbr_findings) == 1
    assert "INVALID" in abbr_findings[0].message
    assert "LOCA_TYPE" in abbr_findings[0].where


def test_validate_check_abbr_disabled(tmp_path: Path) -> None:
    """With check_abbr=False, abbr_unknown findings are suppressed."""
    lines = [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","check off"',
        "",
        '"GROUP","ABBR"',
        '"HEADING","ABBR_HDNG","ABBR_CODE","ABBR_DESC"',
        '"UNIT","","",""',
        '"TYPE","X","X","X"',
        '"DATA","LOCA_TYPE","CP","Cable Percussion"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_TYPE"',
        '"UNIT","",""',
        '"TYPE","ID","PA"',
        '"DATA","BH02","INVALID"',
        "",
    ]
    ags = tmp_path / "off.ags"
    ags.write_bytes(("\r\n".join(lines)).encode("utf-8"))
    db = tmp_path / "off.ags5db"
    ags5db.convert(ags, db)

    findings = ags5db.validate(db, check_abbr=False)
    assert not any(f.code == "abbr_unknown" for f in findings)


# --- F2a-2e: info / groups / headings --------------------------------


def test_info_basic(numeric_db: Path) -> None:
    summary = ags5db.info(numeric_db)
    assert summary["file"].endswith("num.ags5db")
    assert summary["size_mb"] > 0
    # Synthetic fixtures don't necessarily have format_version set; just
    # confirm the key shape exists.
    assert "format_version" in summary
    assert "library_version" in summary
    assert summary["n_groups"] > 0
    assert summary["n_nonempty"] >= 1   # at least PROJ + LOCA carry rows
    codes = [g["code"] for g in summary["groups"]]
    assert "PROJ" in codes and "LOCA" in codes


def test_groups_all_and_nonempty(numeric_db: Path) -> None:
    all_groups = ags5db.groups(numeric_db)
    populated = ags5db.groups(numeric_db, nonempty=True)
    # Every populated group is also in the full list.
    pop_codes = {g["code"] for g in populated}
    all_codes = {g["code"] for g in all_groups}
    assert pop_codes <= all_codes
    # All populated groups have rows > 0.
    assert all(g["rows"] > 0 for g in populated)
    # PROJ + LOCA are populated in numeric_db.
    assert {"PROJ", "LOCA"} <= pop_codes


def test_headings_loca(numeric_db: Path) -> None:
    hs = ags5db.headings(numeric_db, "LOCA")
    names = [h["name"] for h in hs]
    assert "LOCA_ID" in names
    assert "LOCA_GL" in names
    loca_id = next(h for h in hs if h["name"] == "LOCA_ID")
    assert loca_id["status"] == "KEY"
    loca_gl = next(h for h in hs if h["name"] == "LOCA_GL")
    assert loca_gl["ags_type"] == "2DP"


def test_headings_unknown_group_raises(numeric_db: Path) -> None:
    with pytest.raises(RuntimeError):
        ags5db.headings(numeric_db, "NOPE")


def test_info_missing_file_raises(tmp_path: Path) -> None:
    with pytest.raises(RuntimeError):
        ags5db.info(tmp_path / "missing.ags5db")


def test_inspect_scalar_only(numeric_db: Path) -> None:
    report = ags5db.inspect(numeric_db)
    assert "format_version" in report
    assert "library_version" in report
    assert report["n_groups"] > 0
    assert report["n_headings"] > 0
    # No --group → no group block / headings.
    assert "group" not in report
    assert "headings" not in report


def test_inspect_with_group(numeric_db: Path) -> None:
    report = ags5db.inspect(numeric_db, group="LOCA")
    assert report["group"]["code"] == "LOCA"
    assert isinstance(report["headings"], list)
    names = [h["name"] for h in report["headings"]]
    assert "LOCA_ID" in names and "LOCA_GL" in names


def test_inspect_unknown_group_raises(numeric_db: Path) -> None:
    with pytest.raises(RuntimeError):
        ags5db.inspect(numeric_db, group="NOPE")


# --- F2a-2d: diff -----------------------------------------------------


def _write_loca_ags(tmp_path: Path, name: str, locas: list[tuple[str, str]]) -> Path:
    """Tiny helper: build a .ags4 with PROJ + N LOCA rows then convert."""
    lines = [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","diff test"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_TYPE"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
    ]
    for loca_id, loca_type in locas:
        lines.append(f'"DATA","{loca_id}","{loca_type}"')
    lines.append("")
    ags = tmp_path / f"{name}.ags"
    ags.write_bytes(("\r\n".join(lines)).encode("utf-8"))
    db = tmp_path / f"{name}.ags5db"
    ags5db.convert(ags, db)
    return db


@pytest.fixture(scope="module")
def diff_db_ab(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Shared read-only diff 'a'-side: LOCA [BH01 CP, BH02 RC], built once
    per module. The identical + added/removed/modified diff tests both use
    it (diff never mutates its inputs), so the convert runs once not twice."""
    return _write_loca_ags(
        tmp_path_factory.mktemp("diff_a_ab"), "a", [("BH01", "CP"), ("BH02", "RC")]
    )


@pytest.fixture(scope="module")
def diff_db_a1(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Shared read-only diff 'a'-side: LOCA [BH01 CP]. Used by the
    sample-cap + missing-file diff tests."""
    return _write_loca_ags(tmp_path_factory.mktemp("diff_a_a1"), "a", [("BH01", "CP")])


def test_diff_identical_dbs_reports_no_changes(diff_db_ab: Path, tmp_path: Path) -> None:
    a = diff_db_ab
    b = _write_loca_ags(tmp_path, "b", [("BH01", "CP"), ("BH02", "RC")])
    rep = ags5db.diff(a, b)
    assert rep.has_changes is False
    assert rep.changed_groups == []
    assert rep.groups_only_in_a == []
    assert rep.groups_only_in_b == []


def test_diff_detects_added_removed_modified(diff_db_ab: Path, tmp_path: Path) -> None:
    # A has BH01 (CP) + BH02 (RC); B has BH01 (CP) + BH03 (CP) + BH02 (CP).
    #   added in B  : BH03
    #   modified    : BH02 (RC -> CP)
    #   unchanged   : BH01
    a = diff_db_ab
    b = _write_loca_ags(tmp_path, "b", [("BH01", "CP"), ("BH02", "CP"), ("BH03", "CP")])
    rep = ags5db.diff(a, b)
    assert rep.has_changes is True
    loca_diffs = [g for g in rep.changed_groups if g.code == "LOCA"]
    assert len(loca_diffs) == 1
    g = loca_diffs[0]
    assert g.added == 1
    assert g.removed == 0
    assert g.modified == 1
    assert g.unchanged == 1


def test_diff_sample_tuples_capped(diff_db_a1: Path, tmp_path: Path) -> None:
    # 4 added rows; samples=2 should cap to 2.
    a = diff_db_a1
    b = _write_loca_ags(
        tmp_path, "b",
        [("BH01", "CP"), ("BH02", "CP"), ("BH03", "CP"), ("BH04", "CP"), ("BH05", "CP")],
    )
    rep = ags5db.diff(a, b, samples=2)
    g = next(g for g in rep.changed_groups if g.code == "LOCA")
    assert g.added == 4
    assert len(g.sample_added) == 2


def test_diff_missing_file_raises(diff_db_a1: Path, tmp_path: Path) -> None:
    a = diff_db_a1
    with pytest.raises(RuntimeError):
        ags5db.diff(a, tmp_path / "missing.ags5db")


def test_list_blobs_with_attachments(tmp_path: Path) -> None:
    """A fixture that has FILE attachments should return them via list_blobs."""
    # Build an AGS4 file with a FILE group + an attachment on disk.
    ags = tmp_path / "with_attach.ags"
    attach = tmp_path / "report.pdf"
    attach.write_bytes(b"%PDF-1.4 fake pdf bytes")
    lines = [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","blobs test"',
        "",
        '"GROUP","FILE"',
        '"HEADING","FILE_FSET","FILE_NAME","FILE_DESC"',
        '"UNIT","","",""',
        '"TYPE","ID","X","X"',
        '"DATA","FSET1","report.pdf","Test PDF"',
        "",
    ]
    ags.write_bytes(("\r\n".join(lines)).encode("utf-8"))
    db = tmp_path / "with_attach.ags5db"
    ags5db.convert(ags, db, attachments_dir=tmp_path)

    blobs = ags5db.list_blobs(db)
    assert len(blobs) == 1
    assert blobs[0]["parent_code"] == "FILE"
    assert blobs[0]["filename"] == "report.pdf"
    assert blobs[0]["kind"] == "attachment"
    assert blobs[0]["byte_length"] == len(b"%PDF-1.4 fake pdf bytes")

    # Filter by parent_code works.
    only_file = ags5db.list_blobs(db, parent_code="FILE")
    assert len(only_file) == 1
    only_loca = ags5db.list_blobs(db, parent_code="LOCA")
    assert only_loca == []

    # Filter by kind.
    by_kind = ags5db.list_blobs(db, kind="attachment")
    assert len(by_kind) == 1
