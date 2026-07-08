# One engine, many doors

There is exactly **one** AGS4 engine — a Rust parser + validator — and every
way you reach it is a door onto that same engine. Pick the door that fits where
you are; the behaviour behind it is identical.

## One engine

The parser that tokenises an AGS4 file, the dictionary that types each column,
and the numbered-rules validator are all Rust. Nothing reimplements them per
language. So a file that reads clean in Python reads clean at the CLI, in Node,
and inside DuckDB — same dtypes, same findings, same rule numbers. There is no
"Python parser" drifting from a "JavaScript parser"; there is one engine with
several front doors:

- **Python** — `import laterite` (this site).
- **`lat` CLI** — `lat validate delivery.ags --json` (see the [CLI reference](../reference/cli.md)).
- **Node** — `@laterite/*` on npm.
- **DuckDB** — the `laterite_ags4` loadable extension.

## Many input doors

Inside Python, `read` itself has three doors — and they all feed the same
engine, so it doesn't matter which one your data arrives through:

```python
import laterite

# A minimal AGS4 string — GROUP / HEADING / UNIT / TYPE rows, then DATA.
ags4_text = '''"GROUP","LOCA"
"HEADING","LOCA_ID","LOCA_GL"
"UNIT","","m"
"TYPE","ID","2DP"
"DATA","BH01","23.68"
"DATA","BH02","32.49"
'''

ags = laterite.read(text=ags4_text)   # the text= door
loca = ags["LOCA"]
print(loca)
print({h: str(loca[h].dtype) for h in ("LOCA_ID", "LOCA_GL")})
```

```text
shape: (2, 2)
┌─────────┬─────────┐
│ LOCA_ID ┆ LOCA_GL │
│ ---     ┆ ---     │
│ str     ┆ f64     │
╞═════════╪═════════╡
│ BH01    ┆ 23.68   │
│ BH02    ┆ 32.49   │
└─────────┴─────────┘
{'LOCA_ID': 'String', 'LOCA_GL': 'Float64'}
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
