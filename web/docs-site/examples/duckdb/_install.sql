-- One-time setup in any DuckDB session — CLI, Python, Node or wasm — on DuckDB
-- 1.5.4 or newer. That floor is not ours and is not wasm-specific: the community
-- repository builds an extension once per DuckDB release starting from the one it
-- was accepted for, so nothing earlier resolves on any host.
--
-- Include-only: no page includes this file, and two gates run the examples beside
-- it, each against a DIFFERENT artifact. `tests/test_docs_duckdb_examples.py` is
-- the one in this repo — nightly, against exactly the INSTALL below. Its
-- local-build twin runs monthly in the dev satellite. "Our build works" and "the
-- published one works" are separate claims and both are now measured.
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
