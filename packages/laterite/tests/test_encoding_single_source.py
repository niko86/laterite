"""An encoding label means ONE thing, on every surface — or is refused everywhere.

Three bugs lived here, and only the last is the one people expect:

1. **`--encoding` was a complete no-op on the npx launcher.** Every handler accepted
   the flag (its arg parser has one global valued-flag set) and dropped it on the
   floor. `lat validate legacy.ags --encoding cp1252` decoded as UTF-8 and reported
   findings that were artefacts of the wrong decoder — blaming the file for the
   caller's ignored flag. A knob-NAME gate cannot see this: `--encoding` exists on
   both surfaces, spelled identically. Only comparing OUTPUT catches it.

2. **An unknown label silently became UTF-8** on Node and in the browser, while
   Python raised. That reads like leniency and behaves like corruption: the bytes
   `C3 A9` decode cleanly as `é` in UTF-8 and `Ã©` in cp1252, so a caller who typed
   `cp1252x` got the wrong text and a clean bill of health. The wasm crate even had a
   test *codifying* it ("an unknown label falls back to UTF-8, not an error").

3. **`latin9` / `latin-9` worked only on the `lat` binary**, which kept a private
   label table wider than the shared leaf's. The Python library rejected the same
   labels. Promoted into the leaf.

The parse leaf was right the whole time. The bugs were all in the thin wrappers
ABOVE it — which is why the surface census asks each launcher's OWN resolver, never
the leaf: a census that asked the leaf would have agreed with itself and seen
nothing.
"""

from __future__ import annotations

from pathlib import Path

import laterite
import pytest
from laterite import _cli
from laterite import _laterite_native as _native

#: Bytes `C3 A9` decode cleanly under BOTH encodings, to DIFFERENT text. So the
#: encoding choice silently changes the value rather than erroring — which is exactly
#: what makes a silent fallback dangerous rather than merely sloppy.
_AMBIGUOUS = (
    b'"GROUP","PROJ"\r\n'
    b'"HEADING","PROJ_ID","PROJ_NAME"\r\n'
    b'"UNIT","",""\r\n'
    b'"TYPE","ID","X"\r\n'
    b'"DATA","P1","Caf\xc3\xa9"\r\n'
)


def test_the_two_decodings_really_do_differ() -> None:
    """The premise of every test below: this input is genuinely ambiguous."""
    assert _AMBIGUOUS.decode("utf-8").count("Café") == 1
    assert _AMBIGUOUS.decode("cp1252").count("CafÃ©") == 1


# --- the label table ---------------------------------------------------------


@pytest.mark.parametrize(
    ("label", "expected"),
    [
        ("utf-8", "UTF-8"),
        ("utf8", "UTF-8"),
        ("cp1252", "windows-1252"),
        ("windows-1252", "windows-1252"),
        ("latin1", "windows-1252"),
        # Hyphenated. WHATWG's `for_label` does NOT know it; the leaf does.
        ("latin-1", "windows-1252"),
        ("iso-8859-1", "windows-1252"),
        ("iso-8859-15", "ISO-8859-15"),
        # The two the `lat` binary alone used to accept, via a private table.
        ("latin9", "ISO-8859-15"),
        ("latin-9", "ISO-8859-15"),
        ("l9", "ISO-8859-15"),
        # Not special-cased: proves `for_label` is still reached, i.e. nobody has
        # replaced the fallthrough with a hand-list.
        ("shift_jis", "Shift_JIS"),
    ],
)
def test_label_resolves_the_same_everywhere(label: str, expected: str) -> None:
    assert _native.resolve_encoding_label(label) == expected


def test_an_unknown_label_resolves_to_nothing() -> None:
    """The policy, in one assertion. NOT a fallback to UTF-8."""
    assert _native.resolve_encoding_label("cp1252x") is None
    assert _native.resolve_encoding_label("not-a-charset") is None


def test_no_label_means_utf8() -> None:
    """Absent is not unknown — a caller who says nothing gets the AGS4 default."""
    assert _native.resolve_encoding_label(None) == "UTF-8"
    assert _native.resolve_encoding_label("") == "UTF-8"


# --- the policy, through the real API ---------------------------------------


def test_a_typod_label_raises_rather_than_silently_decoding(tmp_path: Path) -> None:
    """The corruption vector, closed. The caller is told about their typo instead of
    being handed text decoded by an encoding they never asked for."""
    f = tmp_path / "ambig.ags"
    f.write_bytes(_AMBIGUOUS)
    with pytest.raises(Exception, match="unknown encoding"):
        laterite.read(f, encoding="cp1252x")


def test_the_label_actually_changes_the_VALUE(tmp_path: Path) -> None:
    """`--encoding` is honoured, not merely accepted.

    This is the assertion npx failed: it took the flag and ignored it. Asserting the
    flag EXISTS proves nothing — assert that it changes the bytes you get back.
    """
    f = tmp_path / "ambig.ags"
    f.write_bytes(_AMBIGUOUS)

    as_utf8 = laterite.read(f, encoding="utf-8").table("PROJ")["PROJ_NAME"][0]
    as_cp1252 = laterite.read(f, encoding="cp1252").table("PROJ")["PROJ_NAME"][0]

    assert as_utf8 == "Café"
    assert as_cp1252 == "CafÃ©"
    assert as_utf8 != as_cp1252, "if these match, the encoding is being ignored"


def test_latin9_is_accepted_by_the_library_not_just_the_binary(tmp_path: Path) -> None:
    """`latin-9` used to work on `lat` and raise here — the private-table divergence.

    All three name ISO-8859-15, so all three must decode identically.
    """
    f = tmp_path / "ambig.ags"
    f.write_bytes(_AMBIGUOUS)
    values = {
        label: laterite.read(f, encoding=label).table("PROJ")["PROJ_NAME"][0]
        for label in ("latin9", "latin-9", "iso-8859-15", "l9")
    }
    assert len(set(values.values())) == 1, f"aliases of one encoding disagreed: {values}"


# --- the uvx launcher --------------------------------------------------------


def test_the_uvx_cli_rejects_a_typod_label(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    f = tmp_path / "ambig.ags"
    f.write_bytes(_AMBIGUOUS)
    assert _cli.main(["validate", str(f), "--encoding", "cp1252x"]) == 5
    assert "unknown encoding" in capsys.readouterr().err


def test_the_uvx_cli_honours_a_valid_label(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    f = tmp_path / "ambig.ags"
    f.write_bytes(_AMBIGUOUS)
    assert _cli.main(["validate", str(f), "--encoding", "cp1252"]) != 5
    capsys.readouterr()


# --- the census probe list ---------------------------------------------------


def test_census_probe_lists_agree() -> None:
    """The three launchers must probe the SAME labels.

    If they drifted, each would answer a different question and the census's
    agreement would be meaningless — a gate that compares nothing and reports
    success. The Rust list is the authority (`ENCODING_PROBES` in
    `commands/census.rs`); this pins Python's copy to it.
    """
    import re

    rust = (
        Path(__file__).resolve().parents[3]
        / "rust-packages"
        / "laterite-ags4-check"
        / "src"
        / "commands"
        / "census.rs"
    ).read_text()
    block = re.search(r"ENCODING_PROBES: &\[&str\] = &\[(.*?)\];", rust, re.S)
    assert block, "ENCODING_PROBES not found in census.rs"
    labels = tuple(re.findall(r'"([^"]+)"', block.group(1)))
    assert labels == _cli._ENCODING_PROBES, (
        "the Rust and Python census probe lists have drifted — the launchers would be "
        "answering different questions and the census would call it agreement"
    )
    assert "cp1252x" in labels, "the policy pin (an unknown label) must stay in the list"
