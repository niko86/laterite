-- One-time setup in any DuckDB session (CLI, Python, Node, wasm ≥ 1.5.4).
-- Include-only for the docs: the test gate loads a locally built extension.
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
