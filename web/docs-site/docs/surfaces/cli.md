# CLI — `lat-check`

`lat-check` is the shipped command-line validator: the engine as a single binary,
built for **CI gates and shell one-liners**. It's deliberately focused — validate,
fix, certify — with machine-readable output and meaningful exit codes.

```console
$ lat-check delivery.ags
delivery.ags: valid (4.1.1, exact) — 0 findings
```

## Validate in CI

```console
$ lat-check delivery.ags --json
```

Exit code is the gate: **`0`** when the file is clean, **non-zero** when findings
remain — so a pipeline step is just `lat-check "$f"`. `--json` (pretty) and
`--ndjson` (one finding per line) give machine-readable output; `--out` /
`--json-out` write to a file. The edition is picked from `TRAN_AGS`; force it with
`--dict-version 4.1|4.2`.

## Fix in place

```console
$ lat-check delivery.ags --fix --in-place
```

`--fix` applies the **safe** mechanical repairs (CRLF / BOM / embedded-CR /
short-row padding); `--fix-out <path>` writes a copy, `--in-place` overwrites, and
`--fix-risky` also applies the intent-guessing fixes. Exit `0` if the result is
clean, `1` if findings remain that can't be auto-fixed.

## Certify

```console
$ lat-check delivery.ags --emit-index
# → delivery.ags.idx  (a validity certificate; --index-out <path> to redirect)
```

After a clean validation, `--emit-index` mints the [`.ags.idx`
certificate](../concepts/certificate-lifecycle.md) beside the file — the same
sidecar the library surfaces read to skip re-validating an unchanged file.

## Where it stops — by design

`lat-check` is a **validator**, not a data library: it doesn't read groups into
frames, query across groups, build AGS4 from data, or diff revisions. Reach for
[Python](python.md) or [DuckDB](../duckdb/index.md) for those — the [capability
matrix](index.md#what-each-door-can-do) shows the split at a glance.

!!! tip "Full flag reference"
    Every flag and exit code is in the [CLI reference](../reference/cli.md).
