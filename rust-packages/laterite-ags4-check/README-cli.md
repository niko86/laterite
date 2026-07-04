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
    --fix                mechanically repair the file: apply the SAFE
                         fixes (CRLF / BOM / embedded-CR / short-row pad /
                         numeric reformat / TRAN delimiter+concatenator
                         rows) and write the result. Non-destructive —
                         writes a sibling <file>.fixed.ags by default.
                         Exit 0 if the repaired file is clean, 1 if
                         findings remain that can't be auto-fixed.
    --fix-risky          like --fix but ALSO applies the intent-guessing
                         fixes (duplicate-heading rename, dd/mm date
                         canonicalisation, smart-quote→ASCII typography)
    --in-place           with --fix: overwrite the source file in place
    --fix-out <path>     with --fix: write the repaired file to <path>
    --diff <other.ags>   compare the input file against <other> and print the
                         KEY-aware/type-aware revision delta (per-group
                         +added -removed ~changed; --json for the full delta)
    --no-warnings        errors only — suppress the WARNING tier, which is
                         shown by default (malformed DICT, nonstandard
                         abbreviations, unrecognised TRAN_AGS edition)
    --show-fyi           include FYI-severity findings (e.g. Rule 1)
    --check-files        also run Rule 20's on-disk check: the sidecar
                         FILE/<fset>/<name> tree must exist next to the
                         .ags. Default OFF — data-level Rule 20 is
                         path-independent (the library default); enable
                         for a packaging/QA pass on a real delivery.
    --encoding <name>    source text encoding for legacy extended-ASCII
                         files: utf-8 (default) | cp1252 | latin1 |
                         iso-8859-1 | iso-8859-15. latin1 / iso-8859-1
                         map to Windows-1252 (the CP1252 superset
                         python-ags4 uses by default).
    --list-rules         print the AGS4 rule catalogue (title / severity /
                         fixable / cited observations) and exit; add --json
                         for the full machine-readable form. No input file.
    --emit-index         after a clean check, mint the .ags.idx validity
                         certificate (byte-offset index + validation
                         provenance) beside the file. Skipped if the file
                         still has errors; warnings/FYI don't block it.
    --index-out <path>   with --emit-index: write the certificate to <path>
                         instead of <file>.ags.idx
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
    lat-check delivery.ags --fix                # → delivery.fixed.ags
    lat-check delivery.ags --fix --in-place     # repair in place
    lat-check delivery.ags --fix-risky --fix-out clean.ags
    lat-check delivery.ags --emit-index         # → delivery.ags.idx (if clean)

> The clean-room boundary, licence, and bundled-dictionary provenance
> are documented in the crate's `README.md` and `data/PROVENANCE.md`.
