"""The docs type gate's pure halves (tools/check_doc_types.py, #565).

The tsc runs need built packages and live in the node/wasm CI jobs; what THIS
buildless module pins is the machinery those runs stand on: corpus discovery
(zero items must be impossible while the docs tree is intact), the diagnostic
parser, the leg routing, and the allowlist's shape. A gate whose discovery
silently empties is the failure mode the tool's own zero-check exists for —
these tests make sure that check has something to stand on.
"""

import importlib.util
import sys
from pathlib import Path

_TOOLS = Path(__file__).resolve().parents[1] / "tools"


def _load():
    # The house pattern for testing a tools/ script (test_check_changelog.py):
    # load by path, not by import name.
    sys.path.insert(0, str(_TOOLS))
    spec = importlib.util.spec_from_file_location(
        "check_doc_types", _TOOLS / "check_doc_types.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_doc_types"] = mod
    spec.loader.exec_module(mod)
    return mod


cdt = _load()


def test_discovery_finds_both_corpora():
    """The docs tree as it stands yields a non-empty corpus on each leg —
    pages route to node, the wasm examples to wasm — so the tool's own
    zero-items failure can only fire on genuine discovery breakage."""
    corpora = cdt.collect()
    node_keys = set(corpora["node"])
    wasm_keys = set(corpora["wasm"])
    assert any(k.startswith("page ") for k in node_keys)
    assert any(k.startswith("example node/") for k in node_keys)
    assert any(k.startswith("example wasm/") for k in wasm_keys)
    # No assertion that the wasm leg has zero page programs: today its one page
    # is include-only (covered per-file), but an inline wasm fence would be a
    # legitimate page program the tool already routes — pinning today's zero
    # would make adding one a test failure about nothing.


def test_wasm_pages_route_by_import_specifier():
    """A page importing the wasm package must never land on the node leg,
    where its specifier cannot resolve and every error would be about the
    harness rather than the page."""
    corpora = cdt.collect()
    for key, src in corpora["node"].items():
        if key.startswith("page "):
            assert cdt.WASM_SPEC not in src, key


def test_diag_re_parses_a_tsc_head_line():
    m = cdt._DIAG_RE.match(
        "cookbook__x.mjs(3,1): error TS2339: Property 'ok' does not exist."
    )
    assert m and m.group("code") == "TS2339"
    assert cdt._DIAG_RE.match("  nested detail line") is None


def test_allow_entries_carry_code_and_reason():
    """Every suppression names a TS code and says why — an entry without a
    reason is a blind spot nobody can review, and the tool prints these on
    every run."""
    for pat, code, why in cdt.ALLOW:
        assert code.startswith("TS") and code[2:].isdigit()
        assert len(why) > 20, f"({pat}, {code}) needs a real reason"


def test_controls_carry_the_518_shape():
    """The node control reads a field off a typed value (the #518 class); the
    wasm control imports a name that does not exist. If someone edits either
    into validity, the positive-control failure at run time is the backstop —
    this is the cheaper first tripwire."""
    assert "report.ok" in cdt._CONTROLS["node"]
    assert "thisExportDoesNotExist565" in cdt._CONTROLS["wasm"]
