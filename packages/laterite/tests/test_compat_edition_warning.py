"""compat warns (never silently) when it auto-resolves an ambiguous TRAN_AGS
to a different edition than python-ags4 would (#190 / O-30 / O-42)."""
import warnings

import pytest
from laterite import compat

_HEAD = (
    '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
    '"DATA","P1"\r\n\r\n"GROUP","TRAN"\r\n'
    '"HEADING","TRAN_DLIM","TRAN_RCON","TRAN_AGS"\r\n"UNIT","","",""\r\n'
    '"TYPE","X","X","X"\r\n"DATA","|","+","{ed}"\r\n'
)


def _write(tmp_path, tran_ags):
    p = tmp_path / "f.ags"
    p.write_text(_HEAD.format(ed=tran_ags), newline="")
    return str(p)


def test_warns_on_4_0_alias_divergence(tmp_path):
    # "4.0" -> laterite 4.0.4, python-ags4 4.0.3 -> warn, naming both editions.
    with pytest.warns(UserWarning, match=r"4\.0\.4.*python-ags4.*4\.0\.3"):
        compat.check_file(_write(tmp_path, "4.0"))


def test_warns_on_bare_4_divergence(tmp_path):
    # bare "4" -> laterite 4.0.4, python-ags4 4.1.1 -> warn.
    with pytest.warns(UserWarning, match=r"python-ags4 1\.2\.0 would use 4\.1\.1"):
        compat.check_file(_write(tmp_path, "4"))


def _edition_warnings(fn):
    """Run `fn`, return only the edition-divergence UserWarnings it raised."""
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        fn()
    return [w for w in caught if "python-ags4" in str(w.message)]


def test_no_warning_when_editions_agree(tmp_path):
    # "4.1" / "4.2" / "4.1.1" resolve the same on both sides -> no edition warning.
    for ed in ("4.1", "4.2", "4.1.1"):
        assert not _edition_warnings(lambda e=ed: compat.check_file(_write(tmp_path, e)))


def test_no_warning_when_dictionary_is_explicit(tmp_path):
    # An explicit edition is the caller's choice — don't second-guess it.
    assert not _edition_warnings(
        lambda: compat.check_file(_write(tmp_path, "4.0"), standard_AGS4_dictionary="4.0.4")
    )
