# lat-check CLI

`lat-check` is laterite's command-line validator. **`pip install laterite` puts it
on your `PATH`** — the Python face of the engine, with the same flags, JSON/NDJSON
shapes, and exit codes as the native build. A prebuilt **native binary** is also
attached to each [GitHub release](https://github.com/niko86/laterite/releases)
(the `laterite-binaries-*` archives).

Point it at an AGS4 file and it reports the numbered AGS Format Rule violations —
and can also **repair** the file, **diff** two revisions, or print the **rule
catalogue**. It always sets an exit code you can branch on.

```bash
lat-check FILE [options]
lat-check --readme         # the full built-in guide
lat-check -h               # short usage
```

The dictionary edition is auto-detected from the file's `TRAN_AGS` row — no flag
needed (override with `--dict-version`).

## Validate (the default)

```bash
lat-check examples/sample_site.ags
```

```text
examples/sample_site.ags: clean (0 findings)
```

A valid file prints one line and exits `0`. A file with problems prints a count,
then a `Rule | Line | Group | Description` table — one row per violation
(file-level findings, like a missing group, carry no line, shown as `-`) — and
exits `1`:

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

### Severity tiers

Findings are graded **error / warning / FYI** (see [Severity tiers](../concepts/severity-tiers.md)).
By default `lat-check` shows errors **and** warnings (malformed `DICT`, nonstandard
abbreviations, an unrecognised `TRAN_AGS` edition):

```bash
lat-check delivery.ags --no-warnings   # errors only
lat-check delivery.ags --show-fyi      # also include FYI findings (e.g. Rule 1)
```

## Repair — `--fix`

Mechanically repair a file and write the result — non-destructive by default
(a sibling `<file>.fixed.ags`). Exit `0` if the repaired file is clean, `1` if
findings remain that can't be auto-fixed.

```bash
lat-check delivery.ags --fix           # apply the SAFE fixes
```

Safe fixes: CRLF / BOM / embedded-CR normalisation, short-row padding, numeric
reformatting, and synthesising the `TRAN` delimiter/concatenator rows.

| Flag | Effect |
| --- | --- |
| `--fix` | safe fixes only → `<file>.fixed.ags` |
| `--fix-risky` | also apply intent-guessing fixes (duplicate-heading rename, dd/mm date canonicalisation, smart-quote → ASCII) |
| `--in-place` | overwrite the source file instead of writing a sibling |
| `--fix-out PATH` | write the repaired file to `PATH` |

## Diff two revisions — `--diff`

```bash
lat-check old.ags --diff new.ags
```

A KEY-aware, type-aware delta per group (`+added` / `-removed` / `~changed`);
add `--json` for the full machine-readable delta.

## The rule catalogue — `--list-rules`

```bash
lat-check --list-rules            # no input file
lat-check --list-rules --json     # machine-readable
```

Prints every AGS4 rule with its title, severity, whether it's auto-fixable, and
the observations it cites.

## Output formats

| Flag | Output |
| --- | --- |
| *(none)* | the human-readable table above |
| `--json` | one pretty JSON report — the same shape Python's `Ags4File.report.to_json()` produces |
| `--ndjson` | one flat JSON object per finding, per line — stream-friendly |
| `--out PATH` | write the active format to `PATH` instead of stdout |
| `--json-out PATH` | also tee the JSON report to `PATH` |

```bash
lat-check delivery.ags --json
```

```json
{
  "file": "delivery.ags",
  "findings": {
    "AGS Format Rule 14": [
      { "line": null, "group": "TRAN", "desc": "TRAN group not found." }
    ],
    "AGS Format Rule 4": [
      { "line": 5, "group": "PROJ", "desc": "DATA row field count does not match the HEADING row." }
    ]
  }
}
```

`findings` is keyed by rule; each value is a list of `{ line, group, desc }`
(`line` is `null` for file-level findings). A clean file gives the same shape
with an empty `findings: {}`.

!!! tip
    The exit code carries the verdict in every format — `0` for clean, `1` for
    findings — so a script can branch without parsing stdout.

## Other options

| Flag | Purpose |
| --- | --- |
| `--dict-version V` | force a dictionary edition (`4.0.3` … `4.2`) instead of auto-detecting from `TRAN_AGS` |
| `--encoding NAME` | source encoding for legacy files: `utf-8` (default), `cp1252`, `latin1`, `iso-8859-1`, `iso-8859-15` |
| `--check-files` | also run Rule 20's on-disk check (the `FILE/<fset>/<name>` sidecar tree must exist next to the `.ags`) — off by default |
| `--quiet` | suppress the progress spinner |
| `--tui` | interactive findings browser (needs the `tui` build feature + an interactive terminal) |
| `--readme` | print the full built-in guide and exit |

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Clean — the file is valid (no findings). |
| `1` | Findings — one or more rule violations (or, with `--fix`, fixes that couldn't be applied remain). |
| other | I/O or usage error — file not found, unreadable, or a bad invocation. |

Reserve `0`/`1` for verdict logic; treat any other non-zero code as a failure to
run, not a validation result. In a CI gate, `lat-check FILE` failing the build on
exit `1` is usually all you need.

---

See also: the [Python cheatsheet](./cheatsheet.md) for the in-process API, and
[Validate](../learn/validate.md) for `.validate()`, which runs the same engine.
