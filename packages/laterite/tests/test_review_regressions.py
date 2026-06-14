"""Regression tests for fixes shipped in the recent code-review pass.

Covers four things that lacked direct test coverage at the time the bugs
were either discovered or first introduced:

1. L-group CSV filename collision on multi-LOCA non-SAMP groups (MOND).
   Pre-fix, every parent's chunks collided on `MOND__0000.csv` and only
   the last LOCA's data survived the .agsx write.

2. `read_ags5db` tolerating views missing from older files (newer
   library vs file written by an older registry).

3. The shared `parse_value` returning None on unparseable numeric input
   instead of raising — real AGS files use sentinels like "NA" in
   numeric columns.

4. Model field types matching the canonical type system (0DP -> int,
   YN -> bool, DT -> datetime). Catches future drift between
   `_modelgen._py_type` and `canonical_type`.
"""

from __future__ import annotations

import datetime as _dt

import laterite
import pytest
from laterite.ags_types import parse_value

# ---------------------------------------------------------------------------
# 1. L-group multi-LOCA round-trip (the MOND silent-data-loss bug)
#
# Originally regressed against the .agsx writer's L-group CSV filename
# collision. Stage F2a retired the .ags5db ↔ .agsx pipeline; the
# multi-LOCA MOND regression now lives in `tests/test_ags4_to_agsx.py`
# (added in F2a-4) exercising the Python AGS4 → .agsx helper instead.
# ---------------------------------------------------------------------------


# ---------------------------------------------------------------------------
# 2. Reader tolerates views missing from older files
# ---------------------------------------------------------------------------


# `test_read_ags5db_tolerates_missing_view` retired with F2c-3.
#
# The old Python `read_ags5db` caught `duckdb.CatalogException` on each
# group's view query and emitted a warning, so old files written by a
# library that didn't know a particular group could still be read
# (skipping that group). The Rust `laterite.ags5db.read_db` queries
# `_spec_groups` first to know what views to expect; a missing view
# for a code listed in `_spec_groups` is a corrupt-file signal and is
# now a hard error. The "older library wrote this file" scenario the
# test simulated isn't reachable through any supported workflow now
# that the Python writer is gone, so the test loses its premise.


# ---------------------------------------------------------------------------
# 3. parse_value coercion edge cases (real-world AGS sentinels)
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("ags_type,raw,expected", [
    # Numeric sentinels -> None instead of raising
    ("2DP", "NA",    None),
    ("0DP", "<LOD",  None),
    ("3SF", "-",     None),
    ("1SCI", "n/a",  None),
    # Empty / whitespace -> None
    ("2DP", "",      None),
    ("2DP", "   ",   None),
    ("X",   "",      None),
    # Valid numeric still parses
    ("2DP", "12.34", 12.34),
    ("0DP", "5.0",   5),         # int via float() to tolerate "5.0"
    ("0DP", "5",     5),
    # Strings pass through
    ("X",   "hello", "hello"),
    ("ID",  "BH01",  "BH01"),
    # YN -> bool
    ("YN",  "Y",     True),
    ("YN",  "N",     False),
    ("YN",  "y",     True),
    ("YN",  "bogus", None),
    # DT -> datetime
    ("DT",  "2024-03-15 09:30",      _dt.datetime(2024, 3, 15, 9, 30)),
    ("DT",  "2024-03-15",            _dt.datetime(2024, 3, 15)),
    ("DT",  "not a date",            None),
])
def test_parse_value_handles_real_world_inputs(ags_type, raw, expected):
    assert parse_value(raw, ags_type) == expected


def test_parse_value_unknown_ags_type_passes_string_through():
    """Passthrough groups carry unknown type codes; parse_value should
    return the raw string rather than raising."""
    assert parse_value("anything", "ZZZ") == "anything"
    assert parse_value("", "ZZZ") is None


# `test_modelgen_uses_canonical_type_mapping` retired with F2c-4
# (ags5-models gone). The Rust typed-graph engine's class generation
# in `build.rs` consumes the same `canonical_type` mapping the
# laterite Python wrapper exposes, and the F2b-6a `.pyi` drift test
# (`tests/test_pyi_stubs_match_generator.py`) catches the equivalent
# drift one layer up.


# ---------------------------------------------------------------------------
# 4. AGS3 input is refused through the real engine (O-30)
#
# python-ags4 silently falls an AGS3 file back to AGS4.1.1 and validates it
# against an AGS4 schema; the clean-room engine deliberately refuses
# (UnsupportedEdition, exit code 4) once it sees the unambiguous AGS3
# markers (`**GROUP` / `<UNITS>` / `<CONT>`). The previous coverage only
# hit the hardcoded compat stub, not the engine's parse-level detection.
# ---------------------------------------------------------------------------

# Minimal AGS3-shaped text: `**PROJ` group + `<UNITS>` marker (mirrors the
# Rust `parse::tests::ags3_is_unsupported_edition*` fixture).
_AGS3_TEXT = (
    '"**PROJ"\r\n'
    '"*PROJ_ID","*PROJ_NAME","*PROJ_AGS"\r\n'
    '"<UNITS>","",""\r\n'
    '"P001","Demo","3.1"\r\n'
)


@pytest.mark.parametrize("entry", ["read", "validate"])
def test_ags3_input_is_refused_with_exit_code_4(entry):
    """AGS3 markers trigger UnsupportedEditionError (exit 4), not a silent
    AGS4 validation — through both the read and validate entry points."""
    fn = getattr(laterite, entry)
    with pytest.raises(laterite.UnsupportedEditionError) as exc:
        fn(text=_AGS3_TEXT)
    assert exc.value.exit_code == 4
    # Names the refused edition (clean-room "3.x", not the vague NotAgs4).
    assert "3" in str(exc.value)


def test_ags3_is_not_the_generic_notags4_error():
    """AGS3 must surface as the specific UnsupportedEditionError, not the
    generic NotAgs4Error a structureless file gets — they share exit code
    4 but the distinction is the O-30 point."""
    with pytest.raises(laterite.UnsupportedEditionError):
        laterite.validate(text=_AGS3_TEXT)
    # A genuinely structureless file is still the generic NotAgs4.
    with pytest.raises(laterite.NotAgs4Error):
        laterite.read(text="nope\nstill nope\n")


# ---------------------------------------------------------------------------
# 5. Non-UTF-8 bytes decode lossily, not refused (O-32)
#
# The Rust validator used to hard-fail non-UTF-8 input as `NotUtf8` (zero
# rules evaluated — a black hole). O-32 reconciled this with python-ags4's
# `errors="replace"`: an undecodable byte becomes U+FFFD and the file still
# validates, surfacing a Rule 1 finding (U+FFFD > 255 → not AGS ASCII). A
# future revert to hard-refuse would pass CI without this guard. Bytes can
# only enter through a file path (the text= route is already a Python str).
# ---------------------------------------------------------------------------

# AGS4 with a lone cp1252 byte (0xB0, the degree sign) in a DATA value —
# invalid as UTF-8, the exact shape O-32 documents from the real corpus.
_NON_UTF8_BYTES = (
    b'"GROUP","PROJ"\r\n'
    b'"HEADING","PROJ_ID","PROJ_NAME"\r\n'
    b'"UNIT","",""\r\n'
    b'"TYPE","ID","X"\r\n'
    b'"DATA","P1","temp 20\xb0C site"\r\n'
)


def test_non_utf8_validate_reports_rule_1_without_raising(tmp_path):
    """Invalid UTF-8 input is decoded lossily and reported under Rule 1,
    not refused with a raise (O-32)."""
    bad = tmp_path / "cp1252.ags"
    bad.write_bytes(_NON_UTF8_BYTES)
    rep = laterite.validate(str(bad))  # must not raise
    assert "AGS Format Rule 1" in rep.by_rule()


def test_non_utf8_read_substitutes_u_fffd(tmp_path):
    """read() decodes the bad byte to U+FFFD rather than raising — the
    replacement char lands in the parsed value."""
    bad = tmp_path / "cp1252.ags"
    bad.write_bytes(_NON_UTF8_BYTES)
    f = laterite.read(str(bad))  # must not raise
    proj_name = f["PROJ"]["PROJ_NAME"].to_list()
    assert proj_name == ["temp 20�C site"]
