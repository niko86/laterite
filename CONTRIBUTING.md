# Contributing

laterite is an AGS4 toolchain with a Rust core and several surfaces built on top
of it: a Python wheel, a Node binding, a wasm build, and the `lat` CLI. The AGS
group definitions live in one JSON file and nearly everything else is generated
from it, so a lot of changes are a one-file edit plus a rebuild. This page is how
to get set up, what the CI will hold you to, and two rules to read before you send
a PR.

## Setup

You need a stable Rust toolchain, [`uv`](https://github.com/astral-sh/uv), and
Python 3.14 for the workspace. (The published wheel is abi3 and installs on 3.12+;
the dev floor is higher.)

```bash
git clone https://github.com/niko86/laterite.git
cd laterite
uv sync        # laterite + dev deps into a managed venv
```

`uv sync` installs the package but does not compile the Rust extension. After any
Rust change, rebuild the wheel in place:

```bash
cd packages/laterite && uv run --no-sync maturin develop --release --uv
```

## Tests

```bash
uv run pytest tests/ -q                    # faithfulness gates (see below)
uv run pytest packages/laterite/tests -q   # the wheel's own tests
cd rust-packages && cargo test --workspace
```

The root `tests/` suite is the one people forget. It re-runs every generator and
checks that the committed output still matches — the dictionary projection, the
type stubs, the docs examples, the observations prose, and so on. Run it whenever
you edit a source-of-truth file or a generator, or it will fail on someone else's
PR instead of yours.

If you have python-ags4 checked out alongside at `../ags-python-library`, the
parity oracle runs laterite against its fixtures:

```bash
./tools/run_python_ags4_tests.sh
```

## Editing the AGS dictionary

`rust-packages/laterite-ags4-reference/data/ags_dictionary.json` is the single
source for all 174 AGS groups. The typed Python classes, the validator's per-edition
tables, the wasm dictionary, and the `.pyi` type stubs are all projected from it.
Change it and you have to rebuild and regenerate the stubs:

```bash
cd packages/laterite && uv run --no-sync maturin develop --release --uv
cd - && uv run python tools/generate_pyi.py
```

Adding a group is that edit plus the rebuild. The drift tests in the root suite
fail loudly if a generated file gets out of step, so you will not miss it silently.

## Pull requests

Branch off `main` (`feat/…`, `fix/…`, `docs/…`) and open a PR against `main`; it
needs a green build to merge. CI runs the checks below, so run them first and save
a round trip:

```bash
uv run ruff check .
uv run ty check
uv run pytest tests/ packages/laterite/tests -q
cd rust-packages
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --exclude laterite-ags4-wasm --exclude laterite-ags4-tokenizer-wasm -- -D warnings
```

Keep the diff to what the change actually needs. Small, targeted PRs get reviewed
and merged; a fix bundled with unrelated reformatting or "while I was here" cleanup
does not. If you spot something else worth doing, raise it in an issue or mention
it in the PR rather than folding it in.

When a change alters what the validator reports, say so in the PR description and
update [`COMPAT.md`](COMPAT.md) in the same change — and
[`docs/parity-coverage-map.md`](docs/parity-coverage-map.md) if the parity count
moves.

## Clean-room policy

The validator is written from the AGS4 specification, not from python-ags4's
source. python-ags4 is LGPL-3.0, and copying any of its implementation into
laterite would pull the whole project under LGPL and end the MIT wheel. When you
work on a rule, read the spec, the parity oracle's output, and laterite's own
tests. Do not read python-ags4's source for the answer.

## Reporting bugs

Open an issue with the version you are on:

```bash
python -c "import laterite; print(laterite.__version__)"
```

Attach a minimal AGS file or snippet, and say what you expected against what you
got. For a parity regression against python-ags4, include the python-ags4 version
you compared with — the suite pins 1.2.0.
