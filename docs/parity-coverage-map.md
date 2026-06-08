# Parity coverage — laterite vs python-ags4

This page is the answer to "what does laterite actually cover, and
where does it deliberately diverge from python-ags4?". It is the
public counterpart to the internal observation catalogue in
[`OBSERVATIONS.md`](../OBSERVATIONS.md)
and the parity-decisions catalogue in
[`COMPAT.md`](../COMPAT.md).

## Headline

**122 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (93%).** The remaining 9 are deliberate
non-closures, enumerated below.

**Reproduce it yourself:** `./tools/parity-coverage.sh` clones python-ags4 1.2.0,
runs its own test suite through `laterite.compat`, and reports the parity count
**and** how much of `laterite.compat` that suite exercises (currently **76%** line
coverage — the uncovered remainder is the Rust-backed Excel I/O). It exits
non-zero if parity drops below 122.

The 122/131 count is anchored against python-ags4 **1.2.0**
specifically — the parity-pin (`PYTHON_AGS4_COMPAT`) is exact, not a
floor. A silent upstream behavioural drift would otherwise invalidate
multiple reconcile arms in the O-N catalogue without a test failure
(see [`COMPAT.md`](../COMPAT.md#identity--versioning)).

## Running the parity oracle yourself

```bash
# clone python-ags4 1.2.0 next to this repo
git clone https://gitlab.com/ags-data-format-wg/ags-python-library \
    ../ags-python-library
( cd ../ags-python-library && git checkout 1.2.0 )   # NB: the tag has no `v` prefix

# run python-ags4's own test suite shimmed to laterite.compat
./tools/run_python_ags4_tests.sh
```

The runner generates a `conftest.py` in the cloned sibling repo that
aliases `python_ags4 → laterite.compat`, so pytest collects against
the upstream tests but every behavioural assertion runs through the
Rust engine.

Expected result: **122 passed, 9 failed**. Any other count is a
parity regression (closer to 131) or new fault (further from 122) —
both worth investigating.

## The 9 deliberate non-closures

These are not bugs we plan to fix. They reflect design decisions
where laterite and python-ags4 give different signals for the same
input, and we believe laterite's signal is the more useful one. Each
links to the relevant catalogue entry.

| python-ags4 test                              | category | reason |
|-----------------------------------------------|----------|--------|
| `test_version`                                | identity | laterite reports its own version, not "1.2.0" |
| `test_rule_2`                                 | identity | `Metadata.Checker = "laterite"`, not "python-ags4" |
| `test_rule_2b_1`                              | identity | same as above |
| `test_rule_LBSGCheck`                         | identity | same |
| `test_rule_STNDandPREMCheck`                  | identity | same |
| `test_rule_AGS3`                              | O-30     | we refuse AGS3 input rather than mis-validate as AGS4 |
| `test_rule_6_1`                               | O-2 / O-34 | we refuse non-CSV input as `NotAgs4Error` |
| `test_checking_without_dictionary_raises_error` | H-1    | we raise typed `MissingDictionaryError`; python-ags4 wraps |
| `test_duplicate_groups_raises_error`          | H-1      | same |

Both `H-1` items are an error-shape choice. python-ags4 swallows the
failure into a generic report; we surface a typed exception that
callers can match on. Wrapping it back would lose information for
every caller that benefits from the typed shape.

## Coverage by python-ags4 test module

| python-ags4 module | tests | laterite passes | covered by |
|---|---|---|---|
| `test_ags4.py` (parser, AGS4_to_dataframe, write_AGS4_file, …) | 30 | 30 | `packages/laterite/tests/test_laterite.py` (compat surface) |
| `test_check.py::test_rule_*` (Rules 1–20) | ~85 | 82 | `packages/laterite/tests/test_laterite.py` + Rust crate tests under `rust-packages/ags4-validator/src/rules/` |
| `test_check.py::test_AGS4_check_file*` | 9 | 8 | `packages/laterite/tests/test_laterite.py` (end-to-end check) |
| `test_main.py` (CLI shim) | 4 | 4 | `laterite._cli` + `packages/laterite/tests/test_laterite.py` |

The Rust-side rule modules carry the real behavioural assertions for
each AGS4 rule, with corner-case fixtures (BOM, CR-only line endings,
SF expected-suffix, FILE Rule 20 …) checked at the binary level. The
Python-side `test_laterite.py` re-asserts the same behaviour through
the `laterite.compat` surface so the Python and Rust faces stay
agreement.

## In-repo synthetic tests beyond python-ags4

These exercise behaviour python-ags4 does not test — either because
it's a laterite-specific surface (`.ags5db`, typed PROJ graph,
transport), or because it pins a regression we've fixed (FYI
surfacing, dictionary parity, recipe execution …).

| Module | Tests | Coverage |
|---|---|---|
| `packages/laterite/tests/test_laterite.py` | 34 | python-ags4 parity surface + compat-specific (sort_groups, set_backend, FYI surfacing, JSON helpers) |
| `packages/laterite/tests/test_registry.py` | 8 | Dictionary-as-data: `laterite.registry.GROUPS`, hierarchy walk, heading metadata |
| `packages/laterite/tests/test_transport.py` | 7 | `transport.{pack,unpack,lock,unlock}` — zstd + age envelope; wrong-passphrase failure mode |
| `packages/laterite/tests/test_ags4.py` | 1 | 23 MB / 69-group real-world round-trip (skips if fixture absent) |
| `packages/laterite/tests/test_ags4_typed.py` | 3 | `laterite.ags4.read_typed` → typed PROJ tree |
| `packages/laterite/tests/test_review_regressions.py` | 2 | Pinned-fix regressions (e.g. older-`.ags5db` view tolerance — kept as a smoke test) |
| `packages/laterite-ags5/tests/test_ags5db.py` | 31 | The full `.ags5db` Python surface: convert / export / count / sum / sql / peek / query / info / groups / headings / inspect / validate / diff / Predicate |
| `packages/laterite-ags5/tests/test_typed_graph.py` | 27 | Round-trip the 92 typed `#[pyclass]` groups + dynamic passthrough groups + heading dedup |
| `packages/laterite-ags5/tests/test_db.py` | 14 | DDL invariants (UUID7 PKs, parent-id JOINs, KEY-overlap dedup, content-hash dedup) |
| `packages/laterite-ags5/tests/test_blobs.py` | 8 | Photo / file attachment round-trip, SHA-256 persistence, `list_blobs` filters |
| `packages/laterite-ags5/tests/test_merge.py` | 5 | Append-merge idempotency, KEY-only overlap dedup, real-file scale |

Total ~210 in-repo tests (counts the `tests/` collected for `uv run
pytest`); plus the 131 upstream tests when the parity oracle is run.

## See also

- [`COMPAT.md`](../COMPAT.md) — the user-facing parity catalogue, with rule-by-rule semantic differences
- [`OBSERVATIONS.md`](../OBSERVATIONS.md) — the engineering record of every observation O-1..O-N (5-field house style)
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) — clean-room policy + how to extend the parity oracle
