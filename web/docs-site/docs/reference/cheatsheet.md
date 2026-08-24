# Python cheatsheet

The whole Python surface is three handles plus two result objects. Read one
file, then chain. Every method below hangs off what `read()` gives you.

```python
--8<-- "python/ex05_query_builder.py:code"
```

```text
--8<-- "python/ex05_query_builder.out"
```

A lazy `AgsQuery` (above), built up and then materialised. The tables that
follow are the full lookup: what each method returns, and whether it keeps the
chain alive.

## `Ags4File`: the read handle

`laterite.read(...)` returns this. Methods that return `self` (or a new
`Ags4File`) keep you on the handle; the rest hand you a query, a relation, or a
materialised value.

| Method / attr                                     | Returns                      | Chainable?                                                  | Example                      |
| ------------------------------------------------- | ---------------------------- | ----------------------------------------------------------- | ---------------------------- |
| `read(path)` / `read(text=)` / `read(data=bytes)` | `Ags4File`                   | start of chain                                              | [ex01](../learn/read.md)     |
| `.validate()`                                     | `Ags4File` (same handle)     | yes (runs the rule engine, parks the `Report` on `.report`) | [ex02](../learn/validate.md) |
| `.fix()`                                          | `Ags4File` (new)             | yes (applies safe auto-fixes, original untouched)           | [ex15](../cookbook/index.md) |
| `.at(code, [ids])`                                | `AgsQuery`                   | → query (fan-out from a parent group)                       | [ex03](../learn/query.md)    |
| `.query(sql)`                                     | `AgsQuery` (lazy)            | → query (build with `.filter` / `.select`)                  | [ex05](../learn/query.md)    |
| `.sql(sql)`                                       | `DuckDBPyRelation`           | → relation (terminal: `.pl()` / `.df()` / `.arrow()`)       | [ex06](../learn/query.md)    |
| `.pipe(fn, *args)`                                | whatever `fn` returns        | depends on `fn`                                             | [ex07](../chaining/index.md) |
| `["CODE"]` / `.table(code)`                       | `polars.DataFrame`           | no (one group, materialised)                                | [ex01](../learn/read.md)     |
| `.groups`                                         | `list[str]`                  | no (group codes present)                                    | [ex01](../learn/read.md)     |
| `.report`                                         | `Report` \| `None`           | no (set by `.validate()`)                                   | [ex02](../learn/validate.md) |
| `.text`                                           | `str`                        | no (byte-faithful AGS4 source)                              | [ex08](../learn/produce.md)  |
| `.bytes`                                          | `bytes`                      | no (raw encoded source)                                     | [ex08](../learn/produce.md)  |
| `.certify(path)`                                  | writes an `.ags.idx` sidecar | no (mints the validity cert)                                | [ex08](../learn/produce.md)  |
| `.save(path)`                                     | writes the file              | no (terminal)                                               | [ex08](../learn/produce.md)  |

!!! tip "Born typed"
    `["CODE"]` and the query terminals hand back polars frames where the column
    **dtype is the AGS data type**: `LOCA_GL` arrives as `f64`, not a string to
    re-parse. See [Born typed](../concepts/born-typed.md).

## `AgsQuery`: the lazy query handle

`.at(...)` and `.query(...)` return this. Builder methods are immutable: each
returns a **new** `AgsQuery`, so the plan only runs when you call a terminal.

| Method           | Returns                | Chainable?                                 |
| ---------------- | ---------------------- | ------------------------------------------ |
| `.filter(expr)`  | `AgsQuery` (new)       | yes, a builder                             |
| `.select(*cols)` | `AgsQuery` (new)       | yes, a builder                             |
| `.frame()`       | `polars.DataFrame`     | terminal (single result, handle's backend) |
| `.to_polars()`   | `polars.DataFrame`     | terminal (single result)                   |
| `.to_pandas()`   | `pandas.DataFrame`     | terminal (single result)                   |
| `.relation()`    | `DuckDBPyRelation`     | terminal (single result, still lazy)       |
| `.frames()`      | `dict[str, DataFrame]` | terminal, fan-out (e.g. `["SAMP"]`)        |
| `.groups`        | `list[str]`            | fan-out (group codes in the result)        |

!!! warning "Single-result and fan-out are mutually exclusive"
    An `AgsQuery` is _either_ a single-result query (use `.frame` / `.to_polars`
    / `.to_pandas` / `.relation`) _or_ a fan-out (use `.frames` / `.groups`).
    `.at(code, [ids])` produces the fan-out shape, `.query(sql)` the
    single-result shape. Calling the wrong family raises. And every builder call
    (`.filter`, `.select`) returns a **new immutable** `AgsQuery`; the original
    is unchanged, so you can branch a plan safely. See
    [ex04](../learn/query.md) and [ex05](../learn/query.md).

## `Report`: the validation verdict

Parked on `Ags4File.report` after `.validate()`.

| Method / attr   | Returns         | Notes                                     |
| --------------- | --------------- | ----------------------------------------- |
| `.is_valid`     | `bool`          | the verdict (`True` when no ERROR)        |
| `.count`        | `int`           | number of findings shown                  |
| `.errors` / `.warnings` / `.fyi` | `int` | that count, split by tier          |
| `.dict_version` | `str`           | AGS edition the rules resolved to         |
| `.resolution`   | `str`           | how that edition was chosen               |
| `.findings`     | `list[Finding]` | rule · line · group · description         |
| `.to_json()`    | `str`           | JSON, same shape as `lat validate --json` |

→ worked example: [Validate](../learn/validate.md) ([ex02](../learn/validate.md)).

## `BuildResult`: the producer result

`build_ags4(frames | typed PROJ graph)` returns this. Same `.text` / `.bytes` /
`.save` door as `Ags4File`, plus what the build did.

| Method / attr    | Returns         | Notes                                 |
| ---------------- | --------------- | ------------------------------------- |
| `.text`          | `str`           | the emitted AGS4                      |
| `.bytes`         | `bytes`         | the emitted AGS4, encoded             |
| `.findings`      | `list[Finding]` | validation findings on the built file |
| `.fixes_applied` | `list[str]`     | what the emitter normalised           |
| `.save(path)`    | writes the file | terminal                              |

→ worked examples: [Produce](../learn/produce.md), from frames
([ex09a](../learn/produce.md)) or from a typed PROJ graph
([ex09b](../learn/produce.md)).

---

See also: the [CLI reference](./cli.md) for `lat`, and the
[Cookbook](../cookbook/index.md) for end-to-end recipes.
