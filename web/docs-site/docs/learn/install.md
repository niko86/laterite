# Install & first validate

<!-- doc-code: skip — installs packages — a gate that ran it would rewrite its own environment -->
```bash
pip install laterite
```

That one line gives you the library **and** the `lat` CLI. The base
install is **polars + duckdb** — pyarrow-free, no pandas. Optional extras
add drop-in surfaces only if you want them:

<!-- doc-code: skip — installs packages — a gate that ran it would rewrite its own environment -->
```bash
pip install laterite[compat]          # the python-ags4 drop-in shim (adds pandas) — pyarrow-free
pip install laterite[compat,pyarrow]  # + pyarrow accelerator (faster pandas hop + string dtype)
pip install laterite[pyarrow]         # the Arrow backend (adds pyarrow)
```

`[compat]` alone is already **~3× faster than python-ags4** (object-dtype pandas
via DuckDB). pyarrow is an optional accelerator — see
[Dependency shape](../concepts/dependency-shape.md).

## First validate, from the command line

Point `lat` at an AGS4 file. A clean file says so and exits `0`:

```bash
--8<-- "cli/validate_clean.sh:cmd"
```

```text
--8<-- "cli/validate_clean.out"
```

Break one value — say a `2DP` easting that isn't a number — and the same
command prints a findings table and exits `1`:

```bash
--8<-- "cli/validate_typo.sh:cmd"
```

```text
--8<-- "cli/validate_typo.out"
```

Each row is one numbered-rule violation: the **Rule** that fired, the **Line**
and **Group** it landed in, and a one-line **Description**. Here Rule 8 caught a
value that doesn't match its column's declared AGS TYPE.

Need machine-readable output? Add `--json` — `findings` is an empty object when
the file is clean:

```bash
--8<-- "cli/validate_clean_json.sh:cmd"
```

```text
--8<-- "cli/validate_clean_json.out"
```

!!! note "The exit-code contract"
    `lat` exits **`0`** when the file is clean and **`1`** when there are
    findings — nothing else. That makes it a drop-in gate for CI or a pre-commit
    hook: `lat validate delivery.ags` fails the step the moment a rule fires, no
    output parsing required.

Next → [Read & explore a file](./read.md)
