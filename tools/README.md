# tools

Developer-facing scripts.

| Script | Purpose |
|---|---|
| `build-rust.sh` / `build-rust.ps1` | Build the Rust `ags4-check` + `ags5db` binaries into `dist/`. |
| `generate_pyi.py` | Regenerate `packages/laterite/python/laterite/_laterite_native.pyi` from the AGS dictionary. CI guards it via `test_pyi_stubs_match_generator.py`. |
| `run_python_ags4_tests.sh` | Run python-ags4 1.2.0's own test suite shimmed to `laterite.compat` (parity oracle). Needs `../ags-python-library/` cloned. |
