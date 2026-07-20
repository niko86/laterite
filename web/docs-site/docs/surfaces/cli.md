# CLI — one tool, three launchers

`lat` is the AGS4 engine for the **shell** — verb-oriented, built for CI gates and
one-liners, with machine-readable output and meaningful exit codes. It is the same
tool however you launch it, so everything below is written as `lat …`:

| Launcher           | Get it                                                         | Run                                      |
| ------------------ | -------------------------------------------------------------- | ---------------------------------------- |
| **Native binary**  | [GitHub Releases](https://github.com/niko86/laterite/releases) | `lat …`                                  |
| **Python** (`uvx`) | `pip install laterite`                                         | `uvx --from laterite lat …` (or `lat …`) |
| **Node** (`npx`)   | `npm install -g laterite`                                      | `npx laterite …` (or `lat …`)            |

Same verbs, same flags, and **byte-identical** `--json` / `--ndjson` output across
all three — so a CI gate, a Python shop, and a Node service can share the same
downstream tooling.

## The verbs

| Verb                  | What it does                                                                                                 |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| `validate <file>`     | run the numbered AGS Format Rules (the **default** — a bare `lat <file>` runs it)                            |
| `read <file> [group]` | dump a group's rows (table / `--csv` / `--json`), or list the group codes                                    |
| `fix <file>`          | mechanically repair — safe fixes by default, `--risky` for the intent-guessing ones                          |
| `diff <a> <b>`        | the KEY-aware / type-aware revision delta                                                                    |
| `certify <file>`      | mint the [`.ags.idx`](../concepts/certificate-lifecycle.md) validity certificate                             |
| `rules`               | print the AGS4 rule catalogue                                                                                |
| `pack` / `unpack`     | zstd-compress a file for transport, and restore it                                                           |
| `lock` / `unlock`     | add an age passphrase envelope (passphrase from `--password-file` / `$LAT_TRANSPORT_PASSWORD`, never a flag) |
| `excel <in> <out>`    | convert AGS4 ↔ Excel (direction inferred from the output extension)                                          |

For querying across groups (SQL) or building AGS4 from data, reach for
[Python](python.md) or [DuckDB](../duckdb/index.md) — the [capability
matrix](index.md#what-each-door-can-do) shows the split.

## CLI-native idioms

**A CI gate is just the exit code** — `0` when clean, non-zero when findings remain:

```bash
lat validate "$f" || exit 1        # fail the pipeline on any finding
```

`0` clean · `1` findings · `3` not-found / unreadable · `4` not-AGS4 / bad input ·
`5` bad arguments · `6` schema violation.

**Pipe the machine output.** `--json` is pretty, `--ndjson` is one finding per line
— both stream to `jq`, and both are the same bytes the Python and Node reports emit:

```bash
lat validate delivery.ags --json | jq '.findings | keys'
lat read delivery.ags LOCA --csv > loca.csv
```

**A bare file validates.** `lat delivery.ags` is shorthand for
`lat validate delivery.ags` — the common case needs no verb.

**Package for transport without a data round-trip.** `pack` / `lock` move bytes
(any file type), so a `pack`ed-then-`unpack`ed file is byte-identical — safe to
validate later. The passphrase for `lock` / `unlock` never appears on the command
line (it would leak into `ps` and shell history):

```bash
LAT_TRANSPORT_PASSWORD=… lat lock delivery.ags delivery.ags.age
lat unlock delivery.ags.age out.ags --password-file pw.txt
```

!!! tip "The full reference"
Every verb, flag, and exit code — the shipped `lat --readme` guide — is the
[CLI command reference](../reference/cli.md).
