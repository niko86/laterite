"""End-to-end tests for the ``lat`` Python CLI (``laterite._cli``) — the uvx
launcher (coverage campaign P2, see ``ags-wiki/concepts/coverage-campaign.md``).

The CLI's contract is its **exit code + output**, faithful to the Rust ``lat``
binary. So every test drives ``_cli.main([...])`` exactly as a shell would and
asserts the returned exit code and the captured output — never merely that a flag
parses (the surface census exists precisely because a flag can parse and be
dropped). Exit codes: 0 clean · 1 findings · 3 not-found/io · 4 not-utf8/parse ·
5 bad-args · 6 transport/merge-clash.
"""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from typing import Any

import pytest
from laterite import _cli

_FIX = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
)
_CLEAN = _FIX / "clean_minimal.ags"  # error-clean, certifiable
_DIRTY = _FIX / "rule5_unquoted.ags"  # carries a Rule 5 error


def _run(capsys: Any, *argv: str) -> tuple[int, str, str]:
    """Drive the CLI as a shell would; return (exit_code, stdout, stderr)."""
    code = _cli.main(list(argv))
    cap = capsys.readouterr()
    return code, cap.out, cap.err


# --- validate (the default verb) ------------------------------------------


def test_validate_clean_exits_0(capsys: Any) -> None:
    code, out, _ = _run(capsys, "validate", str(_CLEAN))
    assert code == 0
    assert "clean" in out.lower() or "0 finding" in out.lower()


def test_bare_file_is_validate_shorthand(capsys: Any) -> None:
    """`lat <file>` with no verb is `lat validate <file>`."""
    assert _run(capsys, str(_CLEAN))[0] == 0


def test_validate_dirty_exits_1(capsys: Any) -> None:
    code, out, _ = _run(capsys, "validate", str(_DIRTY))
    assert code == 1
    assert "finding" in out.lower() or "rule" in out.lower()


def test_validate_json_is_valid_json(capsys: Any) -> None:
    code, out, _ = _run(capsys, "validate", "--json", str(_DIRTY))
    assert code == 1
    payload = json.loads(out)  # must be a single valid JSON document
    assert isinstance(payload, (dict, list))


def test_validate_ndjson_lines_are_each_json(capsys: Any) -> None:
    code, out, _ = _run(capsys, "validate", "--ndjson", str(_DIRTY))
    assert code == 1
    lines = [ln for ln in out.splitlines() if ln.strip()]
    assert lines and all(json.loads(ln) for ln in lines)


def test_validate_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    code, _, err = _run(capsys, "validate", str(tmp_path / "nope.ags"))
    assert code == 3
    assert "error" in err.lower()


# --- main() dispatch + argument errors ------------------------------------


def test_json_and_ndjson_mutually_exclusive(capsys: Any) -> None:
    code, _, err = _run(capsys, "validate", "--json", "--ndjson", str(_CLEAN))
    assert code == 5
    assert "mutually exclusive" in err


def test_no_subcommand_exits_5(capsys: Any) -> None:
    code, _, err = _run(capsys)
    assert code == 5
    assert "required" in err.lower()


def test_unexpected_argument_exits_5(capsys: Any) -> None:
    code, _, _ = _run(capsys, "validate", str(_CLEAN), "--no-such-flag")
    assert code == 5


def test_readme_exits_0(capsys: Any) -> None:
    code, out, _ = _run(capsys, "--readme")
    assert code == 0
    assert out.strip()


def test_census_emits_verbs(capsys: Any) -> None:
    code, out, _ = _run(capsys, "census")
    assert code == 0
    payload = json.loads(out)
    # the census reflects the parser — validate/read/merge must be present.
    blob = json.dumps(payload)
    assert "validate" in blob and "merge" in blob


# --- rules ----------------------------------------------------------------


def test_rules_table(capsys: Any) -> None:
    code, out, _ = _run(capsys, "rules")
    assert code == 0
    assert "Rule" in out and "Severity" in out


def test_rules_json(capsys: Any) -> None:
    code, out, _ = _run(capsys, "rules", "--json")
    assert code == 0
    assert "rules" in json.loads(out)


# --- read -----------------------------------------------------------------


def test_read_lists_groups(capsys: Any) -> None:
    code, out, _ = _run(capsys, "read", str(_CLEAN))
    assert code == 0
    assert "PROJ" in out  # Rule 13 guarantees a PROJ group in a clean file


def test_read_groups_json(capsys: Any) -> None:
    code, out, _ = _run(capsys, "read", "--json", str(_CLEAN))
    assert code == 0
    assert "PROJ" in json.loads(out)


@pytest.mark.parametrize("flag", [None, "--json", "--csv"])
def test_read_single_group(capsys: Any, flag: str | None) -> None:
    argv = ["read", str(_CLEAN), "PROJ"] + ([flag] if flag else [])
    code, out, _ = _run(capsys, *argv)
    assert code == 0
    assert "PROJ_ID" in out


def test_read_unknown_group_exits_4(capsys: Any) -> None:
    code, _, err = _run(capsys, "read", str(_CLEAN), "ZZZZ")
    assert code == 4
    assert "not found" in err.lower()


def test_read_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    assert _run(capsys, "read", str(tmp_path / "no.ags"))[0] == 3


# --- diff -----------------------------------------------------------------


def test_diff_identical_files(capsys: Any) -> None:
    code, out, _ = _run(capsys, "diff", str(_CLEAN), str(_CLEAN))
    assert code == 0
    assert "total:" in out


def test_diff_json(capsys: Any) -> None:
    code, out, _ = _run(capsys, "diff", "--json", str(_CLEAN), str(_CLEAN))
    assert code == 0
    assert "groups" in json.loads(out)


def test_diff_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    assert _run(capsys, "diff", str(_CLEAN), str(tmp_path / "no.ags"))[0] == 3


# --- merge ----------------------------------------------------------------


def test_merge_two_files(capsys: Any, tmp_path: Any) -> None:
    out_ags = tmp_path / "merged.ags"
    code, _, _ = _run(capsys, "merge", str(_CLEAN), str(_CLEAN), "--out", str(out_ags))
    assert code == 0
    assert out_ags.exists() and out_ags.stat().st_size > 0


def test_merge_tran_description_and_remarks_reach_the_merged_file(
    capsys: Any, tmp_path: Any
) -> None:
    """``--tran-description`` / ``--tran-remarks`` on THIS launcher, not just the
    binary.

    The two are OTHER headings, so they sit outside ``_tran_from_args``'s
    all-five-or-none rule and travel a different arm of it. The census now pins
    that all three launchers DECLARE the flags; this pins that this one acts on
    them, by reading the bytes it wrote.
    """
    out_ags = tmp_path / "stamped.ags"
    code, _, _ = _run(
        capsys,
        "merge",
        str(_CLEAN),
        str(_CLEAN),
        "--out",
        str(out_ags),
        "--tran-issue",
        "9",
        "--tran-date",
        "2024-06-01",
        "--tran-producer",
        "Merger",
        "--tran-recipient",
        "Client",
        "--tran-status",
        "Merged",
        "--tran-description",
        "Combined ground investigation",
        "--tran-remarks",
        "Supersedes the first issue",
    )
    assert code == 0
    merged = out_ags.read_text(encoding="utf-8")
    assert "Combined ground investigation" in merged
    assert "Supersedes the first issue" in merged


def test_merge_revision_audit_renders_like_the_rust_binary(
    capsys: Any, tmp_path: Any
) -> None:
    """#373: the audit line's lists print Rust-``{:?}``-style — ``["X"]``, never
    Python's ``['X']``.

    This launcher's whole contract is byte-faithful output to the shipped
    binary, and this line is where it silently wasn't: no test ever merged two
    files that actually DIFFER, so the revision audit had never been rendered
    by anything, and an interpolated Python list repr sat there unobserved.
    The fixture pair changes one cell under the same KEY, which is exactly one
    revision."""
    base = tmp_path / "base.ags"
    rev = tmp_path / "rev.ags"
    base.write_bytes(_CLEAN.read_bytes())
    # The same file with one non-KEY cell changed, so the KEY matches and the
    # later file wins a content revision.
    revised = _CLEAN.read_bytes().replace(
        b"Clean minimal AGS4 fixture", b"Revised-title AGS4 fixture"
    )
    assert revised != _CLEAN.read_bytes(), "the edit missed — fixture changed?"
    rev.write_bytes(revised)
    out_ags = tmp_path / "merged.ags"

    code, out, _ = _run(capsys, "merge", str(base), str(rev), "--out", str(out_ags))
    assert code == 0
    assert "row revision(s):" in out, f"the merge produced no revision:\n{out}"

    audit = next(line for line in out.splitlines() if "changed" in line)
    assert '["' in audit and "['" not in audit, (
        "the audit must render lists the way the Rust binary's {:?} does "
        f"(#373): {audit!r}"
    )


def test_merge_needs_two_files_exits_5(capsys: Any, tmp_path: Any) -> None:
    code, _, err = _run(capsys, "merge", str(_CLEAN), "--out", str(tmp_path / "m.ags"))
    assert code == 5
    assert "two files" in err


def test_merge_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    code, _, _ = _run(
        capsys,
        "merge",
        str(_CLEAN),
        str(tmp_path / "no.ags"),
        "--out",
        str(tmp_path / "m.ags"),
    )
    assert code == 3


# --- certify --------------------------------------------------------------


def test_certify_clean_writes_cert(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "clean.ags"
    shutil.copyfile(_CLEAN, src)
    cert = tmp_path / "clean.ags.idx"
    code, out, _ = _run(capsys, "certify", str(src), "--out", str(cert))
    assert code == 0
    assert cert.exists()
    assert "certificate written" in out


def test_certify_dirty_refused_exits_1(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "dirty.ags"
    shutil.copyfile(_DIRTY, src)
    code, _, err = _run(capsys, "certify", str(src), "--out", str(tmp_path / "d.idx"))
    assert code == 1
    assert "certify" in err.lower()


def test_certify_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    assert _run(capsys, "certify", str(tmp_path / "no.ags"))[0] == 3


# --- fix ------------------------------------------------------------------


def test_fix_writes_sibling(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "dirty.ags"
    shutil.copyfile(_DIRTY, src)
    code, _, _ = _run(capsys, "fix", str(src))
    assert code in (0, 1)  # 0 clean after fix, 1 residual
    assert (tmp_path / "dirty.fixed.ags").exists()


def test_fix_json(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "dirty.ags"
    shutil.copyfile(_DIRTY, src)
    code, out, _ = _run(capsys, "fix", "--json", str(src))
    assert code in (0, 1)
    report = json.loads(out)
    assert set(report) >= {"file", "dest", "applied", "residual"}


def test_fix_in_place_and_fix_out_mutually_exclusive(
    capsys: Any, tmp_path: Any
) -> None:
    src = tmp_path / "dirty.ags"
    shutil.copyfile(_DIRTY, src)
    code, _, err = _run(
        capsys, "fix", str(src), "--in-place", "--fix-out", str(tmp_path / "o.ags")
    )
    assert code == 5
    assert "mutually exclusive" in err


# --- pack / unpack round-trip ---------------------------------------------


def test_pack_unpack_round_trip(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "in.ags"
    shutil.copyfile(_CLEAN, src)
    packed = tmp_path / "in.ags.zst"
    unpacked = tmp_path / "out.ags"
    assert _run(capsys, "pack", str(src), str(packed))[0] == 0
    assert packed.exists()
    assert _run(capsys, "unpack", str(packed), str(unpacked))[0] == 0
    assert unpacked.read_bytes() == src.read_bytes()


def test_pack_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    code, _, _ = _run(capsys, "pack", str(tmp_path / "no.ags"), str(tmp_path / "o.zst"))
    assert code == 3


# --- lock / unlock round-trip ---------------------------------------------


def test_lock_unlock_round_trip(capsys: Any, tmp_path: Any) -> None:
    src = tmp_path / "in.ags"
    shutil.copyfile(_CLEAN, src)
    pwfile = tmp_path / "pw.txt"
    pwfile.write_text("correct horse", encoding="utf-8")
    locked = tmp_path / "in.ags.age"
    unlocked = tmp_path / "out.ags"
    # --log-n 2 keeps scrypt cheap so the test stays fast.
    assert (
        _run(
            capsys,
            "lock",
            str(src),
            str(locked),
            "--password-file",
            str(pwfile),
            "--log-n",
            "2",
        )[0]
        == 0
    )
    assert locked.exists()
    assert (
        _run(
            capsys, "unlock", str(locked), str(unlocked), "--password-file", str(pwfile)
        )[0]
        == 0
    )
    assert unlocked.read_bytes() == src.read_bytes()


def test_lock_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    pwfile = tmp_path / "pw.txt"
    pwfile.write_text("x", encoding="utf-8")
    code, _, _ = _run(
        capsys,
        "lock",
        str(tmp_path / "no.ags"),
        str(tmp_path / "o.age"),
        "--password-file",
        str(pwfile),
    )
    assert code == 3


# --- excel round-trip + direction inference -------------------------------


def test_excel_export_then_import(capsys: Any, tmp_path: Any) -> None:
    xlsx = tmp_path / "out.xlsx"
    back = tmp_path / "back.ags"
    assert _run(capsys, "excel", str(_CLEAN), str(xlsx))[0] == 0  # → .xlsx = export
    assert xlsx.exists()
    assert _run(capsys, "excel", str(xlsx), str(back), "--import")[0] == 0
    assert back.exists()


def test_excel_undecidable_direction_exits_5(capsys: Any, tmp_path: Any) -> None:
    code, _, err = _run(capsys, "excel", str(_CLEAN), str(tmp_path / "out.dat"))
    assert code == 5
    assert "infer direction" in err


def test_excel_missing_file_exits_3(capsys: Any, tmp_path: Any) -> None:
    assert (
        _run(capsys, "excel", str(tmp_path / "no.ags"), str(tmp_path / "o.xlsx"))[0]
        == 3
    )
