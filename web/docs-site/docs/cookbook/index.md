# Cookbook

Task-indexed recipes. Each one is a runnable block you can lift straight into your code — the snippet
on the page is the *exact* file the CI gate executes, so the output is real. New here? the
[Learn path](../learn/index.md) walks the same ground in order; this page is for *"how do I…?"*.

## Reading & typing

- [**Get one group as a typed frame**](get-typed-frame.md) — `read(...)["LOCA"]`, born-typed.
- [**Read XN columns as numeric**](read-xn-numeric.md) — opt-in `xn="numeric"` for the AGS numeric-or-text type.
- [**Explore the registry & KEY chain**](explore-registry.md) — `child_groups` / `inherited_key_names`.

## Validating

- [**Validate a delivery**](validate-a-delivery.md) — in Python and on the `lat` CLI (+ exit codes).
- [**List the rules / report the edition**](list-rules.md) — `list_rules()` and `dict_for()`.

## Querying

- [**Filter & select one group**](filter-select.md) — the lazy `.query().filter().select()` builder.
- [**SQL across groups**](sql-across-groups.md) — `.sql(...)` joins on the shared keys.
- [**Pull one borehole's record set**](borehole-record-set.md) — `.at(...)` fan-out → `.frames()`.
- [**Splice your own step (`.pipe`)**](pipe.md) — drop a function into the chain.

## Producing AGS4

- [**Build from frames**](build-from-frames.md) — `build_ags4({code: frame})`.
- [**Build from a typed graph**](build-from-typed-graph.md) — `build_ags4(PROJ(...))` (#214).

## Repairing & comparing

- [**Fix a dirty file**](fix-a-dirty-file.md) — `.fix()` → a new, repaired handle.
- [**Diff two revisions**](diff-revisions.md) — KEY-aware, type-aware.
- [**Certify a clean file**](certify.md) — mint an `.ags.idx` and skip re-validation.

## Sharing & migrating

- [**Pack / encrypt for transport**](transport.md) — zstd + optional age encryption.
- [**Drop-in for python-ags4**](compat.md) — `from laterite import compat as AGS4`.

!!! tip
    Want to see the whole fluent API assembled from these parts? The [Chaining showcase](../chaining/index.md)
    climbs a power ladder from a one-line `read().validate()` to raw SQL, your own functions, and the
    certify fast-path.
