# Contributing

Thanks for your interest. This page covers setup, the test commands,
and a couple of project rules.

## Setup

You need Python **3.14**, a stable Rust toolchain, and
[`uv`](https://github.com/astral-sh/uv).

```bash
git clone https://github.com/niko86/laterite.git
cd laterite
uv sync                                  # installs both wheels + dev deps
```

After a Rust change, rebuild the affected wheel:

```bash
cd packages/laterite          && uv run --no-sync maturin develop --release --uv
cd packages/laterite-ags5     && uv run --no-sync maturin develop --release --uv
```

## Tests

```bash
uv run pytest packages/laterite/tests -q
uv run pytest packages/laterite-ags5/tests -q
cargo test --manifest-path rust-packages/Cargo.toml --workspace --release
```

If you have python-ags4 cloned alongside (`../ags-python-library`)
you can also run the parity oracle:

```bash
./tools/run_python_ags4_tests.sh
```

## Pull requests

Fork, branch (`feat/...`, `fix/...`, `docs/...`), open a PR against
`main`. CI runs `cargo test`, `cargo clippy`, `cargo fmt --check`,
and `pytest`. PRs need a green build to merge.

If a change affects validator behaviour, update
[`OBSERVATIONS.md`](OBSERVATIONS.md) (and where relevant
[`COMPAT.md`](COMPAT.md) and
[`docs/parity-coverage-map.md`](docs/parity-coverage-map.md)) in the
same PR.

## Clean-room policy

The validator engine is written from the AGS4 spec, not from
python-ags4 source. python-ags4 is LGPL-3.0; copying any of its
implementation into laterite would force-license laterite under
LGPL too and kill the MIT distribution path. If you're contributing
a rule fix, look at the spec, the parity oracle output, and
laterite's own tests — not at python-ags4's source.

## Reporting bugs

Open an issue with the laterite version
(`python -c "import laterite; print(laterite.__version__)"`), a
minimal AGS file or snippet, and what you expected vs. what you got.

For parity regressions vs. python-ags4, include the python-ags4
version you compared against (we pin to 1.2.0).
