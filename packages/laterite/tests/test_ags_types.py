"""Unit + property tests for `laterite.ags_types`.

`canonical_type` / `display_hint` are thin Python wrappers over the Rust
`laterite-ags4-core` type system; the value here is pinning the *contract* the
Python callers depend on (the `CanonicalType` identity, the `ValueError`
on unknown codes, the printf-style display hints) plus a totality
property over `parse_value` — the single ingest entry point that must
never raise on arbitrary AGS4-shaped text.
"""

from __future__ import annotations

import datetime as _dt

import pytest
from hypothesis import given
from hypothesis import strategies as st
from laterite.ags_types import (
    CanonicalType,
    canonical_type,
    display_hint,
    parse_value,
)

# ---------------------------------------------------------------------------
# canonical_type
#
# NOTE on the "all 8 enum members" coverage goal: only FIVE of the eight
# `CanonicalType` members are reachable through `canonical_type(ags_type)`.
# The Rust mapping (`laterite-ags4-types::canonical_type`) routes AGS4 type codes to
# STRING / INTEGER / DECIMAL / DATETIME / BOOL only — DATE / TIME / ENUM
# exist in the enum for the type system's completeness (DuckDB `sql_type`
# DATE/TIME/VARCHAR storage) but no AGS4 type code resolves to them. We
# cover every reachable resolution below and separately pin the enum's
# label round-trip for the three storage-only members.
# ---------------------------------------------------------------------------

# (ags_type, expected CanonicalType) — one representative per code family.
_KNOWN_RESOLUTIONS = [
    ("X", CanonicalType.STRING),
    ("ID", CanonicalType.STRING),
    ("PA", CanonicalType.STRING),
    ("PT", CanonicalType.STRING),
    ("PU", CanonicalType.STRING),
    ("U", CanonicalType.STRING),
    ("T", CanonicalType.STRING),
    ("MC", CanonicalType.STRING),
    ("DMS", CanonicalType.STRING),
    ("XN", CanonicalType.STRING),
    ("0DP", CanonicalType.INTEGER),
    ("2DP", CanonicalType.DECIMAL),
    ("3DP", CanonicalType.DECIMAL),
    ("3SF", CanonicalType.DECIMAL),
    ("1SCI", CanonicalType.DECIMAL),
    # RL is a delimited RECORD LINK (`GROUP|KEY1|KEY2`, AGS Rule 11) — text, not a
    # number. This row asserted DECIMAL, which is how the bug survived: parsing a
    # link as a float yields Null, so every RL column read back as an all-null f64
    # and the link was destroyed (#503).
    ("RL", CanonicalType.STRING),
    ("DT", CanonicalType.DATETIME),
    ("YN", CanonicalType.BOOL),
]


@pytest.mark.parametrize(("ags_type", "expected"), _KNOWN_RESOLUTIONS)
def test_canonical_type_resolves_known_codes(ags_type, expected):
    result = canonical_type(ags_type)
    # `is` — the StrEnum identity callers rely on (`x is CanonicalType.DECIMAL`).
    assert result is expected


def test_canonical_type_covers_every_reachable_member():
    """The five members `canonical_type` can actually return."""
    reachable = {canonical_type(code) for code, _ in _KNOWN_RESOLUTIONS}
    assert reachable == {
        CanonicalType.STRING,
        CanonicalType.INTEGER,
        CanonicalType.DECIMAL,
        CanonicalType.DATETIME,
        CanonicalType.BOOL,
    }


def test_canonical_type_storage_only_members_exist_and_round_trip():
    """DATE / TIME / ENUM are storage-only (no AGS code maps to them) but
    must still be construct-able from their lowercase label — the
    `_spec_headings` interchange representation."""
    assert CanonicalType("date") is CanonicalType.DATE
    assert CanonicalType("time") is CanonicalType.TIME
    assert CanonicalType("enum") is CanonicalType.ENUM
    # All eight members carry their lowercase-label value.
    assert {m.value for m in CanonicalType} == {
        "string",
        "integer",
        "decimal",
        "datetime",
        "date",
        "time",
        "bool",
        "enum",
    }


@pytest.mark.parametrize("bad", ["ZZZ", "", "  ", "FLOAT", "date", "enum", "2D", "DPP"])
def test_canonical_type_raises_on_unknown_code(bad):
    with pytest.raises(ValueError, match="unknown AGS type code"):
        canonical_type(bad)


# ---------------------------------------------------------------------------
# display_hint
# ---------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("ags_type", "expected"),
    [
        ("2DP", "%.2f"),
        ("0DP", "%.0f"),
        ("5DP", "%.5f"),
        ("3SF", "%.3g"),
        ("1SF", "%.1g"),
        ("1SCI", "%.1e"),
        ("2SCI", "%.2e"),
    ],
)
def test_display_hint_numeric_codes(ags_type, expected):
    assert display_hint(ags_type) == expected


@pytest.mark.parametrize("ags_type", ["X", "ID", "PA", "DT", "YN", "U", "T", "ZZZ", ""])
def test_display_hint_non_numeric_is_none(ags_type):
    assert display_hint(ags_type) is None


@pytest.mark.parametrize(
    "ags_type", ["2DP", "0DP", "5DP", "3SF", "1SF", "1SCI", "2SCI"]
)
def test_display_hint_returns_usable_format_string(ags_type):
    """The returned hint must be a working printf-style format — applying
    it to a float must not raise and must produce a string."""
    fmt = display_hint(ags_type)
    assert fmt is not None
    out = fmt % 1.5
    assert isinstance(out, str)
    # And round-trips a representative value without error.
    assert isinstance(fmt % -0.0, str)
    assert isinstance(fmt % 123456.789, str)


# ---------------------------------------------------------------------------
# parse_value — totality property
#
# `parse_value` is the single source of truth for AGS4 ingest. It
# is *permissive by contract*: unparseable input returns None, it never
# raises. The property below asserts that totality + the type-shape
# invariant (the return is always one of the documented native types)
# over generated (raw, ags_type) pairs. We deliberately do NOT assert
# exact parsed values for arbitrary text — only the totality + shape.
# ---------------------------------------------------------------------------

# Real AGS type codes (covering every resolution branch) + deliberate junk.
_AGS_TYPE_CODES = [
    "X",
    "ID",
    "PA",
    "PT",
    "PU",
    "U",
    "T",
    "MC",
    "DMS",
    "XN",
    "RL",  # string
    "0DP",  # integer
    "2DP",
    "3DP",
    "5DP",
    "1SF",
    "3SF",
    "1SCI",
    "2SCI",  # decimal
    "DT",  # datetime
    "YN",  # bool
    "ZZZ",
    "",
    "   ",
    "FLOAT",
    "2D",
    "garbage",
    "🙂",  # junk / passthrough
]

# The closed set of types parse_value is documented to return.
_ALLOWED_RETURN_TYPES = (
    type(None),
    int,
    float,
    bool,
    _dt.datetime,
    _dt.date,
    _dt.time,
    str,
)

_RAW_STRATEGY = st.one_of(
    st.none(),
    st.text(),
    # Bias towards inputs that *look* parseable so the numeric / datetime /
    # bool branches are actually exercised, not just the None-on-junk path.
    st.sampled_from(
        [
            "",
            "  ",
            "0",
            "5",
            "5.0",
            "-3.14",
            "1e9",
            "NaN",
            "inf",
            "-inf",
            "1,234",
            "  42  ",
            "Y",
            "N",
            "y",
            "n",
            "true",
            "TRUE",
            "1",
            "2024-03-15",
            "2024-03-15 09:30",
            "2024-03-15 09:30:00",
            "15/03/2024",
            "not a date",
            "<LOD",
            "NA",
            "n/a",
            "-",
        ]
    ),
)


@given(raw=_RAW_STRATEGY, ags_type=st.sampled_from(_AGS_TYPE_CODES))
def test_parse_value_is_total_and_type_shaped(raw, ags_type):
    """parse_value never raises and always returns one of the documented
    native types (None | int | float | bool | datetime | date | time | str)."""
    result = parse_value(raw, ags_type)
    assert isinstance(result, _ALLOWED_RETURN_TYPES)


@given(raw=st.text())
def test_parse_value_string_codes_return_str_or_none(raw):
    """For an `X`/`ID` (string) code the result is always `str` or `None`
    — never a number/bool/datetime. (NB: the empty→None boundary uses the
    Rust `.trim()` Unicode-whitespace set, which is NARROWER than Python's
    `str.strip()`: e.g. `'\\x1f'` is stripped by Python but not by Rust, so
    we don't assert Python-strip equivalence here.)"""
    result = parse_value(raw, "X")
    assert result is None or isinstance(result, str)
    # bool is an int subclass, but the string branch can never yield one.
    assert not isinstance(result, bool)


@given(
    raw=st.text(alphabet=st.characters(min_codepoint=33, max_codepoint=126), min_size=1)
)
def test_parse_value_string_code_non_blank_ascii_round_trips_to_str(raw):
    """A non-blank printable-ASCII value under a string code always parses
    back to a non-None `str` (the common, well-defined case)."""
    result = parse_value(raw, "X")
    assert isinstance(result, str)
    assert result == raw


@given(
    n=st.integers(min_value=-(10**12), max_value=10**12),
)
def test_parse_value_integer_code_round_trips(n):
    """An `0DP` (integer) code parses an integer-valued string back to int."""
    result = parse_value(str(n), "0DP")
    assert result == n
    assert isinstance(result, int)


# ---------------------------------------------------------------------------
# parse_value — exact canonical values (the #531 single-source guard)
#
# The PyO3 wrapper no longer re-implements the format tables / typed parsers:
# it dispatches on `canonical_type` and calls the SAME `parse_datetime` /
# `parse_date` / `parse_time` / `parse_bool` that back the Rust leaf's own
# `parse_value` (the one that feeds `_content_hash`). These pins mirror the
# leaf's `parse_value_canonical_form_is_pinned_...` test through the Python
# return-type mapping, so a drift on either side of that one source is loud.
# ---------------------------------------------------------------------------


def test_parse_value_datetime_exact_values():
    # Full datetime.
    assert parse_value("2020-08-18 09:30:00", "DT") == _dt.datetime(
        2020, 8, 18, 9, 30, 0
    )
    # Date-only DT → promoted to midnight (the shared parse_datetime rule).
    assert parse_value("2020-08-18", "DT") == _dt.datetime(2020, 8, 18, 0, 0, 0)
    # dd/mm/yyyy date-only under DT normalises the same way.
    assert parse_value("15/03/2024", "DT") == _dt.datetime(2024, 3, 15, 0, 0, 0)
    # Unparseable → None (permissive, never raises).
    assert parse_value("not a date", "DT") is None


def test_parse_value_bool_exact_values():
    for token in ("Y", "YES", "TRUE", "1", "y", "true"):
        assert parse_value(token, "YN") is True
    for token in ("N", "NO", "FALSE", "0", "n", "false"):
        assert parse_value(token, "YN") is False
    assert parse_value("maybe", "YN") is None


def test_parse_value_numeric_exact_values():
    # Decimal → float; trailing-zero precision is a float, not the string.
    assert parse_value("10.00", "2DP") == 10.0
    assert isinstance(parse_value("10.00", "2DP"), float)
    # Integer → int; "5.0" notation tolerated.
    assert parse_value("5.0", "0DP") == 5
    assert isinstance(parse_value("5.0", "0DP"), int)


def test_parse_value_record_link_stays_string():
    # RL is a delimited record LINK (text), never a number — the #503 guard.
    assert parse_value("SAMP|BH01|1.00", "RL") == "SAMP|BH01|1.00"
    assert isinstance(parse_value("SAMP|BH01|1.00", "RL"), str)
