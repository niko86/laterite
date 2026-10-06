"""#1008 — ``row_order="key"`` on ``build_ags4`` and ``merge``.

Pinned on the issue's own repro: two deliveries whose locations and samples
interleave in arrival order come out sorted by their KEY headings, and the
sort changes nothing but the order rows are written in.
"""

from __future__ import annotations

import typing
from typing import Any

import laterite
import polars as pl
import pytest
from laterite import _laterite_native as _native

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
MERGE_STAMP = laterite.TranStamp(
    issue="2", date="2026-01-02", producer="A", recipient="B", status="Draft"
)
SAMP_SCHEMA = {
    "LOCA_ID": pl.String,
    "SAMP_TOP": pl.Float64,
    "SAMP_REF": pl.String,
    "SAMP_TYPE": pl.String,
    "SAMP_ID": pl.String,
}


def _build(rows: list[tuple[Any, ...]], **kw: Any) -> bytes:
    """The issue's `build`: LOCA from the samples' locations, then SAMP."""
    loca = pl.DataFrame({"LOCA_ID": sorted({r[0] for r in rows})})
    samp = pl.DataFrame(rows, schema=SAMP_SCHEMA, orient="row")
    frames = [
        ("PROJ", pl.DataFrame({"PROJ_ID": ["P1"]})),
        ("LOCA", loca),
        ("SAMP", samp),
    ]
    return laterite.build_ags4(
        frames, dict_version="4.2", synthesise_metadata=True, tran=STAMP, **kw
    ).bytes


A = [("BH10", 2.0, "2", "D", None), ("BH2", 1.0, "1", "B", None)]
B = [("BH2", 0.5, "1", "ES", None), ("BH1", 3.0, "3", "B", None)]


def _merge(**kw: Any) -> laterite.MergeResult:
    return laterite.merge(
        _build(A), _build(B), dict_version="4.2", tran=MERGE_STAMP, **kw
    )


def _samp(data: bytes) -> list[tuple[Any, ...]]:
    return laterite.read(data)["SAMP"].select("LOCA_ID", "SAMP_TOP", "SAMP_TYPE").rows()


def test_the_issue_repro_comes_out_sorted():
    out = laterite.read(_merge(row_order="key").bytes)
    assert out["LOCA"]["LOCA_ID"].to_list() == ["BH1", "BH2", "BH10"]
    assert _samp(_merge(row_order="key").bytes) == [
        ("BH1", 3.0, "B"),
        ("BH2", 0.5, "ES"),
        ("BH2", 1.0, "B"),
        ("BH10", 2.0, "D"),
    ]


def test_input_is_byte_identical_to_the_default():
    assert _merge(row_order="input").bytes == _merge().bytes
    assert laterite.read(_merge().bytes)["LOCA"]["LOCA_ID"].to_list() == [
        "BH10",
        "BH2",
        "BH1",
    ]
    assert _build(A + B, row_order="input") == _build(A + B)


def test_sorting_never_changes_what_merge_reconciled():
    plain, keyed = _merge(), _merge(row_order="key")
    assert plain.bytes != keyed.bytes
    assert plain.revisions == keyed.revisions
    assert plain.warnings == keyed.warnings
    for code in ("LOCA", "SAMP"):
        a, b = laterite.read(plain.bytes)[code], laterite.read(keyed.bytes)[code]
        assert sorted(a.rows()) == sorted(b.rows()), code


def test_build_sorts_numeric_keys_by_value_and_keeps_ties_in_order():
    rows = [
        ("BH1", 10.0, "1", "B", "ten"),
        ("BH1", 9.5, "1", "B", "nine and a half"),
        ("BH1", 9.5, "1", "B", "tie, second"),
    ]
    data = _build(rows, row_order="key")
    ids = laterite.read(data)["SAMP"]["SAMP_ID"].to_list()
    # 10.00 sorts after 9.50 (as text it would come first); the two 9.50 rows
    # tie on every key and keep their order.
    assert ids == ["nine and a half", "tie, second", "ten"]


def test_an_unknown_row_order_is_a_value_error():
    with pytest.raises(ValueError, match="unknown row_order 'sorted'"):
        _build(A, row_order="sorted")
    a = _build(A)
    with pytest.raises(ValueError, match="unknown row_order 'sorted'"):
        laterite.merge(a, a, row_order="sorted")


def test_lat_merge_row_order(capsys: Any, tmp_path: Any) -> None:
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_bytes(_build(A))
    b.write_bytes(_build(B))

    def run(*extra: str) -> list[str]:
        out = tmp_path / "out.ags"
        argv = ["merge", str(a), str(b), "--out", str(out), *extra]
        assert _cli.main(argv) == 0, capsys.readouterr().err
        return laterite.read(out.read_bytes())["LOCA"]["LOCA_ID"].to_list()

    assert run() == ["BH10", "BH2", "BH1"]
    assert run("--row-order", "key") == ["BH1", "BH2", "BH10"]
    capsys.readouterr()
    # A bad flag value is a usage error: exit 5, as on the other launchers.
    bad = ["merge", str(a), str(b), "--out", str(tmp_path / "x.ags")]
    assert _cli.main([*bad, "--row-order", "sorted"]) == 5
    assert "--row-order" in capsys.readouterr().err


def test_the_literal_and_the_cli_choices_match_the_engine():
    """`RowOrderMode` cannot derive itself, so it is pinned to `RowOrder::ALL`."""
    from laterite import _cli

    authority = list(_native.registry_row_order_modes())
    assert authority == ["input", "key"]
    assert list(typing.get_args(laterite.RowOrderMode)) == authority
    assert list(_cli._ROW_ORDER_CHOICES) == authority
