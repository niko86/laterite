## What changed

<!-- One or two sentences. Lead with the "why". -->

## Checks (run locally first — CI runs the same)

<!-- The fast gates. Run them before pushing and save a round trip. -->

- [ ] `uv run ruff check .`
- [ ] `uv run ty check`
- [ ] `cd rust-packages && cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --exclude laterite-ags4-wasm --exclude laterite-ags4-tokenizer-wasm -- -D warnings`
- [ ] `uv run pytest tests/ packages/laterite/tests -q`
- [ ] `cargo test --workspace`

## Test plan

<!-- How you verified this. Tick all that apply. -->

- [ ] New behaviour covered by a test
- [ ] `./tools/run_python_ags4_tests.sh` parity count unchanged (or improved)
- [ ] Validator behaviour change → `COMPAT.md` + `docs/parity-coverage-map.md` updated
- [ ] User-visible API change → entry added to `changelog.json`, then
      `uv run --no-sync python tools/gen_changelog.py`
- [ ] Validator behaviour change → O-N added to `observations.json`, then
      `uv run --no-sync python tools/gen_observations.py` (plus its
      `ags-wiki/observations/O-NN.md` page)

<!-- CHANGELOG.md and OBSERVATIONS.md are GENERATED views of the two JSON files
     above, and `--check` gates in ci.yml/nightly.yml fail on a hand-edit. Edit
     the JSON and regenerate; never the Markdown. -->

- [ ] Wiki updated — ags-wiki reflects any behaviour/architecture/build change (`lint.py --since` passes)

## Clean-room confirmation

For changes to the validator engine: confirm any rule semantics were
written from the AGS spec, not copied from python-ags4 source. See
`CONTRIBUTING.md` for the clean-room policy.

- [ ] Not applicable (docs / CI / packaging / tests)
- [ ] Clean-room confirmed
