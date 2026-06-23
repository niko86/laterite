## What changed

<!-- One or two sentences. Lead with the "why". -->

## Test plan

<!-- How you verified this. Tick all that apply. -->

- [ ] `cargo test --workspace --release` passes
- [ ] `uv run pytest tests/ packages/laterite/tests -q` passes
- [ ] `./tools/run_python_ags4_tests.sh` parity count unchanged (or improved)
- [ ] New behaviour covered by a test
- [ ] Validator behaviour change → `OBSERVATIONS.md` updated
- [ ] User-visible API change → `CHANGELOG.md` updated

## Clean-room confirmation

For changes to the validator engine: confirm any rule semantics were
written from the AGS spec, not copied from python-ags4 source. See
`CONTRIBUTING.md` for the clean-room policy.

- [ ] Not applicable (docs / CI / packaging / tests)
- [ ] Clean-room confirmed
