"""The landing demo's seeded delivery must fail in exactly the four ways the
page narrates, and in no others (#393).

The demo's whole argument is "here is a real delivery, and here is what the
engine thinks of it". That argument dies if the engine thinks a dozen things the
page never mentions — the reader cannot tell the seeded lesson from the noise,
and concludes the validator is fussy rather than useful.

The delivery drawn in the site design did exactly that. Run its emitted file
through the shipped validator and rules **7, 8, 10a (x5), 10c, 14, 15, 16 and
17** fire, of which only Rule 8 is deliberate. Worse, the most interesting seeded
defect — a lab row pointing at a sample that does not exist, the one the page
captions "needs a human" — **never fired**: `LLPL` was drawn with four of its
seven KEY headings missing, so Rule 10a reported the missing columns and Rule 10c
degraded to `Could not check parent entries: KEY fields missing in LLPL or SAMP`.
The orphan story was unreachable, and nothing said so.

So this asserts the finding SET by identity, not a count — a count passes for the
wrong reasons the moment one rule stops firing and another starts. And it asserts
10c reports an unmatched parent key rather than an unrunnable check, because
those two are the same number and opposite meanings.

The fixture is committed rather than generated: it is the page's copy as much as
its data, and a generator would put the seeded defects one indirection away from
the test that pins them.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
DELIVERY = REPO / "web" / "landing" / "demo" / "seeded-delivery.ags"
SCHEMA_TS = REPO / "web" / "landing" / "demo" / "schema.ts"
SYNC = REPO / "web" / "scripts" / "sync-demo-schema.mjs"

#: The chain the page draws, parent first.
CHAIN = ["PROJ", "LOCA", "SAMP", "LLPL"]

#: The four seeded defects, as (rule, group, what it must be about). Rule 16
#: appears TWICE on purpose: the bad `SAMP_TYPE` value is part of `LLPL`'s key
#: tuple, so one bad cell is two findings. The page shows both rather than
#: deduping, because that duplication is the repeated-KEY lesson firing live.
SEEDED = {
    ("AGS Format Rule 8", "LOCA", "LOCA_GL"),
    ("AGS Format Rule 16", "SAMP", "SAMP_TYPE"),
    ("AGS Format Rule 16", "LLPL", "SAMP_TYPE"),
    ("AGS Format Rule 14", "TRAN", None),
    ("AGS Format Rule 10c", "LLPL", None),
}


@pytest.fixture(scope="module")
def report():
    laterite = pytest.importorskip("laterite", reason="needs the built wheel")
    return laterite.validate(str(DELIVERY))


def _identity(row: dict) -> tuple[str, str, str | None]:
    """(rule, group, subject) — the subject being the heading a cell finding
    names, or the heading a line finding quotes, or None."""
    rule, group, desc = row["rule"], row["group"], row["desc"]
    if row["heading"]:
        return (rule, group, row["heading"])
    for token in desc.replace('"', " ").split():
        if "_" in token and token.isupper():
            return (rule, group, token)
    return (rule, group, None)


def test_the_delivery_fails_in_exactly_the_seeded_ways(report) -> None:
    """The acceptance criterion, by identity. Falsify by deleting the `ABBR`
    group from the fixture: rule 16 stops firing and rule 17 starts, and a
    count-based assertion would still see five."""
    got = {_identity(r) for r in report.findings.to_dicts()}
    assert got == SEEDED, (
        "the seeded delivery no longer fails in exactly the four ways the page "
        f"narrates\n  unexpected: {sorted(map(str, got - SEEDED))}\n"
        f"  missing:    {sorted(map(str, SEEDED - got))}"
    )


def test_the_orphan_reports_as_an_unmatched_parent_not_an_unrunnable_check(
    report,
) -> None:
    """The regression that made the page's best story unreachable. Both states
    are "Rule 10c on LLPL"; only the wording separates them."""
    tenc = [r for r in report.findings.to_dicts() if r["rule"].endswith("10c")]
    assert len(tenc) == 1, f"expected one Rule 10c finding, got {len(tenc)}"
    desc = tenc[0]["desc"]
    assert "No parent entry in SAMP" in desc, (
        f"Rule 10c did not match parent keys at all — it said {desc!r}. A KEY "
        "heading has gone missing from LLPL or SAMP and the orphan story is dead."
    )
    assert "Could not check" not in desc
    assert "BH02" in desc, "the orphan named is not the seeded one"


def test_every_seeded_defect_is_an_error(report) -> None:
    """The design handoff captions the `SAMP_TYPE` defect a warning. The engine
    calls it an error, and the engine is the authority — #397 renders severity
    from engine output, so a caption that disagreed would be visibly wrong."""
    sev = {r["severity"] for r in report.findings.to_dicts()}
    assert sev == {"error"}, f"expected every seeded finding to be an error, got {sev}"


# --- the fixture against the registry ------------------------------------


def _drawn_headings() -> dict[str, list[str]]:
    """The HEADING row each group draws in the committed fixture."""
    out: dict[str, list[str]] = {}
    group = None
    for line in DELIVERY.read_text(encoding="utf-8").splitlines():
        cells = [c.strip('"') for c in line.split('","')] if line else []
        if not cells:
            continue
        cells[0] = cells[0].lstrip('"')
        cells[-1] = cells[-1].rstrip('"')
        if cells[0] == "GROUP":
            group = cells[1]
        elif cells[0] == "HEADING" and group:
            out[group] = cells[1:]
    return out


def test_the_fixture_draws_every_key_heading_the_registry_defines() -> None:
    """The acceptance criterion that survives a dictionary edition bump: if a
    future edition adds a KEY heading to any of these groups, the fixture starts
    failing Rule 10a and the page's tables go stale. Fail here first, with the
    heading named, rather than there."""
    registry = pytest.importorskip("laterite.registry")
    drawn = _drawn_headings()
    for code in CHAIN:
        keys = [h.name for h in registry.GROUPS[code].headings if "KEY" in h.status]
        assert keys, f"{code} has no KEY headings — the registry shape changed"
        missing = [k for k in keys if k not in drawn.get(code, [])]
        assert not missing, (
            f"{code} in the seeded delivery is missing KEY heading(s) "
            f"{missing} — the registry defines {keys}. This is exactly the "
            "defect that made Rule 10c unrunnable."
        )


def test_key_headings_are_drawn_in_dictionary_order() -> None:
    """Rule 7 checks heading order. The mock had `LOCA_REM` and `LOCA_FDEP`
    the other way round, which is a finding the page would never explain."""
    registry = pytest.importorskip("laterite.registry")
    drawn = _drawn_headings()
    for code in CHAIN:
        order = [h.name for h in registry.GROUPS[code].headings]
        got = [h for h in drawn[code] if h in order]
        assert got == sorted(got, key=order.index), (
            f"{code} draws its headings out of dictionary order: {got}"
        )


def test_llpl_is_the_nine_column_table_the_layout_has_to_absorb() -> None:
    """Written out because it is the constraint behind the whole mobile
    pattern: seven KEY columns plus two results is why the demo needs a pinned
    key and a row carousel rather than a plain grid."""
    assert len(_drawn_headings()["LLPL"]) == 9


# --- the generated schema ------------------------------------------------


def test_the_generated_schema_is_what_the_dictionary_says() -> None:
    """`schema.ts` is committed AND regenerated by the build hook, so the only
    way it can be wrong is a hand-edit between builds. Regenerate into a temp
    location and compare rather than trusting the tree."""
    before = SCHEMA_TS.read_text(encoding="utf-8")
    proc = subprocess.run(
        ["node", str(SYNC)], capture_output=True, text=True, cwd=str(REPO / "web")
    )
    assert proc.returncode == 0, f"sync-demo-schema failed:\n{proc.stderr}"
    after = SCHEMA_TS.read_text(encoding="utf-8")
    if before != after:
        SCHEMA_TS.write_text(before, encoding="utf-8")
        pytest.fail(
            "web/landing/demo/schema.ts is stale — run "
            "`node scripts/sync-demo-schema.mjs` from web/"
        )


def test_the_schema_the_page_renders_matches_the_fixture_it_renders() -> None:
    """The two halves of the demo: the schema draws the columns, the fixture
    fills them. A mismatch means the page renders a header with no data under
    it, or drops a column the file carries."""
    drawn = _drawn_headings()
    ts = SCHEMA_TS.read_text(encoding="utf-8")
    for code in CHAIN:
        names = [
            line.split('name: "', 1)[1].split('"', 1)[0]
            for line in ts.splitlines()
            if 'name: "' in line
        ]
        for heading in drawn[code]:
            assert heading in names, (
                f"{code}.{heading} is in the seeded delivery but not in the "
                "generated schema the page renders from"
            )


def test_the_seeded_final_depth_is_the_rails_total() -> None:
    """#399's borehole rail runs to the seeded `LOCA_FDEP` rather than a number
    written into the rail. Pin the coupling here so changing the fixture's depth
    fails visibly instead of desynchronising the rail's scale."""
    rows = [
        line
        for line in DELIVERY.read_text(encoding="utf-8").splitlines()
        if line.startswith('"DATA","BH01"')
    ]
    assert rows, "BH01 has no DATA row in LOCA"
    assert rows[0].endswith('"25.00"'), (
        "BH01's LOCA_FDEP is no longer 25.00 — the rail's total is derived from "
        "it, so update the rail's expectation or the fixture, not just one"
    )


def test_the_delivery_has_no_tran_group() -> None:
    """Seeded defect 3, asserted directly: Rule 14's absence would otherwise be
    indistinguishable from the rule not running."""
    assert '"GROUP","TRAN"' not in DELIVERY.read_text(encoding="utf-8")


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-q"]))


def test_the_committed_fixture_is_utf8_with_crlf() -> None:
    """AGS4 is a CRLF format; a checkout that normalised the line endings would
    change every line number the findings cite."""
    raw = DELIVERY.read_bytes()
    raw.decode("utf-8")
    assert b"\r\n" in raw, "the seeded delivery lost its CRLF line endings"
    assert not raw.replace(b"\r\n", b"").count(b"\n"), "mixed line endings"


def test_the_fixture_is_small_enough_to_ship_on_a_landing_page() -> None:
    """It is inlined into the page bundle. Not a measured number in prose — the
    assertion IS the instrument, and it fails when the fixture outgrows its job."""
    assert DELIVERY.stat().st_size < 8 * 1024, (
        f"the seeded delivery has grown to {DELIVERY.stat().st_size} bytes; it "
        "ships inline on a landing page and should stay a demonstration, not a corpus"
    )


def test_json_report_round_trips(report) -> None:
    """The page consumes engine output as JSON; assert the shape it will read."""
    parsed = json.loads(report.to_json())
    assert isinstance(parsed, dict)


# --- the editing loop, against the engine rather than the rendered list ---


def _validate(text: str, tmp_path: Path):
    laterite = pytest.importorskip("laterite", reason="needs the built wheel")
    path = tmp_path / "edited.ags"
    path.write_bytes(text.encode("utf-8"))
    return laterite.validate(str(path))


def test_repairing_the_2dp_defect_removes_rule_8_from_the_engine(tmp_path) -> None:
    """#398's acceptance criterion, asserted on the ENGINE's output and not on
    the rendered list — a page that merely stopped drawing a finding while the
    validator still reported it would pass a UI assertion and be a lie.

    The edit is exactly the one the page's `setCell` makes: one cell in the LOCA
    row, `11.8` to `11.80`. Everything else has to survive it, which is the half
    that catches an edit path that rewrites more than the cell it was given.
    """
    src = DELIVERY.read_bytes().decode("utf-8")
    edited = src.replace('"BH01","CP","11.8"', '"BH01","CP","11.80"')
    assert edited != src, "the seeded 2DP defect is no longer where the edit expects it"

    got = {_identity(r) for r in _validate(edited, tmp_path).findings.to_dicts()}
    gone = SEEDED - got
    assert gone == {("AGS Format Rule 8", "LOCA", "LOCA_GL")}, (
        f"repairing the decimal places should clear Rule 8 and nothing else; "
        f"it cleared {sorted(map(str, gone))}"
    )
    assert not got - SEEDED, "the edit introduced a finding the page never mentions"


def test_the_orphan_survives_every_safe_repair(tmp_path) -> None:
    """The page's argument, pinned. Fix applies the mechanical repairs; the
    orphaned LLPL row is what a validator can only report and a human has to
    decide about, so it must still be there afterwards."""
    src = DELIVERY.read_bytes().decode("utf-8")
    edited = src.replace('"BH01","CP","11.8"', '"BH01","CP","11.80"').replace(
        '"S1","b"', '"S1","B"'
    )
    got = {_identity(r) for r in _validate(edited, tmp_path).findings.to_dicts()}
    assert ("AGS Format Rule 10c", "LLPL", None) in got, (
        "the orphaned LLPL row was repaired away — the page's whole "
        "validator-versus-fixer argument depends on it standing"
    )


def test_the_shipped_fixer_repairs_one_thing_and_says_so(tmp_path) -> None:
    """What the page's Fix button actually does, since it calls the engine's own
    fixer rather than a bespoke repair. #398's text describes three repairs; the
    engine mechanically applies one. This pins the real number so the page's copy
    cannot drift from it — and fails loudly if the fixer ever grows the other
    two, which would be the moment to reword the button's note."""
    laterite = pytest.importorskip("laterite", reason="needs the built wheel")
    result = laterite.fix(str(DELIVERY))

    assert len(result.applied) == 1, (
        f"the shipped fixer now applies {len(result.applied)} fixes, not 1 — the "
        "landing page's Fix note says what is left needs a human, and that "
        f"count moved: {[f['kind'] for f in result.applied]}"
    )
    assert result.applied[0]["kind"] == "reformat_numeric"
    assert result.applied[0]["rule"].endswith("Rule 8")

    # And what it leaves is the three the page says need a human.
    left = {_identity(r) for r in _validate(result.text, tmp_path).findings.to_dicts()}
    assert left == SEEDED - {("AGS Format Rule 8", "LOCA", "LOCA_GL")}, (
        f"the fixer's output no longer fails in the three ways the page's Fix "
        f"note describes: {sorted(map(str, left))}"
    )
