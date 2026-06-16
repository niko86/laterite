# lat-check

A clean-room Rust validator for the **AGS4** geotechnical
transfer-format — the capability `python-ags4`'s `AGS4.check_file()`
provides, with no Python at runtime. Reports the numbered AGS Format
Rule violations in a file.

## Usage

    lat-check <file.ags> [options]
    lat-check --readme        # this document

## Options

    --dict-version <V>   bundled dictionary edition:
                         auto (default — picked from the file's
                         TRAN_AGS) | 4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2
    --dict <path>        external dictionary override (not supported)
    --json               machine-readable findings (pretty JSON)
    --ndjson             one flat JSON object per finding per line
    --out <path>         write the active format to <path> instead of
                         stdout (prints a one-line confirmation)
    --json-out <path>    also tee the JSON report to <path>
    --show-warnings      include WARNING-severity findings
    --show-fyi           include FYI-severity findings (e.g. Rule 1)
    --check-files        also run Rule 20's on-disk check: the sidecar
                         FILE/<fset>/<name> tree must exist next to the
                         .ags. Default OFF — data-level Rule 20 is
                         path-independent (the library default); enable
                         for a packaging/QA pass on a real delivery.
    --quiet              suppress the progress spinner
    --tui                interactive findings browser (needs the
                         `tui` build feature + an interactive terminal)
    --readme             print this document and exit
    -h, --help           short usage

## Dictionary auto-selection

By default the edition is chosen **per file from its `TRAN_AGS`**:
an exact bundled match wins (`4.0.3/4.0.4/4.1/4.1.1/4.2`), else the
newest bundled patch of that `major.minor` (`4.0`→4.0.4, `4.1.5`→
4.1.1), else a fallback to 4.1.1 (matching python-ags4's
`LATEST_DICT_VERSION`). AGS 3.x is refused (exit 4) rather than
silently validated against an AGS4 schema. `--dict-version` forces
one edition regardless.

## Output

Human table on a TTY (coloured unless `NO_COLOR`); `--json` for a
nested `{file, findings:{rule:[{line,group,desc}]}}` document;
`--ndjson` for a stream. Progress is on stderr; the report on stdout.

## Exit codes

    0  clean (no findings)
    1  findings present
    3  file not found / unreadable
    4  not UTF-8 / not an AGS4 file / unsupported AGS edition (3.x)
    5  bad arguments / bad dictionary

## Examples

    lat-check delivery.ags                 # human table
    lat-check delivery.ags --json | jq .   # machine-readable
    lat-check delivery.ags --dict-version 4.2   # force an edition
    lat-check delivery.ags --show-fyi --out report.txt

> The clean-room boundary, licence, and bundled-dictionary provenance
> are documented in the crate's `README.md` and `data/PROVENANCE.md`.
