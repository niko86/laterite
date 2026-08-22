"""`lat <verb> --help` is help for ONE verb, on every launcher (#509).

`README-cli.md`'s own Usage block promises it:

    lat <command> --help      # help for one command

and it was true of the native binary alone. The Python launcher tested `"--help"
in argv` position-blind across the whole of argv, so `lat certify --help` took the
same branch as `lat --help` and printed the entire 203-line guide — the document
the reader reached for `--help` to avoid. The Node launcher had no `--help` path
at all: the flag fell through to the unknown-flag refusal, exit 5.

WHERE THE TEXT COMES FROM, and why it is not new prose. Each verb already has a
`## <verb> …` section in the guide, written and gated: `gen_wiki_cli.py` pins the
verb list against `cli.rs`, and `test_cli_readme_flags.py` pins each section's
flags against clap's derive. Scoped help slices that section rather than
introducing a fourth description of the same flags — which would be a fourth thing
to keep in step, and this issue is about copies that did not stay in step.

`## Global options` is appended because clap does the same: `lat certify --help`
on the binary lists `--quiet` and the dictionary flags, which belong to no single
verb. A scoped help that hid them would send a reader back to the full guide.

THE TRANSPORT VERBS SHARE A SECTION. `## transport — pack / unpack / lock /
unlock` documents four verbs together, so the lookup matches a verb against the
WORDS of a heading rather than its first token. That is the only heading in the
guide where the two differ, and getting it wrong is silent: `lat lock --help`
would fall back to the whole document, which is the defect being fixed.
"""

from __future__ import annotations

import io
from contextlib import redirect_stdout

import pytest

pytestmark = pytest.mark.needs_env

from laterite import _cli  # noqa: E402  (the marker must precede the built import)

#: Every verb the launcher accepts. Read from the parser rather than restated, so
#: a new verb arrives here already under test — the argument
#: `test_docs_examples.py` makes for its glob.
VERBS = sorted(_cli.verbs())


def _run(argv: list[str]) -> tuple[int, str]:
    buf = io.StringIO()
    with redirect_stdout(buf):
        code = _cli.main(argv)
    return code, buf.getvalue()


def test_there_are_verbs_to_check() -> None:
    """Zero is a bad witness: an empty verb list makes every case below vacuous."""
    assert VERBS, "the parser exposes no verbs"


@pytest.mark.parametrize("verb", VERBS)
def test_verb_help_is_scoped_to_that_verb(verb: str) -> None:
    """The whole point: shorter than the guide, and about the verb asked for."""
    _, full = _run(["--readme"])
    code, out = _run([verb, "--help"])

    assert code == 0, f"`lat {verb} --help` exited {code}"
    assert out != full, (
        f"`lat {verb} --help` printed the entire guide — the position-blind "
        "`--help in argv` test is back"
    )
    assert len(out.splitlines()) < len(full.splitlines()), (
        f"`lat {verb} --help` is not shorter than `lat --readme`"
    )
    assert f"lat {verb}" in out or f"## {verb}" in out or verb in out.split("\n")[0], (
        f"`lat {verb} --help` never names the verb:\n{out[:400]}"
    )


@pytest.mark.parametrize("verb", VERBS)
def test_verb_help_carries_the_global_options(verb: str) -> None:
    """clap lists them under every verb; a scoped help that dropped them would
    send the reader back to the document they were avoiding."""
    _, out = _run([verb, "--help"])
    assert "--quiet" in out, f"`lat {verb} --help` omits the global options"


@pytest.mark.parametrize("verb", ["pack", "unpack", "lock", "unlock"])
def test_the_shared_transport_section_answers_for_each_of_its_verbs(verb: str) -> None:
    """The one heading whose first word is not a verb. A first-token lookup passes
    every other case and silently falls back to the full guide here."""
    _, out = _run([verb, "--help"])
    assert "transport" in out, f"`lat {verb} --help` did not find the shared section"
    assert len(out.splitlines()) < 60, (
        f"`lat {verb} --help` looks like the whole guide, not a section"
    )


def test_bare_help_still_prints_the_whole_guide() -> None:
    """The other half of the contract — and the direction a scoping change breaks
    by accident. `lat --help` has no verb to scope to."""
    _, full = _run(["--readme"])
    for flag in ("--help", "-h"):
        code, out = _run([flag])
        assert code == 0 and out == full, f"`lat {flag}` no longer prints the guide"


def test_a_bare_file_scopes_help_to_validate() -> None:
    """`lat <file> --help` is `lat validate <file> --help`, the shorthand this
    launcher already applies to dispatch.

    Measured against the binary, which answers it with validate's help — its argv
    pre-scan splices the default verb in before clap sees the flag. Printing the
    whole guide here instead would be a launcher divergence introduced BY the fix
    for launcher divergence.
    """
    _, full = _run(["--readme"])
    code, out = _run(["delivery.ags", "--help"])
    assert code == 0
    assert out != full, "`lat <file> --help` fell back to the whole guide"
    assert "validate" in out


def test_help_after_a_verb_beats_the_verbs_own_arguments() -> None:
    """`--help` wins over doing the work, the way clap orders it. A file argument
    alongside must not send certify off to mint a certificate."""
    code, out = _run(["certify", "examples/sample_site.ags", "--help"])
    assert code == 0
    assert "certify" in out
