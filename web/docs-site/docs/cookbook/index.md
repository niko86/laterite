# Cookbook

A task-indexed map of "how do I…?" — each entry points to the worked example (and the Learn page) that answers it. Skim for your task; follow the link for the runnable recipe.

## Reading & typing

- **Get one group as a typed frame** — read a group straight into a polars frame whose dtypes *are* the AGS types. See [Read a group as a typed frame](../learn/read.md).
- **Read XN columns as numbers / explore the registry** — pull `XN`-typed columns as numeric and browse the group/heading registry. See [Explore the registry & XN columns](../learn/query.md).

## Validating

- **Validate a file in Python** — run the numbered-rule engine and read the `Report`. See [Validate in Python](../learn/validate.md).
- **List the rules / report the edition** — enumerate the validator's rules and the dictionary edition a file resolved to. See [Rules & the dictionary](../learn/validate.md).
- **Validate from the command line** — `lat-check FILE` for a clean/findings verdict (and `--json`). See the [CLI reference](../reference/cli.md).

## Querying

- **Filter & select within one group** — build a lazy query, narrow rows and columns, then materialise. See [Query a group](../learn/query.md).
- **Join across groups with SQL** — run SQL over the whole file (DuckDB under the hood). See [SQL across groups](../learn/query.md).
- **Pull a borehole's record set** — fan out from `LOCA` to its child groups and grab the frames. See [Fan-out from a location](../learn/query.md).

## Producing AGS4

- **Build AGS4 from frames** — assemble a `BuildResult` from a dict of frames, then `.text` / `.bytes` / `.save`. See [Produce AGS4](../learn/produce.md).
- **Build AGS4 from a typed graph** — construct the `PROJ` typed-class tree and emit it. See [Produce AGS4](../learn/produce.md).

## Repairing

- **Fix a dirty file** — apply safe repairs to a non-conforming delivery and re-emit. Validate first ([Validate in Python](../learn/validate.md)), then fix what's reported.

## Comparing, certifying & sharing

- **Diff two revisions** — compare two versions of a file group-by-group to see what moved between deliveries.
- **Certify a clean file** — mint an `.ags.idx` certificate so a later `.validate()` can skip the rule engine. See the [certify CLI](../reference/cli.md).
- **Pack & encrypt for transport** — `pack` / `lock` a file (and `unlock` / `unpack`) for handing off.

## Migrating

- **Drop-in for python-ags4** — swap `python_ags4` for `laterite.compat` with minimal edits, keeping the pandas backend. See [Install & import](../learn/install.md).
