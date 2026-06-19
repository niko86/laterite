"""PR C — the `.ags.idx` certificate consumer.

`Ags4File.certify()` mints a validity certificate (a clean-validation proof + a
byte-offset index) for an already-validated-clean file; `read(..., index=...)`
loads + freshness-checks one and, if fresh, lets a default `.validate()` skip the
rule engine (returning a `Report.from_cert` whose `resolution == "certified"`).

The two design rules this pins (owner-settled):
- **certify does not auto-validate** — it vouches for a prior clean `.validate()`,
  raising if none was run (or it found findings). No hidden validation.
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


def test_certify_requires_a_prior_validate(tmp_path):
    src = _write(tmp_path)
    with pytest.raises(Ags4Error, match="call .validate.. before .certify"):
        lat.read(src).certify()  # no .validate() — must not auto-validate


def test_certify_refuses_an_unvalidated_clean_file_too(tmp_path):
    # even though the file IS clean, certify won't silently validate it for you.
    src = _write(tmp_path)
    h = lat.read(src)
    with pytest.raises(Ags4Error, match="does not run one"):
        h.certify()


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


# --- consume (validate-skip) -----------------------------------------------


def test_fresh_cert_makes_validate_skip_the_engine(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).validate().certify()

    rep = lat.read(src, index=cert).validate().report
    assert rep.resolution == "certified"  # the engine-skipped sentinel
    assert rep.count == 0 and rep.is_valid
    assert rep.dict_version == "4.2"  # carried from the cert (TRAN_AGS=4.2)


def test_read_without_index_still_runs_the_engine(tmp_path):
    src = _write(tmp_path)
    lat.read(src).validate().certify()
    # no index= → ordinary validation, NOT the certified sentinel
    rep = lat.read(src).validate().report
    assert rep.resolution != "certified"
    assert rep.is_valid


def test_asking_for_more_than_the_cert_vouches_runs_the_engine(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).validate().certify()
    # warnings= asks for more than the cert covers → engine runs (no skip)
    rep = lat.read(src, index=cert).validate(warnings=True).report
    assert rep.resolution != "certified"


def test_certified_read_round_trips_from_text(tmp_path):
    cert = lat.read(text=CLEAN).validate().certify(path=tmp_path / "t.idx")
    rep = lat.read(text=CLEAN, index=cert).validate().report
    assert rep.resolution == "certified" and rep.count == 0


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


def test_report_from_cert_is_a_clean_certified_report(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).validate().certify()
    sidecar = lat._laterite_native.Sidecar.from_json(cert.read_bytes())
    rep = lat.Report.from_cert(sidecar, src=(str(src), None, None))
    assert rep.resolution == "certified"
    assert rep.is_valid and rep.count == 0
    assert rep.dict_version == sidecar.edition
    assert rep.file == str(src)


def test_sidecar_records_provenance_and_index(tmp_path):
    src = _write(tmp_path)
    cert = lat.read(src).validate().certify()
    sc = lat._laterite_native.Sidecar.from_json(cert.read_bytes())
    assert sc.validator == "laterite_ags4" and sc.validator_version
    assert sc.edition == "4.2"
    assert sc.warnings == 0 and sc.fyi == 0  # errors-only validate
    # the byte index locates every group, in file order, tiling [0, size)
    idx = sc.index()
    assert sc.order == ["PROJ", "TRAN", "UNIT", "TYPE"] == list(idx)
    assert idx["PROJ"][0] == 0
    assert idx["TYPE"][1] == sc.size
