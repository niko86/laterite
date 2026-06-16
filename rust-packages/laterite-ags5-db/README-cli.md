# ags5db

Rust CLI for **`.ags5db`** files (the DuckDB-backed AGS5 store) —
browse, query, convert. Read + write surface, parity-tested against
the Python `lat-db` CLI.

## Usage

    lat-db <command> [args] [global flags]
    lat-db --readme         # this document
    lat-db --help           # full command list
    lat-db <command> --help # flags for one command

## Commands

Read:

    info          file summary + per-group row counts
    groups        list groups with row counts
    headings      schema (headings) for one group
    peek          view rows from one group (safe alternative to sql)
    count         row count for one group, optionally filtered
    sum           SUM(field) on one group, optionally filtered
    sql           raw read-only DuckDB SELECT (auto-LIMIT 1000)
    recipe        print a query template from the catalogue
    agent-context one-call warm-up: metadata + groups + samples
    inspect       dump the self-describing _spec_* tables
    diff          compare two .ags5db files (exit 1 on diff)

Write / convert:

    pack / unpack         .ags5db ↔ .ags5db.zst (zstd transport)
    lock / unlock         + passphrase-encrypt (.ags5db.zst.age)
    ags4-to-db            AGS4 (CSV-with-headers) → .ags5db
    db-to-ags4            .ags5db → AGS4 (+ FILE/<fset>/<name> sidecar
                          tree from stored blobs; --validate to spec-check)
    db-to-agsx            .ags5db → .agsx  (tar+zstd of XML + L-CSVs)

## Global flags

    -o, --output <MODE>  table | json | ndjson | csv | tsv
                         (default: table on a TTY, ndjson when piped)
    --json               shortcut for --output json (pretty)
    --no-color           disable ANSI (also honours NO_COLOR)
    -q, --quiet          suppress progress lines on stderr
    --readme             print this document and exit

Results go to **stdout** in the chosen mode; progress/diagnostics to
**stderr** — pipe-clean for scripting and agents.

## Exit codes

    0  success
    1  diff found (diff command)
    2  pre-6.5 file (inspect/headings)
    3  file not found / unreadable
    4  unknown group code
    5  --where predicate parse error
    6  schema error
    7  unsupported feature (e.g. AGS4 Record Link on db-to-ags4)
    8  SQL error (sql command)
    10 validation failed (db-to-ags4 --validate: output written but
       not spec-conformant)

## Examples

    lat-db info site.ags5db
    lat-db peek site.ags5db LOCA --where "LOCA_GL > 50" -o csv
    lat-db sql  site.ags5db "SELECT LOCA_ID FROM v_LOCA LIMIT 5" --json
    ags5db ags4-to-db delivery.ags site.ags5db
    ags5db db-to-ags4 site.ags5db out.ags --validate
    lat-db diff old.ags5db new.ags5db        # exit 1 if they differ

`db-to-ags4` reconstructs the AGS4 Rule 20 sidecar layout: stored
attachment blobs are written as `FILE/<FILE_FSET>/<FILE_NAME>` next to
the output `.ags` (FSET + original name recovered from the FILE group;
an unresolvable blob falls back to a flat write + warning). Spec-check
that tree with `lat-check out.ags --check-files` (the on-disk Rule 20
half — off by default, see the validator README).
