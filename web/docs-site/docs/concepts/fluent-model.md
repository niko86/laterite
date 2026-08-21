# The fluent model

laterite is fluent by design: **every method returns a known type**, so chains
compose without surprises. You read a file into an `Ags4File`, then each call
either hands the same handle back (so it slots mid-chain), forks off a query, or
terminates into a frame, a relation, or bytes. Nothing in the lazy parts runs
until you ask for a result.

```python
import laterite

file = laterite.read("delivery.ags")                 # ── Ags4File
locas = file.validate().query("SELECT * FROM LOCA")  # ── self → AgsQuery (lazy)
```

## What each method returns

| Method                                    | Returns                                           | Chainable?                                             |
| ----------------------------------------- | ------------------------------------------------- | ------------------------------------------------------ |
| `read(path \| text= \| data=)`            | `Ags4File`                                        | the chain root                                         |
| `.validate(warnings=)`                    | `self` (`Ags4File`) — `Report` on `.report`       | yes, slots mid-chain                                   |
| `.fix()`                                  | a **new** `Ags4File`                              | yes (off the fixed handle)                             |
| `.at(code, ids)`                          | `AgsQuery` — **fan-out** (a borehole's group set) | yes (query branch)                                     |
| `.query(sql)`                             | `AgsQuery` — **single-result**, lazy              | yes (query branch)                                     |
| `.sql(sql)`                               | `DuckDBPyRelation`                                | terminal (materialise with `.pl()`/`.df()`/`.arrow()`) |
| `.pipe(fn, *args)`                        | whatever `fn` returns                             | yes, if `fn` returns a handle                          |
| `["LOCA"]`                                | polars frame                                      | terminal                                               |
| `.save` / `.certify` / `.text` / `.bytes` | path / `.ags.idx` / `str` / `bytes`               | terminal                                               |

The two query branches have **different terminals**: a `.query(sql)` builder
takes `.filter` / `.select` (each returning a new `AgsQuery`) and materialises
with `.frame()` / `.to_polars()` / `.to_pandas()` / `.relation()`; an `.at(...)`
fan-out exposes `.groups` and `.frames()`.

!!! warning "Single-result and fan-out are mutually exclusive"
    The single-result methods (`.filter` / `.select` / `.frame()`) and the
    fan-out methods (`.groups` / `.frames()`) live on **different `AgsQuery`
    shapes** and don't mix on one handle. And every builder call returns a
    **new, immutable** `AgsQuery` — reassign (`q = q.filter(...)`) rather than
    expecting in-place mutation, which also lets you fork a chain without one
    branch disturbing another.

## Why it composes

Because the return type of each step is fixed and documented, you can read a
chain left-to-right and know what's in your hand at every point: `read` gives an
`Ags4File`, `.validate()` keeps it, `.query()` opens a lazy plan, a terminal
cashes it in. The escape hatch — `.pipe(fn)` — never breaks the chain, since it
just threads the handle through your own function.

## See also

- [Chaining](../chaining/index.md) — the seven-rung power ladder, each rung runnable.
- [Reference cheatsheet](../reference/cheatsheet.md) — the one-screen map of every method and its return type.
