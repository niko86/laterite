"""The doc-truth gate must say what it declined to look at.

`check_doc_refs` only treats a backticked token as a path if it contains a `/`.
That is a deliberate precision heuristic — without it `read()` and `pandas` are
"dead references" — but it is also an unstated scope, and the gate reported green
without ever mentioning it. `compat.py` sat cited in the docs and unchecked for
months; the gate engaged only once the citation grew a slash, and then failed for
being repo-relative. A blind spot found by tripping over it is the failure mode
gates exist to prevent (#460, item 1 of #295).

The house rule this pins is the general one: **a gate that drops input says what
it dropped**. So the count is asserted off the gate's own stdout, on a clean run
and on a failing one, because a report nobody asserts is the same silence one
level up.

Recall is deliberately NOT widened here — `README.md` in prose still goes
unchecked. These tests pin the reporting, and the count they make visible is the
evidence for deciding the widening separately.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load():
    """Import `tools/check_doc_refs.py` as a module — `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "check_doc_refs", REPO / "tools" / "check_doc_refs.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_doc_refs"] = mod
    spec.loader.exec_module(mod)
    return mod


gate = _load()


@pytest.fixture
def fake_repo(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """A checkout-shaped tree the gate scans instead of this repo.

    Synthetic rather than live: an assertion on the real corpus' count would go
    red the day someone legitimately edits a doc, which would make a reporting
    test a brake on prose.
    """
    (tmp_path / "tools").mkdir()
    (tmp_path / "tools" / "check_doc_refs.py").touch()  # a live reference to cite
    (tmp_path / "rust-packages").mkdir()
    monkeypatch.setattr(gate, "ROOT", tmp_path)
    monkeypatch.setattr(gate, "REPO_DOCS", ["README.md"])
    return tmp_path


def _run(args: list[str], capsys: pytest.CaptureFixture[str]) -> tuple[str, int]:
    monkeyargs = ["check_doc_refs.py", *args]
    old, sys.argv = sys.argv, monkeyargs
    code = 0
    try:
        gate.main()
    except SystemExit as exc:  # --check on a dead reference
        code = int(exc.code or 0)
    finally:
        sys.argv = old
    return capsys.readouterr().out, code


# --- the partition itself -------------------------------------------------


def test_bare_backticked_filename_is_counted_as_skipped() -> None:
    targets, skipped = gate._targets("see `compat.py` and `pyproject.toml`")
    assert targets == set()
    assert skipped == {"compat.py", "pyproject.toml"}


def test_path_shaped_backticked_token_is_checked_not_skipped() -> None:
    targets, skipped = gate._targets("see `tools/check_doc_refs.py`")
    assert targets == {"tools/check_doc_refs.py"}
    assert skipped == set()


def test_markdown_link_to_a_bare_filename_is_still_checked() -> None:
    """The scope is backticks only — `LINK_RE` matches were never filtered."""
    targets, skipped = gate._targets("[compat](COMPAT.md)")
    assert targets == {"COMPAT.md"}
    assert skipped == set()


def test_a_linked_target_does_not_cancel_the_same_token_in_prose() -> None:
    """Counted per citation, not per filename — deliberately, so the number means
    "references this gate did not resolve", which is what a widening decision
    needs. The doc is fine; the backticked citation is still one nothing read."""
    targets, skipped = gate._targets("[c](COMPAT.md) beside prose `COMPAT.md`")
    assert targets == {"COMPAT.md"}
    assert skipped == {"COMPAT.md"}


def test_a_token_the_regex_never_matched_is_not_counted() -> None:
    """`read()` and `pandas` are not suffixed, so they are not dropped input."""
    _, skipped = gate._targets("call `read()` on a `pandas` frame")
    assert skipped == set()


# --- the report, which is the thing that was missing -----------------------


def test_the_count_prints_on_a_clean_run(
    fake_repo: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    (fake_repo / "README.md").write_text(
        "`compat.py` lives beside `tools/check_doc_refs.py`\n", encoding="utf-8"
    )
    out, code = _run(["--check"], capsys)
    assert code == 0
    assert "every referenced path exists" in out
    assert "skipped 1 slash-free backticked token" in out


def test_the_count_prints_on_a_failing_run(
    fake_repo: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """Not just on the happy path: a red run must still declare its blind spot."""
    (fake_repo / "README.md").write_text(
        "`compat.py`, and [gone](tools/gone.py)\n", encoding="utf-8"
    )
    out, code = _run(["--check"], capsys)
    assert code == 1
    assert "does not exist" in out
    assert "skipped 1 slash-free backticked token" in out


def test_zero_skipped_reports_zero(
    fake_repo: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    (fake_repo / "README.md").write_text(
        "only `tools/check_doc_refs.py` here\n", encoding="utf-8"
    )
    out, _ = _run([], capsys)
    assert "skipped 0 slash-free backticked token" in out


def test_published_crate_readmes_are_counted_too(
    fake_repo: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """The strict crates.io pass runs the same partition, so it reports too."""
    (fake_repo / "README.md").write_text("nothing here\n", encoding="utf-8")
    crate = fake_repo / "rust-packages" / "laterite-thing"
    crate.mkdir()
    (crate / "Cargo.toml").write_text(
        '[package]\nname = "laterite-thing"\n', encoding="utf-8"
    )
    (crate / "README.md").write_text("built by `build.rs`\n", encoding="utf-8")
    out, _ = _run([], capsys)
    assert "skipped 1 slash-free backticked token" in out


def test_the_tokens_are_listed_on_request(
    fake_repo: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """The count says how many; deciding whether to widen recall needs which."""
    (fake_repo / "README.md").write_text(
        "`compat.py` and `Cargo.toml`\n", encoding="utf-8"
    )
    out, _ = _run(["--skipped"], capsys)
    assert "README.md" in out
    assert "compat.py" in out
    assert "Cargo.toml" in out
    bare, _ = _run([], capsys)
    assert "compat.py" not in bare
