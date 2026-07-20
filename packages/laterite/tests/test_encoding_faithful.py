"""Batch B: encoding faithfulness — `read(encoding=)` is remembered, so chained
`validate`/`fix`/`diff` re-read the source with the SAME encoding instead of
silently assuming UTF-8; and the free `validate()` gains `encoding=`.

(#294 #3/#8 — a cp1252 / latin1 legacy delivery validated from Python used to
decode as UTF-8 (lossy → U+FFFD) and surface spurious Rule 1 findings, with no
way to say `encoding="cp1252"`.)
"""

from __future__ import annotations

import laterite as L

# A minimal AGS4 file whose only non-ASCII byte is 0xB0 — the degree sign in
# cp1252, but invalid UTF-8 (so UTF-8 decode replaces it with U+FFFD → Rule 1).
_CP1252 = (
    '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
    '"TYPE","ID","X"\r\n"DATA","P1","20\xb0 slope"\r\n'
).encode("cp1252")


def _has_rule1(report) -> bool:
    return "AGS Format Rule 1" in report.by_rule()


def _write(tmp_path, data=_CP1252, name="legacy.ags"):
    p = tmp_path / name
    p.write_bytes(data)
    return p


def test_chained_validate_reuses_the_read_encoding(tmp_path):
    """`read(p, encoding="cp1252").validate()` decodes the file as cp1252 (so the
    degree sign is a clean extended-ASCII char), while the default UTF-8 read sees
    a replacement char and flags Rule 1 — the handle stays faithful to its bytes."""
    p = _write(tmp_path)
    assert _has_rule1(L.read(str(p)).validate().report)  # UTF-8: U+FFFD → Rule 1
    assert not _has_rule1(L.read(str(p), encoding="cp1252").validate().report)


def test_free_validate_accepts_encoding(tmp_path):
    p = _write(tmp_path)
    assert _has_rule1(L.validate(str(p)))  # default UTF-8
    assert not _has_rule1(L.validate(str(p), encoding="cp1252"))


def test_chained_validate_encoding_arg_overrides_the_handle(tmp_path):
    """An explicit `encoding=` on `.validate()` wins over the handle's read encoding."""
    p = _write(tmp_path)
    # read as cp1252 (faithful) but force a UTF-8 re-check → the replacement-char Rule 1 returns
    assert _has_rule1(
        L.read(str(p), encoding="cp1252").validate(encoding="utf-8").report
    )


def test_chained_fix_inherits_read_encoding(tmp_path):
    """A cp1252 file with bare-LF endings (Rule 2a, fixable): `.fix()` inherits the
    read encoding, so it decodes the degree sign correctly, applies the CRLF fix,
    and re-emits UTF-8 (° → 0xC2 0xB0) with no residual Rule 1."""
    lf = (
        '"GROUP","PROJ"\n"HEADING","PROJ_ID","PROJ_NAME"\n"UNIT","",""\n'
        '"TYPE","ID","X"\n"DATA","P1","20\xb0"\n'
    ).encode("cp1252")
    p = _write(tmp_path, data=lf, name="lf.ags")
    fixed = L.read(str(p), encoding="cp1252").fix()  # returns the repaired Ags4File
    assert b"\r\n" in fixed.bytes  # Rule 2a fix applied
    assert b"\xc2\xb0" in fixed.bytes  # ° decoded from cp1252, re-emitted as UTF-8
    assert "AGS Format Rule 1" not in {f["rule"] for f in fixed.fix_report.findings}


def _severities(handle) -> tuple[int, int, int]:
    rep = handle.report
    if not rep.count:
        return (0, 0, 0)
    s = rep.findings["severity"].to_list()
    return (s.count("error"), s.count("warning"), s.count("fyi"))


def test_noop_fix_on_clean_cp1252_still_emits_utf8(tmp_path):
    """The *no-op* fix path (nothing safe to fix) must STILL honor "output is
    always UTF-8". `_CP1252` is clean but for an FYI-level degree sign, so the
    default `fix()` applies nothing — and used to pass the raw bytes straight
    through, leaking the invalid-UTF-8 0xB0. Now the no-op transcodes it."""
    p = _write(tmp_path)
    res = L.fix(str(p), encoding="cp1252")  # free fix → FixResult
    assert res.fixes_applied == 0  # genuinely a no-op
    res.bytes.decode("utf-8")  # must NOT raise — the contract is valid UTF-8
    assert b"\xc2\xb0" in res.bytes  # ° transcoded cp1252 → UTF-8, not raw 0xB0


def test_noop_fix_preserves_the_validation_verdict(tmp_path):
    """A no-op fix must not change the validation verdict. Before the transcode
    fix, `read(cp1252).fix()` handed back the raw bytes, which the fluent re-read
    then decoded as UTF-8 — flipping the clean FYI degree sign into a Rule 1
    error. So a no-op silently changed the outcome; assert it no longer does."""
    p = _write(tmp_path)
    before = L.read(str(p), encoding="cp1252").validate(fyi=True)
    after = L.read(str(p), encoding="cp1252").fix().validate(fyi=True)
    assert _severities(before) == _severities(after)
