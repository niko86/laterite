# Install & first validate

```bash
pip install laterite
```

That one line gives you the library **and** the `lat-check` CLI. The base
install is **polars + duckdb** — pyarrow-free, no pandas. Two optional extras
add drop-in surfaces only if you want them:

```bash
pip install laterite[compat]    # the python-ags4 drop-in shim (adds pandas)
pip install laterite[pyarrow]   # the Arrow backend (adds pyarrow)
```

## First validate, from the command line

Point `lat-check` at an AGS4 file. A clean file says so and exits `0`:

```bash
lat-check examples/sample_site.ags
```

```text
examples/sample_site.ags: clean (0 findings)
```

Break one value — say a `2DP` easting that isn't a number — and the same
command prints a findings table and exits `1`:

```bash
lat-check sample_site_typo.ags
```

```text
sample_site_typo.ags: 1 finding(s)
Rule | Line | Group | Description
-----+------+-------+--------------------------------------------------------------------------
8    | 51   | LOCA  | Value "not-a-number" in LOCA_NATE does not match its declared TYPE "2DP".
```

Each row is one numbered-rule violation: the **Rule** that fired, the **Line**
and **Group** it landed in, and a one-line **Description**. Here Rule 8 caught a
value that doesn't match its column's declared AGS TYPE.

Need machine-readable output? Add `--json` — `findings` is an empty object when
the file is clean:

```bash
lat-check examples/sample_site.ags --json
```

```text
{
  "file": "examples/sample_site.ags",
  "findings": {}
}
```

!!! note "The exit-code contract"
    `lat-check` exits **`0`** when the file is clean and **`1`** when there are
    findings — nothing else. That makes it a drop-in gate for CI or a pre-commit
    hook: `lat-check delivery.ags` fails the step the moment a rule fires, no
    output parsing required.

Next → [Read & explore a file](./read.md)
