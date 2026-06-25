# lat-check CLI

The shipped Rust validator binary. `pip install laterite` puts `lat-check` on
your `PATH`. Point it at an AGS4 file; it reports the numbered AGS Format Rule
violations and sets an exit code you can branch on.

## Usage

```bash
lat-check FILE [--json]
```

`FILE` is an AGS4 transfer file. The dictionary edition is picked automatically
from the file's `TRAN_AGS` row — no flag needed.

## A clean run

```bash
lat-check examples/sample_site.ags
```

```text
examples/sample_site.ags: clean (0 findings)
```

A valid file prints one line and exits `0`. Nothing else is written.

## A run with findings

```bash
lat-check delivery.ags
```

```text
delivery.ags: 4 finding(s)
Rule | Line | Group | Description
-----+------+-------+-----------------------------------------------------
14   | -    | TRAN  | TRAN group not found.
15   | -    | UNIT  | UNIT group not found.
17   | -    | TYPE  | TYPE group not found.
4    | 5    | PROJ  | DATA row field count does not match the HEADING row.
```

A count line, then a `Rule | Line | Group | Description` table — one row per
violation. File-level findings (a missing group) carry no line, shown as `-`.
Exit code is `1`.

## `--json`

```bash
lat-check delivery.ags --json
```

```text
{
  "file": "delivery.ags",
  "findings": {
    "AGS Format Rule 14": [
      {
        "line": null,
        "group": "TRAN",
        "desc": "TRAN group not found."
      }
    ],
    "AGS Format Rule 4": [
      {
        "line": 5,
        "group": "PROJ",
        "desc": "DATA row field count does not match the HEADING row."
      }
    ]
  }
}
```

`findings` is keyed by rule; each value is a list of `{ line, group, desc }`
objects (`line` is `null` for file-level findings). A clean file gives the same
shape with an empty object:

```text
{
  "file": "examples/sample_site.ags",
  "findings": {}
}
```

This is the same JSON `Ags4File.report.to_json()` produces in Python.

!!! tip
    The exit code still tells you the verdict in JSON mode — `0` for an empty
    `findings`, `1` when it's populated — so a script can branch without parsing
    stdout.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Clean — the file is valid (no findings). |
| `1` | Findings — one or more AGS Format Rule violations. |
| other | I/O or usage error — file not found, unreadable, or a bad invocation. |

Reserve `0`/`1` for verdict logic; treat any other non-zero code as a failure to
run, not a validation result. In a CI gate, `lat-check FILE` failing the build on
exit `1` is usually all you need.

---

See also: the [Python cheatsheet](./cheatsheet.md) for the in-process API, and
[Validate](../learn/validate.md) for `.validate()`, which runs the same engine.
