"""The encoding-label vocabulary is shared across surfaces (the parse leaf's
`resolve_encoding`), so the friendly legacy spellings — notably the hyphenated
`latin-1`, which is NOT a WHATWG label — are accepted everywhere they used to
diverge: Python *raised* `BadDictError`, Node silently fell back to UTF-8, and
only the browser mapped it to Windows-1252.
"""

from __future__ import annotations

import laterite as L
import pytest

# A cp1252 file: the degree sign 0xB0 is a clean extended-ASCII char under
# Windows-1252 (FYI at most), but invalid UTF-8 (→ U+FFFD → Rule 1 error).
_CP1252 = (
    '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
    '"TYPE","ID","X"\r\n"DATA","P1","20\xb0 slope"\r\n'
).encode("cp1252")


def _has_rule1(report) -> bool:
    return "AGS Format Rule 1" in report.by_rule()


def _write(tmp_path):
    p = tmp_path / "legacy.ags"
    p.write_bytes(_CP1252)
    return str(p)


@pytest.mark.parametrize(
    "label", ["latin-1", "latin1", "iso-8859-1", "windows-1252", "cp1252", "Latin-1", " latin-1 "]
)
def test_windows1252_spellings_all_accepted_and_equivalent(tmp_path, label):
    """Every legacy spelling — including the hyphenated `latin-1` (and mixed
    case / surrounding whitespace) that used to raise — decodes as Windows-1252,
    so the degree sign is clean extended-ASCII and no spurious Rule 1 appears."""
    assert not _has_rule1(L.read(_write(tmp_path), encoding=label).validate().report)


def test_unknown_encoding_still_raises(tmp_path):
    """A genuinely unknown label is still an error on the library surface, not a
    silent UTF-8 fallback — the fix widened the accepted set, it didn't mute
    mistakes."""
    with pytest.raises(Exception) as exc:  # BadDictError
        L.read(_write(tmp_path), encoding="totally-bogus-encoding").validate()
    assert "unknown encoding" in str(exc.value).lower()
