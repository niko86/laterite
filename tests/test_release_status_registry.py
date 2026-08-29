"""The release report reads crates.io, and says so honestly when it cannot.

`release_status.py` derived everything from the tree and nothing from a
registry, so "stamped here" was the only notion of released it had.
`laterite-ags4-emit` 0.12.0 was stamped, written up as published, and absent
from crates.io, and no gate, test or report in this repo could tell — the tree
was self-consistent and the registry was never asked.

Asking introduces the failure the report must not have. A network read can fail,
and a report that turns a failed read into PUBLISH OWED is a nag that cries wolf
until someone switches it off. So the tests that matter here are the ones
pinning **unreachable as its own state**: it must reach the render intact, shout
nothing, and be counted out loud.

The house rule underneath is the general one: **a gate that drops input says what
it dropped**. An unreachable crate is dropped input, so the count is asserted off
the report's own stdout on a clean run, an unreachable run, and a skipped one —
a report nobody asserts is the same silence one level up.

No test here touches the network: `fetch_index` is monkeypatched at the urlopen
seam, and everything above it takes the fetcher as a parameter.
"""

from __future__ import annotations

import importlib.util
import json
import sys
import urllib.error
from email.message import Message
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load():
    """Import `tools/release/release_status.py` — `tools/release/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "release_status", REPO / "tools" / "release" / "release_status.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["release_status"] = mod
    spec.loader.exec_module(mod)
    return mod


rs = _load()


def _status(*crates: dict) -> dict:
    """A report dict shaped like `collect()`'s, without running git or the network."""
    return {
        "engine_crates": [
            {
                "crate": c.get("crate", "laterite-ags4-core"),
                "version": c.get("version", "0.12.0"),
                "last_stamp": "abc1234 2026-08-29 release: x",
                "registry_state": c["registry_state"],
                "registry_latest": c.get("registry_latest", "0.11.0"),
                "api_added": c.get("api_added", 0),
                "api_removed": 0,
                "api_removed_names": [],
                "verdict": c.get("verdict", "none"),
            }
            for c in crates
        ],
        "product": {
            "version": "0.12.0",
            "last_stamp": "abc 2026-08-29 x",
            "verdict": "none",
        },
        "changelog_unreleased": {},
    }


# --- the index path: a wrong path 404s, and a 404 reads as "never published" ---


@pytest.mark.parametrize(
    ("crate", "expected"),
    [
        ("a", "1/a"),
        ("ab", "2/ab"),
        ("abc", "3/a/abc"),
        ("abcd", "ab/cd/abcd"),
        ("laterite-ags4-emit", "la/te/laterite-ags4-emit"),
        ("Laterite-AGS4-Emit", "la/te/laterite-ags4-emit"),
    ],
)
def test_index_path_follows_cargos_name_length_layout(crate, expected):
    assert rs.index_path(crate) == expected


# --- the five states, each of which asks for a different action ---


def test_stamped_version_absent_from_the_index_is_a_publish_owed():
    versions = [{"vers": "0.9.0", "yanked": False}, {"vers": "0.11.0", "yanked": False}]
    assert rs.registry_state("0.12.0", versions) == ("owed", "0.11.0")


def test_stamped_version_present_and_live_is_ok():
    versions = [
        {"vers": "0.11.0", "yanked": False},
        {"vers": "0.12.0", "yanked": False},
    ]
    assert rs.registry_state("0.12.0", versions) == ("ok", "0.12.0")


def test_stamped_version_present_but_yanked_is_its_own_state():
    """crates.io is append-only, so a yanked stamp can never be re-published.

    Reporting it as a plain PUBLISH OWED would send someone to run a publish
    that the registry will refuse; it needs a human and a new number.
    """
    versions = [{"vers": "0.12.0", "yanked": True}]
    assert rs.registry_state("0.12.0", versions)[0] == "yanked"


def test_crate_absent_from_the_registry_entirely_is_a_first_publish():
    assert rs.registry_state("0.1.0", [])[0] == "new"


def test_unreachable_registry_is_unknown_not_unpublished():
    """The whole point: `None` must not collapse into the same answer as `[]`."""
    assert rs.registry_state("0.12.0", None)[0] == "unknown"


# --- an unreachable read must not shout, in either renderer ---


def _crate_row(out: str, crate: str) -> str:
    """The one report line for `crate`.

    Asserted against the ROW, not the whole report: the standing footer explains
    what PUBLISH OWED means and so contains the phrase on every run, which would
    make a whole-output assertion pass for the wrong reason.
    """
    rows = [ln for ln in out.splitlines() if ln.strip().startswith(crate)]
    assert len(rows) == 1, f"expected one row for {crate}, got {rows}"
    return rows[0]


def test_unreachable_crate_shouts_nothing_in_the_report():
    out = rs.render(
        _status(
            {
                "crate": "laterite-ags4-core",
                "registry_state": "unknown",
                "registry_latest": "?",
            }
        )
    )
    assert "PUBLISH OWED" not in _crate_row(out, "laterite-ags4-core")


def test_unreachable_crate_shouts_nothing_in_the_nag():
    line = rs.render_nag(_status({"registry_state": "unknown", "registry_latest": "?"}))
    assert "not on crates.io" not in line


def test_the_nag_never_claims_all_clear_over_a_registry_it_could_not_reach():
    line = rs.render_nag(_status({"registry_state": "unknown", "registry_latest": "?"}))
    assert "nothing owed" in line
    assert "1 crate(s) unreachable" in line


def test_an_unpublished_stamp_does_shout_when_the_registry_answered():
    status = _status({"registry_state": "owed", "registry_latest": "0.11.0"})
    assert "PUBLISH OWED" in rs.render(status)
    assert "not on crates.io" in rs.render_nag(status)


def test_a_bump_owed_and_a_publish_owed_are_both_named_on_one_line():
    """Two different people act on these, so one must not mask the other."""
    line = rs.render(
        _status(
            {
                "registry_state": "owed",
                "verdict": "minor",
                "api_added": 3,
            }
        )
    )
    assert "MINOR, PUBLISH OWED" in line


# --- the dropped-input report, on every run ---


def test_scope_line_counts_the_crates_it_could_not_ask_about():
    out = rs.render(
        _status(
            {"crate": "a-crate", "registry_state": "ok", "registry_latest": "0.11.0"},
            {"crate": "b-crate", "registry_state": "unknown", "registry_latest": "?"},
        )
    )
    assert "registry: 1 of 2 crates answered, 1 unreachable, 0 not asked" in out


def test_scope_line_reports_a_clean_run_too():
    """A count printed only when it is non-zero is a count nobody trusts."""
    out = rs.render(_status({"registry_state": "ok", "registry_latest": "0.12.0"}))
    assert "registry: 1 of 1 crates answered, 0 unreachable, 0 not asked" in out


def test_skipping_the_registry_says_so_and_claims_nothing():
    out = rs.render(_status({"registry_state": "skipped", "registry_latest": "—"}))
    assert "NOT ASKED (--no-registry)" in out
    # The standing footer describes a read that did not happen this run.
    assert "crates.io IS read" not in out


def test_collect_with_no_fetcher_asks_nothing_and_marks_every_crate_skipped():
    """`--no-registry` is the same code path with nothing behind it."""
    status = rs.collect(fetch=None)
    assert status["engine_crates"], "the snapshot census should not be empty"
    assert {c["registry_state"] for c in status["engine_crates"]} == {"skipped"}


# --- the network seam ---


class _Resp:
    def __init__(self, body: bytes):
        self._body = body

    def read(self) -> bytes:
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False


def test_fetch_index_parses_the_ndjson_the_sparse_index_returns(monkeypatch):
    body = (
        json.dumps({"name": "c", "vers": "0.9.0", "yanked": False})
        + "\n"
        + json.dumps({"name": "c", "vers": "0.11.0", "yanked": False})
        + "\n"
    ).encode()
    monkeypatch.setattr(rs.urllib.request, "urlopen", lambda *a, **k: _Resp(body))
    assert [v["vers"] for v in rs.fetch_index("c-crate")] == ["0.9.0", "0.11.0"]


def test_fetch_index_treats_404_as_never_published_not_as_a_failure(monkeypatch):
    def boom(*_a, **_k):
        raise urllib.error.HTTPError("u", 404, "nope", Message(), None)

    monkeypatch.setattr(rs.urllib.request, "urlopen", boom)
    assert rs.fetch_index("c-crate") == []


def test_fetch_index_returns_none_when_the_registry_cannot_be_reached(monkeypatch):
    def boom(*_a, **_k):
        raise urllib.error.URLError("dns")

    monkeypatch.setattr(rs.urllib.request, "urlopen", boom)
    assert rs.fetch_index("c-crate") is None


def test_fetch_index_returns_none_on_a_server_error(monkeypatch):
    """A 500 is not evidence that a crate is unpublished."""

    def boom(*_a, **_k):
        raise urllib.error.HTTPError("u", 503, "unavailable", Message(), None)

    monkeypatch.setattr(rs.urllib.request, "urlopen", boom)
    assert rs.fetch_index("c-crate") is None


def test_fetch_index_refuses_a_half_read_index(monkeypatch):
    monkeypatch.setattr(
        rs.urllib.request,
        "urlopen",
        lambda *a, **k: _Resp(b'{"vers": "0.9.0"}\n{trunca'),
    )
    assert rs.fetch_index("c-crate") is None


# --- version ordering ---


def test_version_key_orders_the_plain_releases_this_tier_ships():
    assert rs.version_key("0.9.0") < rs.version_key("0.11.0") < rs.version_key("0.12.0")
    assert rs.version_key("0.11.0") < rs.version_key("0.11.1")


def test_a_prerelease_sorts_low_so_it_understates_rather_than_invents():
    assert rs.version_key("0.12.0-rc.1") < rs.version_key("0.12.0")
