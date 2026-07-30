"""Contract pins — one cheap executable assertion per prose promise the docstrings
make but nothing checked.

The hardening post-mortem found that several load-bearing contracts lived only as
English in a docstring, with no test adjudicating them — so a change could quietly
break the promise (exactly how the `fix()` no-op leaked non-UTF-8: the Rust and
Python docstrings both said "always UTF-8" with nothing executable holding the
line). Each test here pins one such contract, cheaply. They are not exhaustive
property tests; they are tripwires on specific promises.
"""

from __future__ import annotations

import json
from pathlib import Path

import laterite as L
import polars as pl
import pytest

# --- certify: the source it indexes must be UTF-8 (docstring: "which must be
#     UTF-8 (the byte index rejects other encodings)") ------------------------


def _valid_accented_ags() -> str:
    """A fully-valid AGS4 file (clean at errors-only) whose PROJ_NAME carries a
    non-ASCII, cp1252-encodable character — so the same content exists as valid
    UTF-8 and as non-UTF-8 cp1252 bytes."""
    proj = pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["Café résumé"]})
    # `synthesise_metadata=True` because a VALID file is this helper's
    # precondition, not its subject — without the catalogs the file trips
    # Rule 14/15/17 and the certify guard under test is never reached.
    # The TRAN stamp is part of that precondition: since 0.8.2 the engine mints
    # no TRAN unless the caller states the transmission, so an unstamped build
    # would trip Rule 14 and never reach the guard either.
    # All five, not just issue+date: TRAN_PROD/RECV/STAT are REQUIRED by the
    # dictionary, so a partial stamp is minted but reports Rule 10b (empty
    # REQUIRED field) — which is the honest outcome, and exactly what the old
    # "TBC" placeholder used to hide.
    return L.build_ags4(
        {"PROJ": proj},
        synthesise_metadata=True,
        tran=L.TranStamp(
            issue="1",
            date="2026-07-30",
            producer="Acme Ground Engineering",
            recipient="Client Ltd",
            status="FINAL",
        ),
    ).text


def test_certify_rejects_a_non_utf8_source(tmp_path: Path) -> None:
    """A file that validates error-clean but whose ORIGINAL bytes are non-UTF-8 (read as
    cp1252) cannot be certified — the byte index the certificate carries is UTF-8-only,
    and an offset into bytes it cannot address is not a fact it can write down. The
    non-ASCII cell is FYI-severity (Rule 1), so it does NOT block the validation; the
    UTF-8 guard is what fires.

    The refusal is an `Ags4Error` (it used to be a bare `ValueError` leaking out of the
    native index step, while a file WITH errors was refused as an `Ags4Error` — one door,
    two error families, depending on which way it failed).
    """
    raw = _valid_accented_ags().encode("cp1252")
    with pytest.raises(UnicodeError):
        raw.decode("utf-8")  # precondition: the source really is non-UTF-8
    src = tmp_path / "accented.ags"
    src.write_bytes(raw)
    f = L.read(src, encoding="cp1252").validate(warnings=False)
    assert not f.report.by_rule(), (
        "errors-only validation must be clean to reach the guard"
    )
    with pytest.raises(L.Ags4Error, match="not valid UTF-8"):
        f.certify(path=tmp_path / "accented.ags.idx")


def test_certify_accepts_the_same_content_as_utf8(tmp_path: Path) -> None:
    """The positive control: identical content as UTF-8 certifies fine — proving
    the rejection above is about the encoding, not the accented content."""
    src = tmp_path / "utf8.ags"
    src.write_bytes(_valid_accented_ags().encode("utf-8"))
    out = L.read(src).certify(path=tmp_path / "utf8.ags.idx")
    assert out.exists() and out.read_bytes()[:1] == b"{"


# --- strict build: raises, never returns findings ---------------------------


def test_strict_build_raises_and_never_returns_findings() -> None:
    """`mode="strict"` refuses invalid output with a RuntimeError — it never
    returns a BuildResult carrying findings (the whole point of strict is that a
    caller can trust the bytes without re-inspecting)."""
    invalid = {"LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]})}  # no PROJ/TRAN → errors
    with pytest.raises(RuntimeError, match="strict mode rejected"):
        L.build_ags4(invalid, mode="strict")


# --- synthesis mints the derivable catalogs, but never a PROJ ---------------


def test_synthesis_never_invents_proj() -> None:
    """Opted-in synthesis mints the derivable catalogs, but must NEVER invent a
    PROJ — PROJ_ID is project identity a machine cannot fabricate without
    corrupting provenance. The boundary is derivable-vs-authorial, which is also
    why DICT is never minted, and since 0.8.2 why TRAN needs a caller stamp."""
    res = L.build_ags4(
        {"LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]})},
        mode="autofix",
        synthesise_metadata=True,
        # All five: the stamp is required-complete now, so a build that WANTS a
        # TRAN must state a whole transmission. Two-of-five used to be accepted
        # and wrote three REQUIRED cells empty.
        tran=L.TranStamp(
            issue="1",
            date="2026-07-30",
            producer="Acme Ground Engineering",
            recipient="Client Ltd",
            status="FINAL",
        ),
    )
    groups = L.read(data=res.bytes).groups
    assert "PROJ" not in groups
    assert "DICT" not in groups
    assert {"TRAN", "UNIT", "TYPE"}.issubset(groups)  # the ones it DOES synthesise

    # ...and the same build WITHOUT a stamp mints no TRAN at all, rather than a
    # placeholder that would satisfy Rule 14 while asserting a transmission that
    # never happened. TRAN is authorial, exactly like PROJ and DICT.
    unstamped = L.build_ags4(
        {"LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]})},
        mode="autofix",
        synthesise_metadata=True,
    )
    assert "TRAN" not in L.read(data=unstamped.bytes).groups
    assert any(f["rule"] == "AGS Format Rule 14" for f in unstamped.findings), (
        f"the missing TRAN must be REPORTED, not silently absent: {unstamped.findings}"
    )


def test_autofix_does_not_synthesise_by_default() -> None:
    """The default is opt-OUT of magic: autofix repairs what you wrote and does
    not mint groups you didn't. The gaps come back as findings so you can see
    exactly what you declined."""
    res = L.build_ags4({"LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]})}, mode="autofix")
    groups = L.read(data=res.bytes).groups
    assert not {"TRAN", "UNIT", "TYPE"} & set(groups)
    rules = {f["rule"] for f in res.findings}
    assert any("Rule 14" in r for r in rules)
    assert any("Rule 15" in r for r in rules)
    assert any("Rule 17" in r for r in rules)


# --- Ags4File.close() is idempotent -----------------------------------------


def test_close_is_idempotent() -> None:
    """Closing a handle twice is a no-op the second time — a `with`/finally that
    also calls `close()` explicitly must not raise on the redundant call."""
    src = '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
    f = L.read(text=src)
    f.close()
    f.close()  # must not raise


# --- the `xn` knob is typing-only: it never changes the re-emitted bytes -----


def test_bytes_are_invariant_under_xn_mode() -> None:
    """`xn` selects how X-typed columns are TYPED into frames (string vs numeric);
    it must not perturb the spec-correct re-emit — `.bytes` is byte-identical
    across modes, or a formatting choice would silently ride on a read option."""
    src = (
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_GL"\r\n"UNIT","","m"\r\n'
        '"TYPE","ID","2DP"\r\n"DATA","BH1","1.50"\r\n'
    )
    assert L.read(text=src, xn="string").bytes == L.read(text=src, xn="numeric").bytes


# --- the two Report serialisations describe the same findings ---------------


def test_to_json_and_to_ndjson_describe_the_same_findings() -> None:
    """`to_json` nests occurrences BY RULE (`{rule: [occ, ...]}`); `to_ndjson`
    emits ONE line per occurrence. Different shapes, same underlying facts: the
    flattened `(rule, line, group, desc)` occurrences must agree as multisets, so
    neither view can silently drop or duplicate a finding the other keeps."""
    fx = str(Path(__file__).resolve().parent / "fixtures" / "multi_finding.ags")
    rep = L.validate(fx, warnings=True, fyi=True)

    by_rule = json.loads(rep.to_json())["findings"]
    assert isinstance(by_rule, dict)
    flat = sorted(
        (rule, o["line"], o["group"], o["desc"])
        for rule, occs in by_rule.items()
        for o in occs
    )
    nd = sorted(
        (r["rule"], r["line"], r["group"], r["desc"])
        for r in (
            json.loads(line) for line in rep.to_ndjson().splitlines() if line.strip()
        )
    )
    assert flat == nd
    # And the by-rule keys are exactly what by_rule() reports.
    assert set(by_rule) == set(rep.by_rule())
