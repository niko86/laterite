"""`check_files` — the one check that reads state the AGS4 bytes do not contain.

Rule 20 has two halves. The CONTENT half ("every FILE_FSET used is defined in the
FILE group") is a pure function of the file. The WORLD half ("`FILE/<fset>/<name>`
exists beside the .ags") stats the filesystem — someone can delete that tree without
touching a byte of the delivery, and the verdict flips.

So the WORLD half needs a path. Ask for it against ``bytes`` or ``str`` and there is
no directory to look in — the question cannot be answered. The engine used to answer
it anyway: it dropped the request and reported Rule 20 clean. Every bytes/text read
took that path, and the browser takes it always. A false clean, with no certificate
involved at all.

These tests pin the fix by its OUTPUT: the same call that returned a clean report now
raises.
"""

from __future__ import annotations

import laterite as lat
import pytest
from laterite import WorldCheckRequiresSourceError

RULE_20 = "AGS Format Rule 20"

# Content-clean (PROJ + TRAN + UNIT + TYPE), plus a FILE group declaring one
# attachment. Rule 20's CONTENT half is satisfied — FS1 *is* defined in FILE — so the
# only thing left to say about Rule 20 is whether FILE/FS1/photo.jpg is really there.
WITH_ATTACHMENT = "\r\n".join(
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
        '"GROUP","FILE"',
        '"HEADING","FILE_FSET","FILE_NAME"',
        '"UNIT","",""',
        '"TYPE","X","X"',
        '"DATA","FS1","photo.jpg"',
        "",
    ]
)
DATA = WITH_ATTACHMENT.encode("utf-8")


def _write(tmp_path, name="delivery.ags"):
    p = tmp_path / name
    p.write_bytes(DATA)  # write_bytes: keep CRLF (no translation)
    return p


def _rules(report) -> set[str]:
    return set(report.by_rule())


def test_bytes_plus_check_files_refuses_instead_of_reporting_clean():
    # THE BUG. Before: a clean report, Rule 20 silently unasked. Now: a refusal.
    with pytest.raises(WorldCheckRequiresSourceError) as e:
        lat.read(DATA).validate(check_files=True)
    assert e.value.exit_code == 5
    assert "path" in str(e.value)


def test_text_plus_check_files_refuses_too():
    # The text modality is the bytes modality's twin, and had the same hole.
    with pytest.raises(WorldCheckRequiresSourceError):
        lat.read(WITH_ATTACHMENT).validate(check_files=True)


def test_a_path_makes_the_question_answerable_and_rule_20_fires(tmp_path):
    # The refusal above is not the engine being unable to do the check. Hand it a
    # path with no FILE/ tree beside it and Rule 20 speaks. What changed is that
    # "I cannot answer" no longer looks exactly like "nothing is wrong".
    src = _write(tmp_path)
    rep = lat.read(src).validate(check_files=True).report
    assert RULE_20 in _rules(rep), f"missing FILE/ tree must flag Rule 20: {_rules(rep)}"
    assert not rep.is_valid


def test_the_same_path_is_clean_once_the_tree_exists(tmp_path):
    # ...and clean once the attachment is really there. Two different verdicts over
    # byte-identical .ags content — which is precisely why a certificate (which
    # keys off a SHA-256 of those bytes) may never vouch for this half of Rule 20.
    src = _write(tmp_path)
    leaf = tmp_path / "FILE" / "FS1"
    leaf.mkdir(parents=True)
    (leaf / "photo.jpg").write_bytes(b"x")

    rep = lat.read(src).validate(check_files=True).report
    assert RULE_20 not in _rules(rep), f"tree present → Rule 20 clean: {_rules(rep)}"
    assert rep.is_valid


def test_bytes_without_check_files_are_unaffected():
    # The everyday library call: content-only, path-independent, still clean. The
    # fix refuses a request that was never answerable, not one that was.
    rep = lat.read(DATA).validate().report
    assert RULE_20 not in _rules(rep)
    assert rep.is_valid
