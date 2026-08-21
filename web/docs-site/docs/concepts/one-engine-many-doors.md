# One engine, many doors

There is exactly **one** AGS4 engine — a Rust parser + validator — and every
way you reach it is a door onto that same engine. Pick the door that fits where
you are; the behaviour behind it is identical.

## One engine

The parser that tokenises an AGS4 file, the dictionary that types each column,
and the numbered-rules validator are all Rust. Nothing reimplements them per
language. So a file that reads clean in Python reads clean at the CLI, in Node,
and in the browser — same dtypes, same findings, same rule numbers. There is no
"Python parser" drifting from a "JavaScript parser"; there is one engine with
several front doors:

- **Python** — `import laterite` (this site).
- **`lat` CLI** — `lat validate delivery.ags --json` (see the [CLI reference](../reference/cli.md)).
- **Node** — `laterite` on npm.
- **The browser** — `@laterite/ags4-wasm`, the same engine compiled to wasm (see
  the [wasm reference](../reference/wasm-api.md)).
- **DuckDB** — the `laterite_ags4` loadable extension. A **read-only** door: it
  reads and types a file but runs no rules, so it is the one door that does not
  carry the guarantee above (see
  [Cross-surface parity](./cross-surface-parity.md)).

## Many input doors

Inside Python, `read` itself has three doors — and they all feed the same
engine, so it doesn't matter which one your data arrives through:

```python
--8<-- "python/ex19_read_text_door.py:code"
```

```text
--8<-- "python/ex19_read_text_door.out"
```

The same `2DP` → `Float64` typing you get from a file on disk falls out of an
in-memory string, because the door is just a way in — the engine behind it is
the same. The three doors are:

- `read("delivery.ags")` — a **path** on disk.
- `read(text=...)` — an in-memory **AGS4 string**.
- `read(data=raw_bytes)` — raw **bytes** (an upload, an HTTP body, a blob).

All three return the same object, so the rest of your code never asks where the
data came from.

!!! note "Why it matters"
    Identical behaviour across surfaces is a guarantee, not a coincidence:
    validate in CI with `lat`, then read the same file in Python and get
    the same verdict. And because the input doors converge, a web upload
    (`data=`), a pasted snippet (`text=`), and a file (path) all flow through
    one code path — no special-casing.

← Back to [Read](../learn/read.md)
