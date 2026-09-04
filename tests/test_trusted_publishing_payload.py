"""The crates.io create payload must use the WIRE names, not the Rust ones.

crates.io's `NewGitHubConfig` spells the crate field `pub krate: String` — because
`crate` is reserved in Rust — and carries `#[serde(rename = "crate")]` to put the
real name back on the JSON. Reading the handler's field name and not its rename
sent `krate`, and the registry answered:

    HTTP 422 — Failed to deserialize the JSON body into the target type:
               github_config: missing field `crate`

Cheap to pin and worth pinning, because the feedback is remote, needs a token,
and only appears at the moment you are trying to set up publishing. The other
four names are asserted alongside it: they are not renamed today, and this is
where it would show if that changed.

`workflow_filename` and `environment` are the two that also have to agree with a
live workflow file, so those are checked against the tree rather than against a
literal — a rename on either side breaks the publish for every crate at once,
at publish time, on the registry that cannot be un-published.
"""

from __future__ import annotations

import importlib.util
import sys
import urllib.error
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
TOOL = REPO / "tools" / "release" / "trusted_publishing.py"


def _load():
    spec = importlib.util.spec_from_file_location("trusted_publishing", TOOL)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["trusted_publishing"] = mod
    spec.loader.exec_module(mod)
    return mod


tp = _load()


def test_the_crate_field_goes_over_the_wire_as_crate() -> None:
    cfg = tp.body_for("laterite-ags4-parse")["github_config"]
    assert "crate" in cfg, (
        "crates.io renames this field; sending the Rust name `krate` is a 422 "
        "on the first create"
    )
    assert "krate" not in cfg
    assert cfg["crate"] == "laterite-ags4-parse"


def test_the_other_four_names_are_the_ones_crates_io_reads() -> None:
    assert set(tp.body_for("x")["github_config"]) == {
        "crate",
        "repository_owner",
        "repository_name",
        "workflow_filename",
        "environment",
    }


def test_the_payload_names_a_workflow_that_exists() -> None:
    """The half of the contract that lives in this repo rather than on crates.io."""
    cfg = tp.body_for("x")["github_config"]
    workflow = REPO / ".github" / "workflows" / cfg["workflow_filename"]
    assert workflow.is_file(), (
        f"{cfg['workflow_filename']} is named in every crates.io config and does "
        "not exist here — the publish would fail at the registry"
    )
    assert f"environment: {cfg['environment']}" in workflow.read_text(
        encoding="utf-8"
    ), "the workflow does not use the environment the configs pin it to"


def test_a_crate_the_registry_does_not_know_is_partitioned_not_fatal(
    monkeypatch,
) -> None:
    """The config API 404s for a name with no release — confirmed live when
    `laterite-ags4-excel` was being armed (2026-09-04), where it aborted the
    whole run. That is a state of publish prep, not an API failure: the config
    can only exist after the crate's first publish, so the listing must name
    the crate and carry on for the ones that do exist."""

    def fake_call(method, url, token, body=None):
        if url.endswith("crate=laterite-ags4-excel"):
            raise urllib.error.HTTPError(url, 404, "Not Found", None, None)
        return {"github_configs": []}

    monkeypatch.setattr(tp, "call", fake_call)
    have, unpublished = tp.existing("token")
    assert unpublished == ["laterite-ags4-excel"]
    assert "laterite-ags4-excel" not in have
    assert set(have) | set(unpublished) == set(tp.crates())


def test_other_http_errors_still_abort(monkeypatch) -> None:
    """Only the 404 is a fact about the crate. A 403 is a credential fault (a
    publish-scoped token got exactly that, live) — treating it like the 404
    would print a wall of NO CRATE rows over a bad token."""

    def fake_call(method, url, token, body=None):
        raise urllib.error.HTTPError(url, 403, "Forbidden", None, None)

    monkeypatch.setattr(tp, "call", fake_call)
    with pytest.raises(urllib.error.HTTPError):
        tp.existing("token")


def test_the_crate_list_is_derived_and_real() -> None:
    """`crates()` reads `publish_crates.PUBLISH_SET` rather than restating it, so
    a new publishable crate gets a config by itself instead of being the one
    nobody created. Asserted through the tool — importing `publish_crates` HERE
    would trip the buildless-job marker gate, and would only compare the
    derivation against itself anyway. Checking the names resolve to manifests is
    the stronger claim."""
    names = tp.crates()
    assert names, "no publishable crates — the derivation is broken, not empty"
    assert names == sorted(names)
    for name in names:
        assert (REPO / "rust-packages" / name / "Cargo.toml").is_file(), (
            f"{name} would get a crates.io config and has no manifest here"
        )
