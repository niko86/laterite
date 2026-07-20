# tools

Developer-facing scripts.

| Script | Purpose |
|---|---|
| `build-rust.sh` / `build-rust.ps1` | Build the Rust `lat` + `ags5db` binaries into `dist/`. |
| `generate_pyi.py` | Regenerate `packages/laterite/python/laterite/_laterite_native.pyi` from the AGS dictionary. CI guards it via `test_pyi_stubs_match_generator.py`. |
| `run_python_ags4_tests.sh` | Run python-ags4 1.2.0's own test suite shimmed to `laterite.compat` (parity oracle). Needs `../ags-python-library/` cloned. |
| `parity-coverage.sh` | Clone python-ags4 1.2.0 + run its suite through `laterite.compat` with coverage — reports the 121/131 parity AND how much of `laterite.compat` it exercises. Verify the README's parity claim yourself. |
