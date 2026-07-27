"""End-to-end tests for the ``lat`` Python CLI's deeper error / branch arms
(coverage campaign P2 residue — see ``ags-wiki/concepts/coverage-campaign.md``).

Same discipline as ``test_cli_verbs.py``: drive ``_cli.main([...])`` exactly as a
shell would and assert the exit code + output, never that a flag parses. These
cover the per-verb failure paths (not-found → 3, parse → 4, transport/merge →
6), the cert-assisted validate fallbacks, and the ``diff`` group-delta prints
that the happy-path verb tests did not reach.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

import laterite
from laterite import _cli

_FIX = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
)
_CLEAN = _FIX / "clean_minimal.ags"

# An extra, self-contained group to append so two files differ by exactly one
# group (drives diff's groups-added / groups-removed lines).
_EXTRA_GROUP = (
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID"\r\n'
    '"UNIT",""\r\n'
    '"TYPE","ID"\r\n'
    '"DATA","BH1"\r\n'
)


def _run(capsys: Any, *argv: str) -> tuple[int, str, str]:
    code = _cli.main(list(argv))
    cap = capsys.readouterr()
    return code, cap.out, cap.err


def _clean_copy(tmp_path: Path, name: str = "clean.ags") -> Path:
    p = tmp_path / name
    p.write_bytes(_CLEAN.read_bytes())
    return p


def _garbage(tmp_path: Path, name: str = "garbage.ags") -> Path:
    p = tmp_path / name
    p.write_text("not ags4 at all\r\n")
    return p


# --- validate --index fallbacks ---------------------------------------------


def test_validate_index_on_unparseable_input(tmp_path: Path, capsys: Any) -> None:
    # read() with a cert on a non-AGS4 file raises Ags4Error → the {ok:False}
    # shape flows through as if the engine had produced it (exit 4, parse).
    src = _clean_copy(tmp_path)
    cert = laterite.read(str(src)).certify()
    garbage = _garbage(tmp_path)
    code, _, err = _run(capsys, "validate", "--index", str(cert), str(garbage))
    assert code == 4
    assert "error" in err.lower()


def test_validate_index_with_bad_dict_errors(tmp_path: Path, capsys: Any) -> None:
    # The cert read succeeds, but the follow-on validate(dictionary=...) raises
    # (unreadable --dict) — the second Ags4Error arm of _with_cert.
    src = _clean_copy(tmp_path)
    cert = laterite.read(str(src)).certify()
    code, _, err = _run(
        capsys,
        "validate",
        "--index",
        str(cert),
        "--dict",
        str(tmp_path / "no.ags"),
        str(src),
    )
    assert code == 5
    assert "error" in err.lower()


# --- fix ---------------------------------------------------------------------


def test_fix_extensionless_file_writes_dot_fixed(tmp_path: Path, capsys: Any) -> None:
    # A source path with no suffix takes the `<name>.fixed` dest branch.
    noext = tmp_path / "delivery"
    noext.write_bytes(_CLEAN.read_bytes())
    code, out, _ = _run(capsys, "fix", str(noext))
    assert code == 0
    assert (tmp_path / "delivery.fixed").exists()
    assert "delivery.fixed" in out


def test_fix_out_to_missing_dir_errors(tmp_path: Path, capsys: Any) -> None:
    src = _clean_copy(tmp_path)
    code, _, err = _run(
        capsys, "fix", str(src), "--fix-out", str(tmp_path / "nodir" / "x.ags")
    )
    assert code == 3
    assert "writing" in err


# --- diff --------------------------------------------------------------------


def test_diff_unparseable_second_file(tmp_path: Path, capsys: Any) -> None:
    code, _, err = _run(
        capsys, "diff", str(_clean_copy(tmp_path)), str(_garbage(tmp_path))
    )
    assert code == 4
    assert "error" in err.lower()


def test_diff_reports_groups_added_and_removed(tmp_path: Path, capsys: Any) -> None:
    base = _clean_copy(tmp_path, "base.ags")
    plus = tmp_path / "plus.ags"
    plus.write_bytes(_CLEAN.read_bytes() + _EXTRA_GROUP.encode())
    added_code, added_out, _ = _run(capsys, "diff", str(base), str(plus))
    assert added_code == 0
    assert "groups added:" in added_out and "LOCA" in added_out
    removed_code, removed_out, _ = _run(capsys, "diff", str(plus), str(base))
    assert removed_code == 0
    assert "groups removed:" in removed_out and "LOCA" in removed_out


# --- merge -------------------------------------------------------------------


def test_merge_unparseable_input(tmp_path: Path, capsys: Any) -> None:
    code, _, err = _run(
        capsys,
        "merge",
        str(_clean_copy(tmp_path)),
        str(_garbage(tmp_path)),
        "--out",
        str(tmp_path / "m.ags"),
    )
    assert code == 4
    assert "error" in err.lower()


def test_merge_out_to_missing_dir_errors(tmp_path: Path, capsys: Any) -> None:
    src = _clean_copy(tmp_path)
    code, _, err = _run(
        capsys, "merge", str(src), str(src), "--out", str(tmp_path / "nodir" / "m.ags")
    )
    assert code == 3
    assert "writing" in err


# --- certify -----------------------------------------------------------------


def test_certify_unparseable_input(tmp_path: Path, capsys: Any) -> None:
    # read() raises Ags4Error whose message is not "cannot certify" → the
    # generic Ags4Error arm (exit = the error's own code).
    code, _, err = _run(capsys, "certify", str(_garbage(tmp_path)))
    assert code == 4
    assert "error" in err.lower()


def test_certify_out_to_directory_errors(tmp_path: Path, capsys: Any) -> None:
    # Writing the cert onto an existing directory raises IsADirectoryError —
    # neither FileNotFoundError nor Ags4Error, so the catch-all arm (exit 4).
    src = _clean_copy(tmp_path)
    adir = tmp_path / "adir"
    adir.mkdir()
    code, _, err = _run(capsys, "certify", str(src), "--out", str(adir))
    assert code == 4
    assert "error" in err.lower()


# --- read --------------------------------------------------------------------


def test_read_empty_file_notes_no_groups(tmp_path: Path, capsys: Any) -> None:
    empty = tmp_path / "empty.ags"
    empty.write_text("   \r\n")
    code, _, err = _run(capsys, "read", str(empty))
    assert code == 0
    assert "no groups" in err


def test_read_group_list_to_out_file(tmp_path: Path, capsys: Any) -> None:
    # `read --out <path>` writes the body to disk and notes it on stderr (_emit).
    src = _clean_copy(tmp_path)
    out = tmp_path / "groups.txt"
    code, _, err = _run(capsys, "read", str(src), "--out", str(out))
    assert code == 0
    assert out.exists() and "PROJ" in out.read_text()
    assert "written to" in err


# --- transport: pack / unpack / lock / unlock --------------------------------


def test_pack_failure_returns_6(tmp_path: Path, capsys: Any) -> None:
    # Packing a directory (not a file) fails inside transport.pack → exit 6.
    code, _, err = _run(capsys, "pack", str(tmp_path), str(tmp_path / "o.agsz"))
    assert code == 6
    assert "error" in err.lower()


def test_unpack_missing_input_returns_3(tmp_path: Path, capsys: Any) -> None:
    code, _, err = _run(
        capsys, "unpack", str(tmp_path / "nope.agsz"), str(tmp_path / "o.ags")
    )
    assert code == 3
    assert "not found" in err


def test_unpack_garbage_returns_6(tmp_path: Path, capsys: Any) -> None:
    code, _, err = _run(
        capsys, "unpack", str(_garbage(tmp_path, "g.agsz")), str(tmp_path / "o.ags")
    )
    assert code == 6
    assert "error" in err.lower()


def test_lock_via_getpass_prompt(tmp_path: Path, capsys: Any, monkeypatch: Any) -> None:
    # No --password-file and no env var → the passphrase comes from the (never
    # echoed) getpass prompt. Monkeypatch it so the lock runs unattended.
    monkeypatch.delenv("LAT_TRANSPORT_PASSWORD", raising=False)
    monkeypatch.setattr("getpass.getpass", lambda *a, **k: "hunter2")
    out = tmp_path / "o.agsl"
    code, _, err = _run(
        capsys, "lock", str(_clean_copy(tmp_path)), str(out), "--log-n", "2"
    )
    assert code == 0
    assert out.exists() and "locked" in err


def test_lock_failure_returns_6(tmp_path: Path, capsys: Any) -> None:
    pw = tmp_path / "pw"
    pw.write_text("hunter2")
    # Locking a directory fails inside transport.lock → exit 6.
    code, _, err = _run(
        capsys,
        "lock",
        str(tmp_path),
        str(tmp_path / "o.agsl"),
        "--password-file",
        str(pw),
    )
    assert code == 6
    assert "error" in err.lower()


def test_unlock_missing_input_returns_3(tmp_path: Path, capsys: Any) -> None:
    pw = tmp_path / "pw"
    pw.write_text("x")
    code, _, err = _run(
        capsys,
        "unlock",
        str(tmp_path / "nope.agsl"),
        str(tmp_path / "o.ags"),
        "--password-file",
        str(pw),
    )
    assert code == 3
    assert "not found" in err


# --- excel -------------------------------------------------------------------


def test_excel_export_failure_returns_6(tmp_path: Path, capsys: Any) -> None:
    # --export forces the export direction; an unparseable input fails in the
    # native writer → exit 6.
    code, _, err = _run(
        capsys, "excel", "--export", str(_garbage(tmp_path)), str(tmp_path / "o.xlsx")
    )
    assert code == 6
    assert "error" in err.lower()
