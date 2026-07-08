"""Value-domain gate: the error-kind → exit-code table is single-sourced in the
Rust producer ``ValidatorError::kind()/exit_code()``, and severity in
``Severity::as_str()``. Every surface delegates. This gate keeps the *downstream*
consumers (Python ``_errors.py``, the Rust bindings) honest against the canonical
table and pins that no surface carries a local kind/code/severity literal that
could drift — modelled on ``test_typed_choices.py`` + the ``allowlist-is-live``
idiom.
"""

from __future__ import annotations

from pathlib import Path

import laterite as L
import pytest
from laterite import _errors

_ROOT = Path(__file__).resolve().parents[3]  # …/laterite


def _src(*parts: str) -> str:
    return _ROOT.joinpath(*parts).read_text(encoding="utf-8")


# The canonical PRODUCER table — the 5 `ValidatorError` variants. The surfaces
# add two consumer-only kinds no Rust producer emits: `not_utf8` (the validator
# decodes lossily, so it never surfaces) and `bad_args` (the arg/dispatch layer).
_PRODUCER = {"not_found": 3, "io": 3, "not_ags4": 4, "unsupported_edition": 4, "bad_dict": 5}
_CONSUMER_ONLY = {"not_utf8": 4, "bad_args": 5}
_ALL = {**_PRODUCER, **_CONSUMER_ONLY}


def test_python_kind_table_matches_canonical():
    """`_errors.py`'s kind → exception → exit_code equals the canonical table.
    `not_found`/`io` are routed inline by `raise_for` to FileNotFoundError (3)."""
    got = {"not_found": 3, "io": 3}
    for kind, exc in _errors._KIND_TO_EXC.items():
        got[kind] = exc.exit_code
    assert got == _ALL


def test_python_error_probes_hit_the_right_exit_codes():
    """Behavioural probes — reach the runtime sniff `inspect` can't see."""
    with pytest.raises(FileNotFoundError):
        L.read("/no/such/file.ags").validate()
    with pytest.raises(_errors.Ags4Error) as bad_dict:
        L.validate(text='"GROUP","PROJ"\r\n', dict_version="9.9")
    assert bad_dict.value.exit_code == 5  # bad_dict, rejected before parse
    with pytest.raises(_errors.Ags4Error) as not_ags4:
        L.validate(text="this is not an ags4 file at all\r\n")
    assert not_ags4.value.exit_code == 4  # not_ags4 / unsupported


def test_the_error_domain_lives_only_in_the_validator_producer():
    """The `(variant → kind/code)` literal table exists ONLY in the validator's
    `error.rs`; the Node/Python Rust bindings delegate via `.kind()`/`.exit_code()`
    (the deleted local tuples must not reappear)."""
    err = _src("rust-packages", "laterite-ags4-validator", "src", "error.rs")
    assert 'ValidatorError::NotFound(_) => "not_found"' in err  # the one producer
    for surface in ("laterite-node", "laterite-py"):
        s = _src("rust-packages", surface, "src", "lib.rs")
        assert ".exit_code()" in s and ".kind()" in s, f"{surface} must delegate"
        assert '=> (3, "not_found")' not in s, f"{surface} still has a local table"


def test_wasm_io_collapse_is_an_allowlisted_live_divergence():
    """wasm deliberately collapses NotFound/Io → "io" (no filesystem). Keep the
    allowlist LIVE: the collapse must still be in wasm source AND the producer
    must still say "not_found" for NotFound — else the divergence is gone and this
    allowlist should shrink."""
    wasm = _src("rust-packages", "laterite-ags4-wasm", "src", "lib.rs")
    err = _src("rust-packages", "laterite-ags4-validator", "src", "error.rs")
    assert 'ValidatorError::NotFound(_) | ValidatorError::Io { .. } => "io"' in wasm
    assert 'ValidatorError::NotFound(_) => "not_found"' in err
    val_err = wasm.split("struct ValErr")[1].split("}")[0]
    assert "exit_code" not in val_err, "wasm ValErr carries no exit code (browser has none)"


def test_severity_is_single_sourced_not_debug_derived():
    """Node/wasm stop deriving the severity token from `format!("{:?}")`; the
    emitted tokens are the three canonical ones."""
    for surface in ("laterite-node", "laterite-ags4-wasm"):
        s = _src("rust-packages", surface, "src", "lib.rs")
        assert 'format!("{s:?}").to_lowercase()' not in s, f"{surface} still Debug-derives severity"
        assert ".as_str()" in s
    dirty = (
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_CUSTOM"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\n"DATA","BH1","x"\r\n'
    )
    rep = L.validate(text=dirty, fyi=True)
    sevs = {f.get("severity", "error") for items in rep.by_rule().values() for f in items}
    assert sevs, "expected some findings"
    assert sevs <= {"error", "warning", "fyi"}
