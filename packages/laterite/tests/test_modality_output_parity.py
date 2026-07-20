"""The modality OUTPUT gate — same bytes, three doors, one answer.

`test_modality_parity` asks whether a capability is *offered* in each I/O form.
`test_cross_surface_parity` / `test_free_chained_parity` compare the *names* of the
knobs each surface exposes. Neither ever compares an **answer**. So a modality that
accepts the same file and the same flags and then returns a *different verdict* sits
in the blind spot of every gate we own — which is exactly where this bug was living:

    TRAN_AGS says "4.0.3", but the file uses LOCA_NATD (a 4.0.4-only heading)

    path  -> judged against 4.0.4, 3 findings   <- guard_4_0_4 (O-42) ran
    bytes -> judged against 4.0.3, 5 findings   <- it didn't
    text  -> judged against 4.0.3, 5 findings   <- it didn't

Two phantom Rule 9 findings, on every bytes read, because `laterite-py`,
`laterite-node` and wasm each hand-assembled "resolve the edition, then run the
rules" and each left the content guard out. The knob names matched perfectly.

This gate asserts the thing that actually matters: for the same bytes and the same
options, `read(path)`, `read(text)` and `read(bytes)` must agree on the edition, the
resolution, and every single finding.
"""

from __future__ import annotations

import laterite as lat
import pytest

# --- fixtures: each is (name, ags4 text) ------------------------------------

# THE REGRESSION. TRAN_AGS declares 4.0.3; LOCA_NATD was introduced in 4.0.4. The
# O-42 content guard judges it against 4.0.4 so its newer vocabulary isn't
# false-flagged as non-standard, and emits an FYI saying so. A modality that skips
# the guard reports LOCA_NATD as an unknown heading — a finding about the validator,
# not about the file.
MISLABELLED_4_0_3 = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        "",
        '"GROUP","TRAN"',
        '"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON"',
        '"UNIT","","yyyy-mm-dd","","","","","",""',
        '"TYPE","X","DT","X","X","X","X","X","X"',
        '"DATA","1","2020-08-18","ACME Drilling Ltd","Draft","4.0.3","ACME Consulting","|","+"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATD"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","BH1","x"',
        "",
    ]
)

# A file that is clean at its declared edition — the control. If the gate only ever
# ran on a broken file it would not notice a modality that reports findings on a good
# one.
CLEAN_4_2 = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Clean minimal AGS4 fixture"',
        "",
        '"GROUP","TRAN"',
        '"HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON"',
        '"UNIT","","yyyy-mm-dd","","","","","",""',
        '"TYPE","X","DT","X","X","X","X","X","X"',
        '"DATA","1","2020-08-18","ACME Drilling Ltd","Draft","4.2","ACME Consulting","|","+"',
        "",
        '"GROUP","UNIT"',
        '"HEADING","UNIT_UNIT","UNIT_DESC"',
        '"UNIT","",""',
        '"TYPE","X","X"',
        '"DATA","yyyy-mm-dd","year month day"',
        "",
        '"GROUP","TYPE"',
        '"HEADING","TYPE_TYPE","TYPE_DESC"',
        '"UNIT","",""',
        '"TYPE","X","X"',
        '"DATA","ID","Unique identifier"',
        '"DATA","X","Text"',
        '"DATA","DT","Date and time"',
        "",
    ]
)

# A file the engine has plenty to say about, at every tier — so the gate compares a
# long finding list, not just two zeros.
MESSY_4_2 = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_ZZZZ"',
        '"UNIT","",""',
        '"TYPE","ID","2DP"',
        '"DATA","P1","not-a-number"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE"',
        '"UNIT","","m"',
        '"TYPE","ID","2DP"',
        '"DATA","BH1","12.345"',
        "",
    ]
)

FIXTURES = [
    ("mislabelled_4_0_3", MISLABELLED_4_0_3),
    ("clean_4_2", CLEAN_4_2),
    ("messy_4_2", MESSY_4_2),
]

# The tier combinations a caller can ask for. The guard's FYI only surfaces with
# fyi=True, so a gate that never asked for FYI would have missed half of this bug.
TIERS = [
    pytest.param(False, False, id="errors-only"),
    pytest.param(True, False, id="warnings"),
    pytest.param(True, True, id="warnings+fyi"),
]


def _findings(report) -> list[tuple]:
    """Every finding as a comparable tuple. `report.findings` is a dataframe, so go
    through `by_rule()` — the plain-dict view — and fold the rule back in."""
    rows = [
        (rule, f.get("line"), f.get("group"), f.get("desc"), f.get("severity"))
        for rule, items in report.by_rule().items()
        for f in items
    ]
    return sorted(rows, key=lambda r: tuple("" if x is None else str(x) for x in r))


def _answer(report) -> dict:
    """Everything the caller can observe about a verdict, EXCEPT the source label
    (`<bytes>` vs the path — the one field that is *supposed* to differ)."""
    return {
        "dict_version": report.dict_version,
        "resolution": report.resolution,
        "count": report.count,
        "is_valid": report.is_valid,
        "exit_code": report.exit_code,
        # The findings themselves, not just how many: a modality could land on the
        # right count with the wrong findings.
        "findings": _findings(report),
    }


@pytest.mark.parametrize(("name", "text"), FIXTURES, ids=[n for n, _ in FIXTURES])
@pytest.mark.parametrize(("warnings", "fyi"), TIERS)
def test_path_text_and_bytes_return_the_same_verdict(
    tmp_path, name, text, warnings, fyi
):
    src = tmp_path / f"{name}.ags"
    src.write_bytes(text.encode("utf-8"))  # write_bytes: keep CRLF (no translation)

    from_path = _answer(lat.read(src).validate(warnings=warnings, fyi=fyi).report)
    from_text = _answer(lat.read(text).validate(warnings=warnings, fyi=fyi).report)
    from_bytes = _answer(
        lat.read(text.encode("utf-8")).validate(warnings=warnings, fyi=fyi).report
    )

    assert from_text == from_path, f"text disagrees with path on {name}"
    assert from_bytes == from_path, f"bytes disagrees with path on {name}"


def test_the_4_0_3_guard_reaches_every_modality(tmp_path):
    """The specific regression, asserted by value rather than by agreement — so the
    gate still bites if all three modalities were to break the same way."""
    src = tmp_path / "mislabelled.ags"
    src.write_bytes(MISLABELLED_4_0_3.encode("utf-8"))

    for label, source in (
        ("path", src),
        ("text", MISLABELLED_4_0_3),
        ("bytes", MISLABELLED_4_0_3.encode("utf-8")),
    ):
        rep = lat.read(source).validate(fyi=True).report
        assert rep.dict_version == "4.0.4", (
            f"{label}: TRAN_AGS says 4.0.3 but the file uses a 4.0.4-only heading — "
            f"the O-42 guard must judge it against 4.0.4, got {rep.dict_version}"
        )
        assert rep.resolution == "guessed", f"{label}: resolution {rep.resolution!r}"
        # ...and the file is told why it was judged against an edition it doesn't claim.
        assert any(
            "4.0.4" in (desc or "") and severity == "fyi"
            for _rule, _line, _group, desc, severity in _findings(rep)
        ), f"{label}: the transparency FYI (#222 / O-42) is missing"
