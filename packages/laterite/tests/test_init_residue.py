"""Coverage campaign (P3 residue): the remaining `laterite/__init__.py` arms.

The public surface's less-trodden branches — the `xn="numeric"` no-op cases
(a group with no XN headings, a passthrough group absent from the dictionary,
XN headings not present in the frame), the pandas / Arrow-capsule variants of
the synthetic-key drop, and certifying a handle read from raw `data=` bytes.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import laterite as L
import pandas as pd
import polars as pl
import pytest

_ROOT = Path(__file__).resolve().parents[3]
_CLEAN = (
    _ROOT
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
    / "clean_minimal.ags"
)


def _ags(*lines: str) -> str:
    return "\r\n".join(lines) + "\r\n"


_PROJ = _ags(
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
)


# --- xn="numeric": the `_xn_select` no-op arms ------------------------------


def test_xn_numeric_group_without_xn_headings() -> None:
    # PROJ has no XN-typed headings, so the numeric projection is a plain `*`.
    out = L.read(text=_PROJ, xn="numeric").query("SELECT * FROM PROJ").to_polars()
    assert out.height == 1


def test_xn_numeric_passthrough_group_absent_from_dictionary() -> None:
    # A non-standard group code isn't in the registry (`_GROUPS.get` is None), so
    # there's no XN info to cast against — the projection stays `*`.
    zzzz = _ags(
        '"GROUP","ZZZZ"',
        '"HEADING","ZZZZ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","z1"',
    )
    out = L.read(text=zzzz, xn="numeric").query("SELECT * FROM ZZZZ").to_polars()
    assert out.height == 1


def test_xn_numeric_group_missing_its_xn_column() -> None:
    # FRAC declares an XN heading (FRAC_FI) but this file omits it, so the set of
    # XN columns actually present is empty — again a `*` no-op.
    frac = _ags(
        '"GROUP","FRAC"',
        '"HEADING","LOCA_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","BH1"',
    )
    out = L.read(text=frac, xn="numeric").query("SELECT * FROM FRAC").to_polars()
    assert out.height == 1


# --- synthetic-key drop across frame kinds ----------------------------------


def test_build_drops_synth_key_columns_pandas() -> None:
    # The pandas branch of _drop_synth_keys (the polars branch is covered
    # elsewhere): a `_id` synthetic column must never reach the emitted AGS4.
    frame = pd.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["X"], "_id": [0]})
    br = L.build_ags4({"PROJ": frame}, synthesise_metadata=True)
    assert "_id" not in br.text


class _CapsuleOnly:
    """A frame that exposes only the Arrow C-stream capsule (no `.columns`) —
    the non-columnar branch of _drop_synth_keys returns it untouched."""

    def __init__(self, frame: Any) -> None:
        self._f = frame

    def __arrow_c_stream__(self, requested_schema: Any = None) -> Any:
        return self._f.__arrow_c_stream__(requested_schema)


def test_build_from_capsule_only_frame() -> None:
    frame = _CapsuleOnly(pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["X"]}))
    br = L.build_ags4({"PROJ": frame}, synthesise_metadata=True)
    assert '"GROUP","PROJ"' in br.text


# --- certify from a data= handle --------------------------------------------


def test_certify_data_handle_uses_original_bytes(tmp_path: Path) -> None:
    # A handle read from raw bytes returns those bytes as its cert source
    # (the `data is not None` arm of _source_bytes), not a re-emit.
    handle = L.read(data=_CLEAN.read_bytes())
    dest = handle.certify(str(tmp_path / "clean.ags.idx"))
    assert Path(dest).exists()


# --- to_excel error translation (No valid AGS4 data -> NotAgs4Error) --------


def test_to_excel_file_output_on_non_ags_raises(tmp_path: Path) -> None:
    # The file-writing converter maps the engine's no-data RuntimeError to
    # NotAgs4Error (the _excel_convert arm).
    garbage = tmp_path / "garbage.ags"
    garbage.write_text("not ags4 at all\r\n")

    with pytest.raises(L.NotAgs4Error):
        L.to_excel(str(garbage), str(tmp_path / "out.xlsx"))


def test_to_excel_bytes_mode_on_non_ags_raises(tmp_path: Path) -> None:
    # The in-memory twin (_excel_bytes_convert) makes the same translation.
    garbage = tmp_path / "garbage.ags"
    garbage.write_text("not ags4 at all\r\n")

    with pytest.raises(L.NotAgs4Error):
        L.to_excel(str(garbage))


def test_from_excel_non_xlsx_path_reraises(tmp_path: Path) -> None:
    # A file that isn't a workbook fails to open with a RuntimeError that is not
    # "No valid AGS4 data", so _excel_convert re-raises it unchanged.
    bad = tmp_path / "bad.xlsx"
    bad.write_text("not a workbook\r\n")
    with pytest.raises(RuntimeError):
        L.from_excel(str(bad), str(tmp_path / "out.ags"))


def test_from_excel_non_xlsx_bytes_reraises(tmp_path: Path) -> None:
    # The bytes twin (_excel_bytes_convert) re-raises the same open error.
    with pytest.raises(RuntimeError):
        L.from_excel(b"not a workbook", str(tmp_path / "out.ags"))
