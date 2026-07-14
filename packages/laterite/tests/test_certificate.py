"""The `.ags.idx` certificate: minting one, and consuming one.

`Ags4File.certify()` VALIDATES the file and mints a certificate (an error-clean
validation plus a byte-offset index); `read(..., index=...)` loads + freshness-checks one
and hands it to the engine, which skips the rule pass only if the certificate can answer
the question being asked — `report.certified` is how it says so.

The rules this pins (owner-settled, and rewritten in the trust-model rework):
- **certify runs the validation itself** — it used to demand a prior `.validate()` and
  then vouch for whatever that had found, which made the certificate's contents an
  assertion by the caller. The caller got them wrong: the mint's `warnings`/`fyi`
  parameters were optional, defaulted to zero, and nothing ever passed them.
- **it refuses ERRORS, records warnings** — a warning is not a reason to refuse a
  certificate; it is a reason that certificate cannot answer a warnings request.
- **a certificate never speaks for the world** — Rule 20's on-disk `FILE/` check is not a
  function of the certified bytes, so it re-runs on every read, certified or not. There is
  no field in the stamp with which to claim otherwise.
- **a stale cert fails fast at `read()`** — an explicit `index=` asserts the cert is
  for this file, so a size/SHA mismatch raises `StaleCertError`, never a silent
  fall-back to re-validation.
"""

from __future__ import annotations

import laterite as lat
import pytest
from laterite import Ags4Error, StaleCertError

# A hand-authored clean AGS4 file (PROJ + TRAN + UNIT + TYPE), CRLF as the spec
# mandates, TRAN_AGS=4.2. Inlined so the test is self-contained; it validates with
# zero findings, the precondition `certify()` requires.
CLEAN = "\r\n".join(
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


RULE_20 = "AGS Format Rule 20"

# CLEAN plus a FILE group declaring one attachment. Rule 20's CONTENT half is satisfied
# (FS1 *is* defined in FILE), so the only open question about Rule 20 is the WORLD one:
# is `FILE/FS1/photo.jpg` actually beside the .ags? That question's answer can change
# without the file changing — which is exactly why no certificate may carry it.
WITH_ATTACHMENT = CLEAN + "\r\n".join(
    [
        '"GROUP","FILE"',
        '"HEADING","FILE_FSET","FILE_NAME"',
        '"UNIT","",""',
        '"TYPE","X","X"',
        '"DATA","FS1","photo.jpg"',
        "",
    ]
)


def _write(tmp_path, name="delivery.ags", text=CLEAN):
    p = tmp_path / name
    p.write_bytes(text.encode("utf-8"))  # write_bytes: keep CRLF (no translation)
    return p


# --- mint ------------------------------------------------------------------


def test_certify_mints_sidecar_beside_the_file(tmp_path):
    src = _write(tmp_path)
    out = lat.read(src).validate().certify()
    assert out == tmp_path / "delivery.ags.idx"  # default: source path + ".idx"
    assert out.exists() and out.read_bytes()


def test_certify_accepts_an_explicit_path(tmp_path):
    src = _write(tmp_path)
    dst = tmp_path / "elsewhere.idx"
    assert lat.read(src).validate().certify(path=dst) == dst
    assert dst.exists()


def test_certify_validates_the_file_itself(tmp_path):
    # CONTRACT CHANGE. `certify` used to REQUIRE a prior `.validate()` and then vouch for
    # whatever that had found — which made the certificate's contents an assertion by the
    # caller. The caller got them wrong: the mint took `warnings=0, fyi=0` as DEFAULT
    # ARGUMENTS and nothing ever passed them, so every cert this library minted claimed to
    # have measured zero warnings without having looked.
    #
    # The mint now runs the rules. There is no parameter left through which a caller could
    # assert a verdict, so `read(src).certify()` needs no prior validate — and is honest.
    src = _write(tmp_path)
    out = lat.read(src).certify()
    assert out.exists()
    sc = _cert(out)
    assert sc.errors == 0
    assert sc.warnings is not None, "the mint MEASURED warnings, it did not assume them"
    assert sc.fyi is not None


def test_certify_refuses_a_dirty_file(tmp_path):
    # drop TRAN so the file no longer validates clean
    dirty = CLEAN.split('"GROUP","TRAN"')[0]  # PROJ only
    src = _write(tmp_path, text=dirty)
    h = lat.read(src).validate()
    assert not h.report.is_valid
    with pytest.raises(Ags4Error, match="cannot certify"):
        h.certify()


def test_certify_from_text_needs_an_explicit_path(tmp_path):
    h = lat.read(text=CLEAN).validate()
    with pytest.raises(Ags4Error, match="no source path"):
        h.certify()  # nowhere to derive <source>.idx from
    # ...but an explicit path works
    dst = tmp_path / "fromtext.idx"
    assert h.certify(path=dst) == dst


# --- mint in-memory (certify_bytes, #390) ----------------------------------


def test_certify_bytes_returns_cert_bytes_in_memory(tmp_path):
    src = _write(tmp_path)
    blob = lat.read(src).validate().certify_bytes()
    assert isinstance(blob, bytes)
    assert blob.lstrip().startswith(b"{")  # the .ags.idx JSON, no file written
    assert not (tmp_path / "delivery.ags.idx").exists()  # nothing on disk


def test_certify_bytes_are_a_usable_cert(tmp_path):
    # The in-memory bytes are exactly a certificate: write them out and a later
    # read(index=) consumes them to skip the engine (the whole point — no temp
    # file needed at MINT time, so a web backend can hand them to an upload).
    src = _write(tmp_path)
    blob = lat.read(src).validate().certify_bytes()
    idx = tmp_path / "inmem.ags.idx"
    idx.write_bytes(blob)
    rep = lat.read(src, index=idx).validate(warnings=False).report
    assert rep.certified and rep.is_valid


def test_certify_bytes_matches_the_file_certify(tmp_path):
    # certify() and certify_bytes() mint the SAME certificate for the same handle
    # (bar the mint timestamp) — the .ags.idx is JSON, so compare the freshness
    # fingerprint (file section) + the byte-offset index (groups).
    import json

    src = _write(tmp_path)
    on_disk = json.loads(lat.read(src).validate().certify().read_bytes())
    in_mem = json.loads(lat.read(src).validate().certify_bytes())
    assert in_mem["file"] == on_disk["file"]  # size / sha256 / edition
    assert in_mem["groups"] == on_disk["groups"]  # same byte-offset index


def test_certify_bytes_needs_no_prior_validate_but_still_refuses_a_dirty_file(tmp_path):
    src = _write(tmp_path)
    assert lat.read(src).certify_bytes()  # runs its own validation
    dirty = CLEAN.split('"GROUP","TRAN"')[0]  # PROJ only → error findings
    h = lat.read(_write(tmp_path, name="dirty.ags", text=dirty))
    with pytest.raises(Ags4Error, match="cannot certify"):
        h.certify_bytes()


# --- consume (validate-skip) -----------------------------------------------


def test_fresh_cert_makes_validate_skip_the_engine(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).certify()

    rep = lat.read(src, index=cert).validate(warnings=False).report
    assert rep.certified, "a fresh, matching cert answers an errors-only question"
    assert rep.count == 0 and rep.is_valid
    assert rep.dict_version == "4.2"  # carried from the cert (TRAN_AGS=4.2)
    # `resolution` says WHICH dictionary judged the file, not whether we skipped —
    # one field, one fact. It used to carry a "certified" sentinel and answer neither
    # question properly.
    assert rep.resolution == "exact"


def test_read_without_index_still_runs_the_engine(tmp_path):
    src = _write(tmp_path)
    lat.read(src).certify()
    rep = lat.read(src).validate().report  # no index= → ordinary validation
    assert not rep.certified
    assert rep.is_valid


def test_a_cert_that_measured_warnings_and_found_none_can_answer_for_them(tmp_path):
    # The mint runs BOTH tiers, so a clean file's cert knows there are no warnings and
    # can say so. Under the old model the mint recorded `warnings: 0` WITHOUT looking,
    # and the consumer — knowing it couldn't trust that — refused to use the cert for
    # any warnings request at all. Measure honestly and you can answer more, not less.
    src = _write(tmp_path)
    cert = lat.read(src).certify()
    rep = lat.read(src, index=cert).validate(warnings=True, fyi=True).report
    assert rep.certified
    assert rep.count == 0


def test_certified_read_round_trips_from_text(tmp_path):
    cert = lat.read(text=CLEAN).certify(path=tmp_path / "t.idx")
    rep = lat.read(text=CLEAN, index=cert).validate(warnings=False).report
    assert rep.certified and rep.count == 0


# --- staleness (fail fast at read) -----------------------------------------


def test_stale_cert_raises_at_read(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).validate().certify()
    src.write_bytes(src.read_bytes() + b"\r\n")  # mutate the source under the cert
    with pytest.raises(StaleCertError, match="does not match"):
        lat.read(src, index=cert)


def test_stale_cert_for_text_input_raises(tmp_path):
    cert = lat.read(text=CLEAN).validate().certify(path=tmp_path / "t.idx")
    with pytest.raises(StaleCertError):
        lat.read(text=CLEAN + "\r\n", index=cert)


def test_malformed_cert_raises_value_error(tmp_path):
    src = _write(tmp_path)
    bad = tmp_path / "bad.idx"
    bad.write_bytes(b"{not json")
    with pytest.raises(ValueError):
        lat.read(src, index=bad)


# --- the certificate's own surface (provenance + index) --------------------


def test_sidecar_records_provenance_and_index(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).certify()
    sc = lat._laterite_native.Sidecar.from_json(cert.read_bytes())
    assert sc.validator == "laterite_ags4"
    # The ENGINE fingerprint — a hash of the rule sources and the dictionary, not the
    # wheel's version. A rule can change without a version bump; this cannot.
    assert sc.engine and len(sc.engine) == 16
    assert sc.compat is None  # native validation, not the compat profile
    assert sc.edition == "4.2" and sc.edition_forced is False
    # Measured, all three — `None` would mean "never looked", and the old format could
    # not tell that apart from "looked and found none".
    assert (sc.errors, sc.warnings, sc.fyi) == (0, 0, 0)
    # The byte index locates every group, in file order, tiling [0, size). Each
    # entry is a LIST of spans, not one span: a code can occupy more than one
    # section, and the single-span shape this replaces could only express the first
    # — so slicing a redeclared group silently returned a subset of its rows.
    idx = sc.index()
    assert sc.order == ["PROJ", "TRAN", "UNIT", "TYPE"] == list(idx)
    assert all(len(spans) == 1 for spans in idx.values()), "no group here is redeclared"
    assert idx["PROJ"][0][0] == 0
    assert idx["TYPE"][0][1] == sc.size


def test_cert_from_a_different_engine_is_not_trusted(tmp_path):
    # The skip must be checker-aware, not just byte-fresh: a cert minted by a
    # different/older validator engine is re-validated, never trusted (its clean
    # verdict may not reproduce under today's rules).
    import json

    src = _write(tmp_path)
    cert = lat.read(src).certify()
    data = json.loads(cert.read_bytes())
    data["validation"]["engine"] = "0000deadbeef0000"  # a rule changed under it
    cert.write_bytes(json.dumps(data).encode())

    # bytes still match (so read() does NOT raise StaleCertError) ...
    h = lat.read(src, index=cert)
    # ... but the engine differs, so validate() runs the rules, not the cert.
    rep = h.validate(warnings=False).report
    assert not rep.certified  # engine actually ran
    assert rep.is_valid  # and the file is genuinely clean


def _cert(path):
    return lat._laterite_native.Sidecar.from_json(path.read_bytes())


def test_a_certificate_cannot_claim_a_world_check(tmp_path):
    # THE BUG THIS ARC IS FOR, and the test that used to assert it was CORRECT.
    #
    # There was a `check_files` field on the certificate. Certify with `--check-files`,
    # and a later `validate(check_files=True, index=cert)` read that field, concluded the
    # cert "covered" the request, and skipped. Delete the FILE/ tree in between and the
    # .ags bytes have not moved — same size, same SHA — so the cert is still perfectly
    # valid, and the file reports CLEAN when the truth is one Rule 20 finding.
    #
    # The old test here (`test_check_files_request_needs_a_check_files_cert`) asserted
    # exactly that skip, as the desired behaviour.
    #
    # The field is gone. There is nowhere in the format to record a claim about a
    # directory, because a statement about a file's bytes cannot be one.
    src = _write(tmp_path)
    sc = _cert(lat.read(src).certify())

    raw = (tmp_path / "delivery.ags.idx").read_bytes().decode()
    assert "check_files" not in raw, raw
    assert not hasattr(sc, "check_files")
    assert sc.etag is None and sc.last_modified is None  # Python mints locally


def test_the_world_check_runs_even_on_a_certified_read(tmp_path):
    # The other half: `check_files=True` with a valid cert must STILL stat the FILE/ tree.
    # The cert legitimately answers the content half; the world half is not its business.
    src = _write(tmp_path, text=WITH_ATTACHMENT)
    cert = lat.read(src).certify()

    # No FILE/ tree beside it → Rule 20's on-disk half fires, cert or no cert.
    rep = lat.read(src, index=cert).validate(check_files=True, warnings=False).report
    assert rep.certified, "the CONTENT is certified — that part is true"
    assert RULE_20 in set(rep.by_rule()), (
        f"but the world is checked live, every time: {set(rep.by_rule())}"
    )
    assert not rep.is_valid

    # Materialise the attachment → the same cert, the same bytes, and now it IS clean.
    leaf = tmp_path / "FILE" / "FS1"
    leaf.mkdir(parents=True)
    (leaf / "photo.jpg").write_bytes(b"x")
    rep = lat.read(src, index=cert).validate(check_files=True, warnings=False).report
    assert rep.certified and rep.is_valid
    # Two different verdicts, one certificate, and not a byte of the .ags changed. That
    # is the whole reason a certificate may not vouch for this.


def test_forced_edition_cert_does_not_satisfy_an_auto_request(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).certify(dict_version="4.2")
    assert _cert(cert).edition_forced is True

    # An AUTO request must not be answered by a FORCED cert. Forcing means "ignore
    # TRAN_AGS", so on a file whose declared edition disagrees with its content the two
    # runs apply different dictionaries — even when the edition STRING is the same.
    assert not lat.read(src, index=cert).validate(warnings=False).report.certified
    # ...but the same forcing IS the same question.
    assert (
        lat.read(src, index=cert)
        .validate(dict_version="4.2", warnings=False)
        .report.certified
    )


# --- certify must never destroy a data file (the certify(path=<.ags>) footgun) ---


def test_certify_refuses_to_overwrite_the_source_file(tmp_path):
    """`read(p).validate().certify(p)` reuses the source path — certify must NOT
    write the certificate over the .ags (data loss). It raises and leaves the
    source byte-for-byte intact (verifies the OUTPUT, not just that it ran)."""
    src = _write(tmp_path)
    before = src.read_bytes()
    h = lat.read(src).validate()
    with pytest.raises(Ags4Error):
        h.certify(path=src)
    assert src.read_bytes() == before  # untouched — no clobber


def test_certify_refuses_to_overwrite_another_ags_file(tmp_path):
    """certify won't clobber ANY existing non-certificate file, not just the source."""
    src = _write(tmp_path)
    other = _write(tmp_path, name="other.ags")
    before = other.read_bytes()
    h = lat.read(src).validate()
    with pytest.raises(Ags4Error):
        h.certify(path=other)
    assert other.read_bytes() == before


def test_certify_overwrites_an_existing_certificate(tmp_path):
    """Re-certifying replaces an existing .ags.idx (it IS a certificate) — allowed,
    so the guard protects data files without blocking the normal re-cert flow."""
    src = _write(tmp_path)
    first = lat.read(src).validate().certify()
    assert first.is_file() and first.read_bytes()[:1] == b"{"
    again = lat.read(src).validate().certify()  # overwrite the existing cert
    assert again == first and first.is_file()


# --- the locator must not state a location it does not have -----------------
# A code can occupy more than one section. The index used to be
# `{code: (start, end)}` — a shape that can only express the FIRST — so a sliced
# read of a redeclared group re-parsed part of the file and returned a strict
# SUBSET of the rows the whole-file parse sees, with no error and no warning. The
# DuckDB extension slices from exactly this index. (Cert format v2.)

_REDECLARED = (
    '"GROUP","PROJ"\r\n'
    '"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n"TYPE","ID","X"\r\n'
    '"DATA","P1","Demo"\r\n'
    "\r\n"
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
    '"DATA","BH01"\r\n'
    "\r\n"
    '"GROUP","ABBR"\r\n'
    '"HEADING","ABBR_CODE"\r\n"UNIT",""\r\n"TYPE","X"\r\n'
    '"DATA","CP"\r\n'
    "\r\n"
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
    '"DATA","BH02"\r\n'
)


# `test_index_records_every_span_of_a_redeclared_group` lived here. It built its index
# with `Sidecar.assemble(data, edition, checked_at)` — the factory that let a caller hand
# core a verdict it had not checked, and the reason every certificate this library minted
# claimed `warnings: 0` without looking. That factory is gone (`Sidecar.mint` validates,
# and a redeclared GROUP is a rule violation, so such a file cannot be certified at all).
#
# The property itself is not lost: it is asserted at the source, in
# `laterite-ags4-core/src/index.rs::a_redeclared_group_records_every_span`, where the
# multi-span index is built. That is where the bug was, and where the gate belongs.


def test_cert_format_is_v2(tmp_path):
    """v1 could not express a redeclared group, so it is not readable as v2 — a v1
    cert falls back to a full validation rather than being migrated into a lie."""
    src = _write(tmp_path)
    sc = lat._laterite_native.Sidecar.from_json(
        lat.read(src).validate().certify().read_bytes()
    )
    assert sc.version == 2


# --- the decoder is part of the verdict ------------------------------------------------

#: The same clean file, but PROJ_NAME carries a Greek capital omega — UTF-8 bytes CE A9.
#:
#: Read as UTF-8 that is ONE code point, 937: above the extended-ASCII range Rule 1
#: tolerates, so it is a Rule 1 **error**. Read as windows-1252 the very same two bytes
#: are TWO code points, 206 and 199 — both inside that range, so it is only an **FYI**.
#: One file, two decoders, two verdicts, and they differ in the tier a certificate exists
#: to assert.
OMEGA = CLEAN.replace(
    '"DATA","P1","Clean minimal AGS4 fixture"', '"DATA","P1","\u03a9 site"'
).encode("utf-8")


def test_the_two_decoders_really_do_disagree(tmp_path):
    """The premise, asserted rather than assumed — the rest of this block is only
    interesting because these two numbers differ."""
    src = tmp_path / "omega.ags"
    src.write_bytes(OMEGA)

    assert lat.validate(src).count == 1, "read as UTF-8: a Rule 1 error"
    assert not lat.validate(src).is_valid
    assert lat.validate(src, encoding="windows-1252").count == 0, (
        "read as windows-1252: no error at all"
    )


def test_a_certificate_minted_through_another_decoder_does_not_answer(tmp_path):
    """A certificate seals the BYTES. It does not seal the DECODER — and the rules judge
    the text a decoder produces, not the bytes themselves.

    This was a live false clean: certify under the lenient decoder, then read the very
    same bytes with the default one, and the file came back ``count = 0, certified = True,
    is_valid = True`` — while a plain ``validate()`` of those bytes reported a Rule 1
    error. The certificate now records which decoder produced its verdict, and refuses a
    question asked through another.
    """
    src = tmp_path / "omega.ags"
    src.write_bytes(OMEGA)

    # Error-clean under windows-1252, so it mints — and the stamp says so.
    cert = lat.read(src, encoding="windows-1252").certify()
    assert lat.read(src, index=cert, encoding="windows-1252")._cert.encoding == "windows-1252"

    # The same bytes, read with the default decoder, offering that certificate.
    report = lat.read(src, index=cert).validate().report
    assert not report.certified, "a windows-1252 verdict cannot answer a UTF-8 question"
    assert report.count == 1, "the engine ran, so the Rule 1 error is reported"
    assert not report.is_valid

    # And the decoder it WAS minted under still gets the fast path: a match, not a ban.
    same = lat.read(src, index=cert, encoding="windows-1252").validate(warnings=False).report
    assert same.certified
