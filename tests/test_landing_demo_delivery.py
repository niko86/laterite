"""The landing demo's seeded delivery must fail in exactly the ways the
page narrates, and in no others (#393; the set shrank when #527 seeded TRAN
and retired the permanent Rule 14).

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
import re
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
DELIVERY = REPO / "web" / "landing" / "demo" / "seeded-delivery.ags"
SCHEMA_TS = REPO / "web" / "landing" / "demo" / "schema.ts"

#: The chain the page draws, parent first.
CHAIN = ["PROJ", "LOCA", "SAMP", "LLPL"]

#: Every group whose drawn headings must agree with the registry — the chain
#: plus the File section's TRAN cover sheet (#527).
DRAWN = [*CHAIN, "TRAN"]

#: The three seeded defects, as (rule, group, what it must be about). Rule 16
#: appears TWICE on purpose: the bad `SAMP_TYPE` value is part of `LLPL`'s key
#: tuple, so one bad cell is two findings. The page shows both rather than
#: deduping, because that duplication is the repeated-KEY lesson firing live.
#: Rule 14 left the set when #527 seeded a clean TRAN cover sheet — a finding
#: no demo interaction could clear taught only that green was unreachable.
SEEDED = {
    ("AGS Format Rule 8", "LOCA", "LOCA_GL"),
    ("AGS Format Rule 16", "SAMP", "SAMP_TYPE"),
    ("AGS Format Rule 16", "LLPL", "SAMP_TYPE"),
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
        "the seeded delivery no longer fails in exactly the ways the page "
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
    for code in DRAWN:
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
    for code in DRAWN:
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


def _schema_ts() -> dict[str, dict]:
    """The committed `schema.ts`, read back as data.

    Parsed rather than regenerated. The first version of this shelled out to
    `node scripts/sync-demo-schema.mjs` and compared bytes, which fails in the
    `python` CI job — that job installs no Node toolchain and inherits the
    runner's, which is old enough to reject an ESM import outright. Requiring a
    second language for a check about a JSON file was the wrong shape anyway.

    What matters is not that the file is byte-identical to a fresh render; it is
    that what the page draws AGREES WITH THE DICTIONARY. That is the assertion
    below, and it catches both drifts the byte comparison did — a dictionary
    edition moving underneath the file, and a hand-edit of the file — without
    needing the generator to run.
    """
    text = SCHEMA_TS.read_text(encoding="utf-8")
    groups: dict[str, dict] = {}
    current: str | None = None
    for line in text.splitlines():
        code = re.search(r'^\s*code: "(\w+)",', line)
        if code:
            current = code.group(1)
            groups[current] = {"parent": None, "headings": []}
            continue
        if current is None:
            continue
        parent = re.search(r"^\s*parent: (null|\"(\w+)\"),", line)
        if parent:
            groups[current]["parent"] = parent.group(2)
            continue
        heading = re.search(
            r'^\s*\{ name: "(?P<name>\w+)", key: (?P<key>true|false), '
            r"required: (?P<required>true|false), "
            r'type: "(?P<type>[^"]*)", unit: "(?P<unit>[^"]*)"',
            line,
        )
        if heading:
            groups[current]["headings"].append(
                {
                    "name": heading.group("name"),
                    "key": heading.group("key") == "true",
                    "required": heading.group("required") == "true",
                    "type": heading.group("type"),
                    "unit": heading.group("unit"),
                }
            )
    return groups


def test_the_generated_schema_parses_at_all() -> None:
    """The parser above is load-bearing for the two checks below, so its own
    failure mode — a generator whose output shape changed — is caught here
    rather than showing up as "the dictionary agrees with nothing"."""
    parsed = _schema_ts()
    assert sorted(parsed) == sorted(DRAWN), (
        f"schema.ts no longer parses into the drawn demo groups: {sorted(parsed)}"
    )
    for code in DRAWN:
        assert parsed[code]["headings"], f"{code} parsed with no headings"


def test_the_committed_schema_agrees_with_the_dictionary() -> None:
    """Every field the page renders — the parent that draws the chain, and each
    heading's KEY flag, AGS TYPE and unit — against the registry that decides
    them. A dictionary edition bump that moved any of these would leave the page
    teaching a format the engine no longer implements."""
    registry = pytest.importorskip("laterite.registry")
    parsed = _schema_ts()

    for code in DRAWN:
        group = registry.GROUPS[code]
        assert parsed[code]["parent"] == group.parent, (
            f"{code}'s parent is {parsed[code]['parent']!r} in schema.ts and "
            f"{group.parent!r} in the dictionary — the page would draw the "
            "wrong chain"
        )

        by_name = {h.name: h for h in group.headings}
        for drawn in parsed[code]["headings"]:
            real = by_name.get(drawn["name"])
            assert real is not None, (
                f"{code}.{drawn['name']} is drawn by the page and is not in the "
                "dictionary at all"
            )
            assert drawn["key"] == ("KEY" in real.status), (
                f"{code}.{drawn['name']} is drawn "
                f"{'as KEY' if drawn['key'] else 'as non-KEY'} and the "
                f"dictionary says {real.status}"
            )
            # The other status axis (#616): REQUIRED drives its own header
            # mark now, so a drift here is a wrong mark on the page, not a
            # cosmetic field.
            assert drawn["required"] == ("REQUIRED" in real.status), (
                f"{code}.{drawn['name']} is drawn "
                f"{'as REQUIRED' if drawn['required'] else 'as non-REQUIRED'} "
                f"and the dictionary says {real.status}"
            )
            assert drawn["type"] == real.type, (
                f"{code}.{drawn['name']} is drawn as {drawn['type']} and the "
                f"dictionary says {real.type} — the field card would teach the "
                "wrong AGS TYPE"
            )
            assert drawn["unit"] == (real.unit or "")


def test_the_committed_schema_carries_every_key_heading() -> None:
    """The omission that made Rule 10c unrunnable, guarded on the RENDERED side
    as well as the fixture's: a page drawing four of LLPL's seven KEY columns
    would emit a file the validator cannot check the parent chain of."""
    registry = pytest.importorskip("laterite.registry")
    parsed = _schema_ts()

    for code in DRAWN:
        keys = [h.name for h in registry.GROUPS[code].headings if "KEY" in h.status]
        drawn = [h["name"] for h in parsed[code]["headings"]]
        missing = [k for k in keys if k not in drawn]
        assert not missing, (
            f"schema.ts draws {code} without KEY heading(s) {missing} — the "
            "page would render an incomplete key chain"
        )
        # And in dictionary order, which is what Rule 7 checks.
        order = [h.name for h in registry.GROUPS[code].headings]
        assert drawn == sorted(drawn, key=order.index), (
            f"schema.ts draws {code}'s headings out of dictionary order: {drawn}"
        )


def test_the_schema_the_page_renders_matches_the_fixture_it_renders() -> None:
    """The two halves of the demo: the schema draws the columns, the fixture
    fills them. A mismatch means the page renders a header with no data under
    it, or drops a column the file carries."""
    drawn = _drawn_headings()
    ts = SCHEMA_TS.read_text(encoding="utf-8")
    for code in DRAWN:
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


def test_the_delivery_seeds_one_clean_tran_row() -> None:
    """The inverse of the retired seeded defect 3 (#527): TRAN is present with
    exactly ONE data row — Rule 14's whole demand — so the finding set above
    proves the rule ran and passed, not that it never fired. A second row or a
    missing group would both surface as a Rule 14 the page no longer
    narrates."""
    text = DELIVERY.read_text(encoding="utf-8")
    assert '"GROUP","TRAN"' in text
    tran = text.split('"GROUP","TRAN"')[1].split('"GROUP"')[0]
    rows = [line for line in tran.splitlines() if line.startswith('"DATA"')]
    assert len(rows) == 1, f"TRAN must carry exactly one data row, got {len(rows)}"


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
