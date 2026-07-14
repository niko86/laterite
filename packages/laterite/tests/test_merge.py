"""`laterite.merge` — reconcile N AGS4 deliveries of one project into one file
(the Python face of the shared `laterite-ags4-merge` leaf). These assert the
real merged bytes + the warnings/revisions audit, not just "it runs"."""

import builtins

import laterite
import pytest

# Two deliveries of one project. LOCA is keyed on LOCA_ID.
#   file A: BH1 (NATE 100.00, GL 10.00), BH2 (NATE 200.00, GL 20.00)
#   file B: BH1 (NATE 100.00, GL 11.50 — a real GL revision, identical NATE),
#           BH3 (NATE 300.00, GL 30.00 — a new row)
# B also RE-TYPES LOCA_NATE 2DP -> X: the TYPE conflict strict mode rejects.
_A = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
        '"UNIT","","m","m"',
        '"TYPE","ID","2DP","2DP"',
        '"DATA","BH1","100.00","10.00"',
        '"DATA","BH2","200.00","20.00"',
        "",
    ]
)
_B = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
        '"UNIT","","m","m"',
        '"TYPE","ID","X","2DP"',
        '"DATA","BH1","100.00","11.50"',
        '"DATA","BH3","300.00","30.00"',
        "",
    ]
)


def test_merge_result_shape_and_bytes_reparse():
    """A widened merge returns a real MergeResult whose bytes re-parse as AGS4."""
    res = laterite.merge(_A, _B, on_type_clash="widen")
    assert isinstance(res, laterite.MergeResult)
    assert isinstance(res.bytes, builtins.bytes)
    assert res.bytes.startswith(b'"GROUP"')
    assert isinstance(res.warnings, list)
    assert isinstance(res.revisions, list)
    # The merged bytes are valid AGS4 (emit re-validates), so read() accepts them.
    doc = laterite.read(res.bytes)
    assert doc is not None
    assert res.text == res.bytes.decode("utf-8")


def test_merge_union_keeps_every_borehole():
    """Union, not intersection: BH2 (A-only) and BH3 (B-only) both survive."""
    text = laterite.merge(_A, _B, on_type_clash="widen").text
    for bh in ("BH1", "BH2", "BH3"):
        assert f'"{bh}"' in text, f"{bh} dropped from the union: {text!r}"


def test_merge_recency_last_argument_wins():
    """A later file wins a KEY conflict — BH1's GL becomes B's 11.50, not A's 10.00."""
    loca = _loca_rows(laterite.merge(_A, _B, on_type_clash="widen").text)
    bh1 = next(r for r in loca if r[0] == "BH1")
    assert bh1[2] == "11.50"


def test_merge_reports_the_real_revision_only():
    """revisions names BH1's GL change; the type-widened but unchanged NATE is NOT
    a revision (identical raw value across a 2DP->X widen)."""
    res = laterite.merge(_A, _B, on_type_clash="widen")
    revs = [r for r in res.revisions if r["group"] == "LOCA"]
    assert len(revs) == 1
    r = revs[0]
    assert r["key"] == ["BH1"]
    assert r["changed"] == ["LOCA_GL"]
    assert "LOCA_NATE" not in r["changed"]
    assert r["winner_file"] == 1  # the later argument


def test_merge_strict_type_conflict_raises():
    """Strict (default): LOCA_NATE typed 2DP vs X is an unreconcilable conflict."""
    with pytest.raises(laterite.MergeConflictError) as ei:
        laterite.merge(_A, _B)
    assert ei.value.exit_code == 6
    assert "LOCA_NATE" in str(ei.value)


def test_merge_synthesises_a_merge_tran(tmp_path):
    """tran_issue + tran_date stamp a synthesised merge-TRAN; save() writes bytes."""
    res = laterite.merge(
        _A,
        _B,
        on_type_clash="widen",
        tran_issue="9",
        tran_date="2024-05-01",
        tran_producer="Merger",
    )
    assert '"GROUP","TRAN"' in res.text
    assert '"9"' in res.text
    out = res.save(tmp_path / "merged.ags")
    assert out.read_bytes() == res.bytes


def test_merge_needs_two_sources():
    with pytest.raises(ValueError):
        laterite.merge(_A)


def _loca_rows(text: str) -> list[list[str]]:
    """The LOCA DATA rows of a merged AGS4 document as unquoted field lists."""
    rows, in_loca = [], False
    for line in text.splitlines():
        fields = [f.strip('"') for f in line.split(",")]
        if fields[0] == "GROUP":
            in_loca = len(fields) > 1 and fields[1] == "LOCA"
        elif in_loca and fields[0] == "DATA":
            rows.append(fields[1:])
    return rows


# --- UNIT reconciliation (#501) -------------------------------------------
# TYPE has a universal absorber (`X`); UNIT has none. So a unit clash is fatal in
# EVERY mode — the one place merge is deliberately less forgiving than it is about
# types. Before this, merge took the first non-empty UNIT, discarded the other,
# raised NO warning, and produced a file in which `10500.00` mm was labelled as
# metres — and since both values are valid `2DP` numbers, nothing downstream could
# ever catch it.

_UNIT_M = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_GL"',
        '"UNIT","","m"',
        '"TYPE","ID","2DP"',
        '"DATA","BH01","10.00"',
        "",
    ]
)
_UNIT_MM = _UNIT_M.replace('"UNIT","","m"', '"UNIT","","mm"').replace(
    '"DATA","BH01","10.00"', '"DATA","BH02","10500.00"'
)


@pytest.mark.parametrize("mode", ["error", "widen", "promote"])
def test_conflicting_units_are_fatal_in_every_mode(mode):
    """`widen`/`promote` settle a TYPE clash — but neither may absorb a UNIT
    clash. There is no supertype of metres and millimetres, and picking one
    silently mislabels the other file's values."""
    with pytest.raises(laterite.MergeConflictError) as exc:
        laterite.merge(_UNIT_M, _UNIT_MM, on_type_clash=mode)
    assert exc.value.exit_code == 6
    msg = str(exc.value)
    assert "LOCA_GL" in msg, "the offending heading must be named"
    assert "m" in msg and "mm" in msg, "both declared units must be reported"
    assert "will not convert units" in msg
    # The hint must not send the caller in a circle: no mode can fix this.
    assert "--on-type-clash" not in msg.lower()


def test_a_blank_unit_is_not_a_conflict():
    """Guards the fix against over-erroring: blank means *unspecified*, not a
    competing claim. A sparse delivery routinely leaves UNIT empty, so treating
    blank-vs-`m` as a conflict would break ordinary merges."""
    blank = _UNIT_M.replace('"UNIT","","m"', '"UNIT","",""')
    other = _UNIT_M.replace('"DATA","BH01","10.00"', '"DATA","BH02","12.00"')
    merged = laterite.merge(blank, other)  # strict, and must NOT raise
    assert '"UNIT","","m"' in merged.text, "the declared unit must survive"


# --- the type-clash lattice (#500) ----------------------------------------
# `widen` is byte-faithful but throws the TYPE away, and `X` is the LEAST
# informative resolution available. `promote` keeps the column numeric when every
# clashing code is nDP: max precision, coarser values zero-padded. Rule 8 demands a
# value match its declared TYPE exactly, so promote is the one mode that rewrites a
# cell — and it may only ever ADD trailing zeros, never round.

#: Same borehole list, but LOCA_GL is typed 2DP in one delivery and 5DP in the other.
_DP2 = _UNIT_M  # BH01, LOCA_GL "10.00", TYPE 2DP, UNIT m
_DP5 = (
    _UNIT_M.replace('"TYPE","ID","2DP"', '"TYPE","ID","5DP"')
    .replace('"DATA","BH01","10.00"', '"DATA","BH02","20.12345"')
)


def test_promote_keeps_the_greatest_precision_and_pads_the_coarser_values():
    res = laterite.merge(_DP2, _DP5, on_type_clash="promote")

    assert '"TYPE","ID","5DP"' in res.text, "the merged column keeps 5DP, not X"
    assert _loca_rows(res.text) == [
        ["BH01", "10.00000"],  # padded 2 -> 5 places; no digit changed
        ["BH02", "20.12345"],  # already 5DP; untouched
    ]

    warned = [w for w in res.warnings if w["kind"] == "type_promoted"]
    assert len(warned) == 1, f"promote must announce itself: {res.warnings}"
    assert warned[0]["heading"] == "LOCA_GL"
    assert "5DP" in warned[0]["message"]


def test_widen_still_throws_the_type_away():
    """The contrast that motivates promote: same inputs, `widen` gives up the TYPE."""
    res = laterite.merge(_DP2, _DP5, on_type_clash="widen")
    assert '"TYPE","ID","X"' in res.text
    assert _loca_rows(res.text) == [["BH01", "10.00"], ["BH02", "20.12345"]]


def test_error_is_the_default_and_names_both_escape_hatches():
    with pytest.raises(laterite.MergeConflictError) as exc:
        laterite.merge(_DP2, _DP5)  # no on_type_clash -> "error"
    msg = str(exc.value)
    assert "LOCA_GL" in msg
    assert "promote" in msg and "widen" in msg, f"offer BOTH ways out: {msg}"


def test_significant_figures_never_promote():
    """Zero-padding nDP is a formatting change; zero-padding nSF would assert
    measurement precision the instrument never resolved. So nSF falls back to X."""
    sf3 = _UNIT_M.replace('"TYPE","ID","2DP"', '"TYPE","ID","3SF"')
    sf5 = (
        _UNIT_M.replace('"TYPE","ID","2DP"', '"TYPE","ID","5SF"')
        .replace('"DATA","BH01","10.00"', '"DATA","BH02","20.123"')
    )
    res = laterite.merge(sf3, sf5, on_type_clash="promote")
    assert '"TYPE","ID","X"' in res.text, "nSF has no lossless join — widen"
    assert _loca_rows(res.text) == [["BH01", "10.00"], ["BH02", "20.123"]]
    assert not [w for w in res.warnings if w["kind"] == "type_promoted"]


def test_promote_never_demotes_whatever_the_argument_order():
    """`max` is the only lossless direction, so unlike a KEY conflict (where the
    later argument deliberately wins) the outcome cannot depend on order."""
    for label, (x, y) in {
        "coarse first": (_DP2, _DP5),
        "precise first": (_DP5, _DP2),
    }.items():
        res = laterite.merge(x, y, on_type_clash="promote")
        assert '"TYPE","ID","5DP"' in res.text, label
        values = {row[1] for row in _loca_rows(res.text)}
        assert "20.12345" in values, f"{label}: the precise value survives un-rounded"
        assert "10.00000" in values, f"{label}: the coarse value is padded"


def test_promote_lets_a_merged_row_still_content_hash_against_its_typed_source():
    """The property that motivates the mode. `_content_hash` canonicalises through
    the declared TYPE, so `10.00` hashes as a NUMBER under 2DP but as a STRING under
    X. A widened merge therefore stops value-matching its own inputs; a promoted one
    does not."""
    def bh01_hash(doc) -> str:
        loca = laterite.read(doc, content_hash=True, keys=True)["LOCA"]
        return next(
            h
            for i, h in zip(loca["LOCA_ID"], loca["_content_hash"], strict=True)
            if i == "BH01"
        )

    src_hash = bh01_hash(_DP2)

    promoted = laterite.merge(_DP2, _DP5, on_type_clash="promote")
    assert bh01_hash(promoted.bytes) == src_hash, (
        "10.00 (2DP) and 10.00000 (5DP) are the same number, so the promoted row "
        "still dedups against the typed delivery it came from"
    )

    widened = laterite.merge(_DP2, _DP5, on_type_clash="widen")
    assert bh01_hash(widened.bytes) != src_hash, (
        "the widen sharp edge, pinned: identical bytes, but X makes 10.00 a string "
        "where 2DP made it a number"
    )


def test_an_unknown_mode_is_rejected_by_name():
    with pytest.raises(laterite.Ags4Error) as exc:
        laterite.merge(_DP2, _DP5, on_type_clash="yolo")
    msg = str(exc.value)
    assert "yolo" in msg
    for mode in ("error", "widen", "promote"):
        assert mode in msg, f"the rejection must list {mode!r}: {msg}"


# --- the uvx launcher: the verb this CLI SHIPPED WITHOUT ---------------------
# `lat merge` landed in the native binary (#494) and never reached `laterite._cli`.
# No gate caught it: every cross-surface gate compared one hand-list against another
# hand-list, and both were equally wrong. The surface census (tools/gen_census.py)
# now diffs each launcher's OWN parser against clap's. These tests pin the verb's
# OUTPUT — the merged bytes and the `--json` wire summary — not just that it parses.


def test_cli_merge_writes_the_same_bytes_as_the_library(tmp_path, capsys):
    """`lat merge a b --out m` — the launcher is a thin face over `laterite.merge`."""
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_A)
    b.write_text(_B)
    out = tmp_path / "merged.ags"

    assert _cli.main(["merge", str(a), str(b), "--out", str(out), "--on-type-clash", "promote"]) == 0
    capsys.readouterr()

    expected = laterite.merge(str(a), str(b), on_type_clash="promote").bytes
    assert out.read_bytes() == expected, "the CLI must not reshape the merged document"


def test_cli_merge_json_uses_the_wire_spelling(tmp_path, capsys):
    """`--json` is a contract shared with the Rust binary and the npx launcher:
    `winner_file`, snake_case, whatever language rendered it."""
    import json

    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_A)
    b.write_text(_B)
    out = tmp_path / "merged.ags"

    code = _cli.main(
        ["merge", str(a), str(b), "--out", str(out), "--on-type-clash", "promote", "--json"]
    )
    assert code == 0
    summary = json.loads(capsys.readouterr().out)

    assert summary["bytes"] == len(out.read_bytes())
    assert summary["revisions"], "BH1's GL was revised 10.00 -> 11.50 — that is a revision"
    for r in summary["revisions"]:
        assert "winner_file" in r and "winnerFile" not in r
        assert isinstance(r["winner_file"], int)


def test_cli_merge_refuses_a_type_clash_by_default(tmp_path, capsys):
    """Exit 6 (schema violation) — the shared exit-code scheme, same as the binary."""
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_A)
    b.write_text(_B)

    assert _cli.main(["merge", str(a), str(b), "--out", str(tmp_path / "m.ags")]) == 6
    err = capsys.readouterr().err
    assert "LOCA_NATE" in err, "the refusal must name the heading that clashed"
    for mode in ("promote", "widen"):
        assert mode in err, f"the refusal must offer {mode!r} as the way forward"


def test_cli_merge_requires_out_and_two_files(tmp_path, capsys):
    """`--out` is required, so a merge can never silently overwrite an input."""
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_A)
    b.write_text(_B)

    assert _cli.main(["merge", str(a), str(b)]) == 5  # no --out
    assert _cli.main(["merge", str(a), "--out", str(tmp_path / "m.ags")]) == 5  # one file
    capsys.readouterr()


def test_cli_merge_rejects_a_typod_mode(tmp_path, capsys):
    """A typo must be bad-args (5), NOT a silent fall-through to `error` — which
    would refuse the merge and read like a genuine type clash."""
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_A)
    b.write_text(_B)

    code = _cli.main(
        ["merge", str(a), str(b), "--out", str(tmp_path / "m.ags"), "--on-type-clash", "promot"]
    )
    assert code == 5
    capsys.readouterr()
