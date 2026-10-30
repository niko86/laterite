"""lint.py's A15: a `repo:…::symbol` citation must name something the file DEFINES.

The word check it replaces for Rust and Python passed `emit.rs::synthesise_metadata`
for months after the function became `SynthAccumulator`, because the name lived
on as a struct field and in comments. So the regression case is pinned first,
and every arm of the definition patterns has its own positive and negative.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import pytest
from _tools import load_tool


@pytest.fixture(scope="module")
def refs():
    return load_tool("refs")


def _src(tmp_path, name: str, text: str):
    p = tmp_path / name
    p.write_text(text, encoding="utf-8")
    return p


def test_a_renamed_function_still_mentioned_is_not_defined(refs, tmp_path):
    """The #1169 case: field, struct literal and comment all name it; nothing
    defines it."""
    p = _src(
        tmp_path,
        "emit.rs",
        "pub struct EmitOpts {\n    pub synthesise_metadata: bool,\n}\n"
        "// the batch synthesise_metadata became the accumulator\n"
        "fn build() -> EmitOpts { EmitOpts { synthesise_metadata: true } }\n"
        "pub struct SynthAccumulator;\n",
    )
    assert refs.symbol_defined(p, "synthesise_metadata") is False
    assert refs.symbol_defined(p, "SynthAccumulator") is True


@pytest.mark.parametrize(
    "line",
    [
        "pub fn run_all() {}",
        "pub(crate) async fn run_all() {}",
        "struct run_all;",
        "pub enum run_all {}",
        "trait run_all {}",
        "type run_all = u8;",
        "const run_all: u8 = 1;",
        "static run_all: u8 = 1;",
        "mod run_all;",
        "union run_all { a: u8 }",
        "macro_rules! run_all { () => {} }",
    ],
)
def test_rust_definitions(refs, tmp_path, line):
    assert refs.symbol_defined(_src(tmp_path, "a.rs", line + "\n"), "run_all") is True


def test_rust_prefix_of_a_longer_name_is_not_its_definition(refs, tmp_path):
    p = _src(tmp_path, "a.rs", "pub fn run_all_groups() {}\n")
    assert refs.symbol_defined(p, "run_all") is False


@pytest.mark.parametrize(
    "line",
    [
        "def check(x):",
        "async def check(x):",
        "class check:",
        "    def check(self):",
        "check = 1",
        "check: int = 1",
    ],
)
def test_python_definitions(refs, tmp_path, line):
    assert refs.symbol_defined(_src(tmp_path, "a.py", line + "\n"), "check") is True


@pytest.mark.parametrize(
    "line",
    [
        "    check = 1",  # a local, not what a citation names
        "    run(check=True)",  # a keyword argument
        "check == 1",  # a comparison
        "# check is defined elsewhere",
    ],
)
def test_python_non_definitions(refs, tmp_path, line):
    assert refs.symbol_defined(_src(tmp_path, "a.py", line + "\n"), "check") is False


def test_an_uncovered_language_is_not_judged(refs, tmp_path):
    """None, not False: lint.py leaves these to the report-only word check and
    counts them, rather than failing a language it has no pattern for."""
    p = _src(tmp_path, "a.ts", "export function check() {}\n")
    assert refs.symbol_defined(p, "check") is None


@pytest.mark.parametrize(
    ("raw", "want"),
    [
        ("rust-packages/x/src/emit.rs::SynthAccumulator", "SynthAccumulator"),
        ("rust-packages/x/src/emit.rs::SynthAccumulator).", "SynthAccumulator"),
        ('tools/a.py::_DEFAULT"', "_DEFAULT"),
        ("rust-packages/x/src/emit.rs:120", None),
        ("OBSERVATIONS.md#o-35", None),
    ],
)
def test_cited_symbol(refs, raw, want):
    assert refs.cited_symbol(raw) == want
