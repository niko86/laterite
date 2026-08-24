"""The rule catalogue's O-N citations must resolve to a LIVE canon record.

Two bodies of divergence prose, one gated. The canon (`observations.json`) has
`gen_observations.py` holding its rendered views in step; the rule catalogue's
practitioner notes had nothing, and they are the ones a reader meets in the
webapp beside a failing rule. When the gate was first run it found two: a note on
Rule 9 citing O-10 and one on Rule 14 citing O-20, both superseded by O-30, one
of them saying so in its own text.

What is asserted here is the gate's own behaviour, driven at synthetic data
rather than by dirtying the real files — including the scope line, because the
gate cannot check rule ATTACHMENT at all (the canon records no rule per
observation) and a green tick that did not say so would read as a full check.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pytest

REPO = Path(__file__).resolve().parents[1]


def _load():
    """Import `tools/check_rule_catalogue_refs.py` — `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "check_rule_catalogue_refs", REPO / "tools" / "check_rule_catalogue_refs.py"
    )
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


gate = _load()


def _write(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    *,
    cited: str,
    record: dict,
) -> None:
    catalogue = tmp_path / "rules_meta.json"
    canon = tmp_path / "observations.json"
    catalogue.write_text(
        json.dumps(
            {
                "schema_version": 1,
                "rules": [
                    {"rule": "9", "observations": [{"id": cited, "note": "a note"}]}
                ],
            }
        ),
        encoding="utf-8",
    )
    canon.write_text(
        json.dumps({"sections": [{"observations": [record]}]}), encoding="utf-8"
    )
    monkeypatch.setattr(gate, "CATALOGUE", catalogue)
    monkeypatch.setattr(gate, "CANON", canon)


LIVE = {"id": "O-30", "kind": "VARIANCE", "title": "t", "upstream": False, "body": "b"}
RETIRED = {**LIVE, "id": "O-10", "status": "superseded", "resolved_by": "O-30"}


def test_a_citation_to_a_live_record_passes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _write(tmp_path, monkeypatch, cited="O-30", record=LIVE)
    assert gate.main() == 0


def test_a_citation_to_a_record_that_does_not_exist_fails(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The renumbering case: the number resolves to nothing, or later to
    something else entirely, and the note reads as sourced either way."""
    _write(tmp_path, monkeypatch, cited="O-99", record=LIVE)
    assert gate.main() == 1
    assert "not in the canon" in capsys.readouterr().err


def test_a_citation_to_a_superseded_record_fails_and_names_its_replacement(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The failure that motivated the gate. The message has to carry
    `resolved_by`, or the fix is a search rather than an edit."""
    _write(tmp_path, monkeypatch, cited="O-10", record=RETIRED)
    assert gate.main() == 1
    err = capsys.readouterr().err
    assert "superseded" in err
    assert "resolved by O-30" in err


def test_one_rule_citing_the_same_record_twice_fails(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The failure that REPOINTING produces, and the reason this check exists:
    fixing a superseded citation lands it on the replacement, which the same rule
    may already cite. The reader then gets one divergence told twice in two
    voices, and nothing else here would notice."""
    catalogue = tmp_path / "rules_meta.json"
    canon = tmp_path / "observations.json"
    catalogue.write_text(
        json.dumps(
            {
                "rules": [
                    {
                        "rule": "9",
                        "observations": [
                            {"id": "O-30", "note": "one voice"},
                            {"id": "O-30", "note": "another voice"},
                        ],
                    }
                ]
            }
        ),
        encoding="utf-8",
    )
    canon.write_text(
        json.dumps({"sections": [{"observations": [LIVE]}]}), encoding="utf-8"
    )
    monkeypatch.setattr(gate, "CATALOGUE", catalogue)
    monkeypatch.setattr(gate, "CANON", canon)
    assert gate.main() == 1
    assert "more than once" in capsys.readouterr().err


def test_prose_that_differs_between_the_two_never_fails(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The two are written for different readers and MUST differ — the canon in
    observed/spec/assessment form, the catalogue for someone looking at a broken
    file. Identity is the subject; wording is not."""
    _write(
        tmp_path,
        monkeypatch,
        cited="O-30",
        record={**LIVE, "title": "nothing like the note", "body": "nor this"},
    )
    assert gate.main() == 0


def test_it_states_the_attachment_blind_spot_on_a_passing_run(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The direction that goes quiet. This gate checks that a cited record exists
    and is live — never that the note hangs off the right rule, which the canon
    does not record. A pass that did not say so would read as a full check."""
    _write(tmp_path, monkeypatch, cited="O-30", record=LIVE)
    assert gate.main() == 0
    out = capsys.readouterr().out
    assert "ATTACHMENT unchecked" in out
    assert "citation(s) checked" in out


def test_the_real_files_pass() -> None:
    """The gate against the tree it ships with — the ratchet itself."""
    assert gate.main() == 0
