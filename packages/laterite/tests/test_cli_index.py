"""`lat validate --index <cert>` on the uvx launcher — the door this CLI never had.

The surface census's per-verb FLAG table is what surfaced this. Every earlier gate
compared VERB names, and `validate` is present on all three launchers, so they all
agreed — while the binary and npx took a `.ags.idx` certificate on that verb and uvx
simply had no `--index` at all. A script portable across the three launchers was not.

These tests assert the OUTPUT, never that the flag parses: a certificate that is
honoured says `report.certified`, and one that is not does not. The distinction is the
whole feature — the first implementation here PARSED `--index` perfectly and skipped
nothing, and only asserting on the outcome caught it.

`certified` used to be a *value of* `resolution` ("certified" instead of "exact"), which
conflated two facts: WHICH dictionary judged the file, and WHETHER the rule engine ran.
A certified read now reports both — the edition the cert recorded, and that the engine
was skipped — because a caller reading `resolution` to find out which edition applied
should not have that answer replaced by an unrelated one.
"""

from __future__ import annotations

import json
from pathlib import Path

import laterite
from laterite import _cli

#: The shared hand-authored ERROR-clean fixture (CRLF, pinned by `.gitattributes`) —
#: the same one the Rust cert tests mint over. Referenced, not copied: a second copy
#: is a second thing to drift, which is the bug this whole census exists to catch.
_CLEAN = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
    / "clean_minimal.ags"
).read_bytes()


def _certified(path, cert) -> bool:
    """Did the certificate stand in for the rule engine? — via the library, which is the
    same code path `_cli._with_cert` drives."""
    return (
        laterite.read(str(path), index=str(cert))
        .validate(warnings=False)
        .report.certified
    )


def _mint(tmp_path):
    src = tmp_path / "clean.ags"
    src.write_bytes(_CLEAN)
    cert = laterite.read(str(src)).certify()
    return src, cert


def test_cli_validate_accepts_index(tmp_path, capsys):
    """The flag exists at all — `--index` used to be an unknown-argument error here."""
    src, cert = _mint(tmp_path)
    assert _cli.main(["validate", str(src), "--index", str(cert), "--no-warnings"]) == 0
    capsys.readouterr()


def test_cli_index_skips_the_rule_engine(tmp_path, capsys):
    """A fresh cert must STAND IN for the rules pass, not merely be read and dropped.

    `report.certified` is the only proof of that; the finding count is 0 either way, so
    a `--index` that quietly did nothing would still exit 0 and print the same clean
    verdict.
    """
    src, cert = _mint(tmp_path)
    assert _certified(src, cert) is True

    assert _cli.main(["validate", str(src), "--index", str(cert), "--no-warnings"]) == 0
    err = capsys.readouterr().err
    assert "rule engine skipped" in err, f"the skip must be announced, got: {err!r}"


def test_cli_index_auto_sentinel_does_not_disarm_the_skip(tmp_path, capsys):
    """`--dict-version` defaults to the STRING `"auto"`, the CLI's sentinel for "no pin".

    The library has no such value: hand it `"auto"` and the request looks like a FORCED
    edition, so a certificate minted without one stops covering it and the skip silently
    turns off. That is exactly what this flag did the first time it ran — it parsed, it
    exited 0, it printed a clean verdict, and it never once skipped the engine. Nothing
    but the resolution would have told you.
    """
    src, cert = _mint(tmp_path)

    assert (
        _cli.main(
            [
                "validate",
                str(src),
                "--index",
                str(cert),
                "--no-warnings",
                "--dict-version",
                "auto",
            ]
        )
        == 0
    )
    err = capsys.readouterr().err
    assert "rule engine skipped" in err, "the `auto` sentinel must not disarm the cert"


def test_cli_index_a_forced_edition_correctly_refuses_the_cert(tmp_path, capsys):
    """The other half: a cert minted WITHOUT a forced edition must not cover a request
    that forces one. Proving the skip is a real profile check, not an unconditional yes."""
    src, cert = _mint(tmp_path)

    assert (
        _cli.main(
            [
                "validate",
                str(src),
                "--index",
                str(cert),
                "--no-warnings",
                "--dict-version",
                "4.2",
            ]
        )
        == 0
    )
    err = capsys.readouterr().err
    assert "rule engine skipped" not in err, (
        "a forced edition is more than the cert vouches for"
    )


def test_cli_index_stale_cert_is_a_note_not_an_error(tmp_path, capsys):
    """A cert that went stale must NOT stop the tool: the binary notes it and runs the
    full check. Refusing to validate because a sidecar aged out would be the worse
    failure — the file itself is still perfectly checkable."""
    src, cert = _mint(tmp_path)
    # Append a broken group, so the cert no longer matches AND the file now has findings.
    src.write_bytes(_CLEAN + b'\r\n"GROUP","EXTRA"\r\n')

    code = _cli.main(["validate", str(src), "--index", str(cert)])
    out = capsys.readouterr()

    assert code == 1, "the full check must run and report the new findings"
    assert "--index not used" in out.err, (
        f"the stale cert must be explained: {out.err!r}"
    )
    assert "finding(s)" in out.out


def test_cli_index_wrong_file_cert_is_caught(tmp_path, capsys):
    """A certificate for a DIFFERENT file must be rejected on its bytes. This is the
    case npx got wrong in the most complete way possible: it accepted `--index`, read
    nothing, and validated as if the flag were absent."""
    _src, cert = _mint(tmp_path)
    other = tmp_path / "other.ags"
    # Still error-clean, but not the same bytes — so the cert's SHA-256 cannot match.
    other.write_bytes(_CLEAN + b"\r\n")

    assert (
        _cli.main(["validate", str(other), "--index", str(cert), "--no-warnings"]) == 0
    )
    err = capsys.readouterr().err
    assert "--index not used" in err, (
        "a cert minted for another file must not be trusted"
    )


def test_cli_index_json_output_is_unchanged_by_the_cert(tmp_path, capsys):
    """`--json` is a wire contract shared with the binary and npx. The certified path
    synthesises its verdict rather than running the engine, so it must still produce
    the same JSON shape a clean engine run does."""
    src, cert = _mint(tmp_path)

    assert (
        _cli.main(
            ["validate", str(src), "--index", str(cert), "--no-warnings", "--json"]
        )
        == 0
    )
    certified = json.loads(capsys.readouterr().out)

    assert _cli.main(["validate", str(src), "--no-warnings", "--json"]) == 0
    engine = json.loads(capsys.readouterr().out)

    assert certified == engine, (
        "a certificate must not change what the verdict LOOKS like"
    )
