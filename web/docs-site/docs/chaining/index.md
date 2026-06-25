# Chaining

laterite reads like a sentence. You `read` a file, then keep `.validate()`-ing
and `.query()`-ing and `.at()`-ing off the same handle — and nothing executes
until you ask for a frame. The handle stays the handle; the lazy parts stay lazy.

```text
laterite.read(path|text=|data=) ──► Ags4File ─────────────────────────────────┐
                                       │                                       │
   .validate(warnings=)  ──► self (Ags4File)   …report on .report             │
   .at(code, ids)        ──► AgsQuery  (fan-out: one borehole's group set)     │
   .query(sql)           ──► AgsQuery  (lazy single-result builder)            │
   .sql(sql)             ──► DuckDBPyRelation     ◄── terminal                 │
   .pipe(fn, *args)      ──► whatever fn returns                               │
   ["LOCA"]              ──► polars frame          ◄── terminal               │
   .save / .certify / .text / .bytes               ◄── terminals             │
                                                                              │
AgsQuery ── single-result branch ──────────────────────────────────────────────┤
   .filter(expr)  ──► AgsQuery (new)   .select(*cols) ──► AgsQuery (new)        │
       └─ terminals: .frame() · .to_polars() · .to_pandas() · .relation()      │
                                                                              │
AgsQuery ── fan-out branch (from .at) ──────────────────────────────────────────┘
   .groups ──► list[str]            .frames() ──► dict[code, polars frame]
```

Every builder call returns a **new** `AgsQuery` — the handles are immutable, so
you can fork a chain without one branch mutating another.

## The power ladder

Seven rungs, each runnable. They climb from a one-line self-chain to raw SQL,
your own functions, and the certify fast-path.

### 1 · Minimal self-chain

`.validate()` returns the `Ags4File` itself, so it slots mid-chain; the
`Report` rides along on `.report`.

```python
--8<-- "python/ex02_validate.py"
```

```text
is_valid=True count=0 dict_version='4.1.1' resolution='exact'
```

### 2 · Fan-out a branch

`.at(code, ids)` pivots the handle to one borehole's related group set and hands
back an `AgsQuery`. Ask it `.groups` to see what came along for the ride.

```python
--8<-- "python/ex03_at_fanout_groups.py"
```

```text
['LOCA', 'SAMP', 'LLPL']
```

### 3 · Terminate the fan-out

`.frames()` materialises that record set as a dict of born-typed polars frames,
keyed by group code.

```python
--8<-- "python/ex04_at_frames.py"
```

```text
['LLPL', 'LOCA', 'SAMP']
4
```

### 4 · The lazy builder and its four terminals

`.query(sql)` opens a single-result `AgsQuery`. Stack `.filter` / `.select` —
nothing runs — then pick a terminal to materialise the plan.

```python
--8<-- "python/ex05_query_builder.py"
```

```text
shape: (7, 3)
┌─────────┬───────────┬─────────┐
│ LOCA_ID ┆ LOCA_TYPE ┆ LOCA_GL │
│ ---     ┆ ---       ┆ ---     │
│ str     ┆ str       ┆ f64     │
╞═════════╪═══════════╪═════════╡
│ BH02    ┆ RC        ┆ 32.49   │
│ BH03    ┆ RC        ┆ 28.54   │
│ BH04    ┆ RC        ┆ 29.04   │
│ BH05    ┆ RC        ┆ 31.62   │
│ BH07    ┆ RC        ┆ 31.33   │
│ BH08    ┆ CP        ┆ 28.67   │
│ BH09    ┆ CP        ┆ 30.98   │
└─────────┴───────────┴─────────┘
```

`.frame()` follows the handle's backend, `.to_polars()` / `.to_pandas()` force
one, and `.relation()` hands back the still-lazy `DuckDBPyRelation`.

### 5 · Raw SQL mid-chain

When the builder is too narrow, drop to `.sql(sql)` for a full join across
groups. It returns a `DuckDBPyRelation` — a terminal you materialise with `.pl()`,
`.df()`, or `.arrow()`.

```python
--8<-- "python/ex06_sql_join.py"
```

```text
shape: (14, 2)
┌─────────┬─────┐
│ LOCA_ID ┆ n   │
│ ---     ┆ --- │
│ str     ┆ i64 │
╞═════════╪═════╡
│ BH01    ┆ 4   │
│ BH02    ┆ 2   │
│ BH03    ┆ 3   │
│ BH04    ┆ 4   │
│ BH05    ┆ 2   │
│ …       ┆ …   │
│ BH10    ┆ 3   │
│ BH11    ┆ 3   │
│ BH12    ┆ 3   │
│ BH13    ┆ 4   │
│ BH14    ┆ 4   │
└─────────┴─────┘
```

### 6 · Splice in your own step with `.pipe`

`.pipe(fn, *args)` passes the handle as `fn`'s first argument and returns
whatever `fn` returns — so an escape hatch never breaks the chain. It works on
both `Ags4File` and `AgsQuery`.

```python
--8<-- "python/ex07_pipe.py"
```

```text
first 3 group codes: ['PROJ', 'TRAN', 'UNIT']
LOCA row count via pipe: 14
```

### 7 · The certify fast-path

A clean `.validate()` can mint a `.ags.idx` certificate via `.certify()`.
Re-read with that fresh cert and `.validate()` resolves without ever running the
rule engine — `resolution` reads `certified`, not `exact`.

```python
--8<-- "python/ex08_certify.py"
```

```text
certified
```

!!! warning "Single-result and fan-out don't mix on one `AgsQuery`"
    The single-result terminals (`.filter` / `.select` → `.frame()` /
    `.to_polars()` / `.to_pandas()` / `.relation()`) and the fan-out terminals
    (`.frames()` / `.groups`) belong to different `AgsQuery` shapes — a
    `.query(sql)` builder versus an `.at(...)` fan-out — and are mutually
    exclusive on a given handle. And because every builder call returns a **new**
    immutable `AgsQuery`, reassign (`q = q.filter(...)`) rather than expecting
    in-place mutation.

## Where next

- The [Cookbook](../cookbook/index.md) chains these rungs into end-to-end
  recipes — join, filter, certify, and write back out.
- The [Reference cheatsheet](../reference/cheatsheet.md) is the one-screen map of
  every method on the chain and what it returns.
