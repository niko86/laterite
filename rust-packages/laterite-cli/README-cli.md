# lat

A clean-room Rust tool for the **AGS4** geotechnical transfer format —
validate / read / fix / diff / certify a file, and pack / lock it for transport,
with no Python at runtime. The `validate` capability is what `python-ags4`'s
`AGS4.check_file()` provides. A bare `lat <file.ags>` is shorthand for
`lat validate <file.ags>`.

## Usage

    lat <command> [options]
    lat <file.ags>            # shorthand for: lat validate <file.ags>
    lat <command> --help      # help for one command
    lat --readme              # this document

## Commands

    validate <file>   run the numbered AGS Format Rules and report (the default)
    read <file> [grp] dump a group's rows (table / --csv / --json), or list codes
    fix <file>        mechanically repair the file (safe fixes; --risky for more)
    diff <a> <b>      KEY-aware / type-aware revision delta between two files
    merge <files...>  reconcile 2+ deliveries of one project into one file (--out)
    certify <file>    mint the .ags.idx validity certificate for a clean file
    rules             print the AGS4 rule catalogue (no input file needed)
    pack <in> <out>   zstd-compress any file for transport (unpack reverses it)
    unpack <in> <out> restore a packed file
    lock <in> <out>   zstd + age passphrase-encrypt (unlock reverses it)
    unlock <in> <out> decrypt + decompress a locked file
    excel <in> <out>  convert AGS4 ↔ Excel (direction from the output extension)

## Global options

    --quiet           suppress the progress spinner

`--json` / `--ndjson` are per-verb, not global: a verb that renders a report
declares them, one that cannot never sees the flag. `validate` takes both;
`read` / `diff` / `merge` / `fix` / `rules` take `--json`. The verbs with no
report (certify / pack / unpack / lock / unlock / excel) reject them.

## validate <file>

    --dict-version <V>   bundled edition: auto (default — picked from the file's
                         TRAN_AGS) | 4.0.3 | 4.0.4 | 4.1 | 4.1.1 | 4.2.
                         With --dict, selects the overlay base.
    --dict <path>        custom dictionary (.ags or JSON) layered over a base
                         edition detected from the dictionary itself; overrides
                         of standard definitions are honoured with a warning
    --dict-replace       (with --dict) treat it as a FULL REPLACEMENT — no base
                         edition contributes (cannot combine with --dict-version)
    --encoding <name>    source encoding: utf-8 (default) | cp1252 | latin1 |
                         iso-8859-1 | iso-8859-15 (latin1 ≈ Windows-1252)
    --no-warnings        errors only — suppress the WARNING tier (shown by default)
    --show-fyi           include FYI-severity findings (e.g. Rule 1)
    --check-files        also run Rule 20's on-disk FILE/<fset>/<name> check
    --json               machine-readable findings (pretty JSON)
    --ndjson             one flat JSON object per finding per line
    --out <path>         write the active format to <path> instead of stdout
    --json-out <path>    also tee the JSON report to <path>
    --index <path>       CONSUME an .ags.idx: a fresh (bytes unchanged), same-
                         engine, profile-covering cert lets the check SKIP the
                         rule engine; a stale / foreign cert is re-validated
    --tui                interactive findings browser (needs the `tui` build feature)

## read <file> [group]

    Dump one group's rows as an aligned table (default), `--csv`, or `--json`;
    omit the group to list the file's group codes. Raw file cells (faithful to the
    bytes); `--csv` / `--json` are byte-identical to the Python `lat read`.

    --csv                CSV output (RFC-4180 quoting) instead of the table
    --out <path>         write to <path> instead of stdout

## fix <file>

    --risky              also apply the intent-guessing fixes (duplicate-heading
                         rename, ambiguous dd/mm date canonicalisation, smart-
                         quote→ASCII). Default: the SAFE fixes only.
    --in-place           overwrite the source file in place
    --fix-out <path>     write the repaired file to <path>
                         (default: a sibling <file>.fixed.ags)
                         (also honours --dict-version / --dict / --encoding)
    --json               machine-readable {file, dest, applied, residual} report
                         (the file is written either way; exit 0 clean / 1 residual)

## diff <a> <b>

    Compares <a> (baseline) against <b> (revision): a per-group
    +added -removed ~changed summary, or the full RevisionDelta with --json.
    (also honours --dict-version / --dict / --encoding)

## merge <files...>

    Reconcile two or more deliveries of one project into a single file. Files
    merge in argument order — a LATER file wins a KEY conflict — and rows are
    matched by their dictionary KEY headings, so a re-sorted borehole list still
    merges onto its prior self. The merge is a union: a row present in only one
    file is kept (silence is not deletion). A column two files typed differently
    ERRORS by default; --on-type-clash chooses how to settle it. A conflicting
    UNIT is fatal in EVERY mode (TYPE has an absorber, X; UNIT has none).
    --json emits the {warnings, revisions} audit.
    (also honours --dict-version / --dict / --encoding)

    --out <path>          write the merged file here (REQUIRED — never a default
                          over an input)
    --on-type-clash <MODE>  how to settle a heading the files typed differently:
                          error   refuse (default)
                          widen   fall back to X (text) — raw values kept, but
                                  the column's TYPE is thrown away
                          promote keep the greatest precision when every code is
                                  nDP (2DP + 5DP -> 5DP), zero-padding the coarser
                                  values (10.00 -> 10.00000). Never rounds, never
                                  demotes; nSF/nSCI fall back to widen
    --tran-issue <ISNO>   stamp a synthesised merge-TRAN (with --tran-date; records
                          the inputs' ISNOs/dates in TRAN_REM for provenance)
    --tran-date <DATE>    the merge-TRAN production date (yyyy-mm-dd)
    --tran-producer <NAME>    TRAN_PROD for the merge-TRAN
    --tran-recipient <NAME>   TRAN_RECV for the merge-TRAN
    --tran-status <STATUS>    TRAN_STAT for the merge-TRAN

## certify <file>

    --check-files        record Rule 20's on-disk check in the cert profile
    --out <path>         write the certificate to <path> (default <file>.ags.idx)
                         (also honours --dict-version / --dict / --encoding)

## rules

    --json               emit the raw machine-readable rules_meta.json

## transport — pack / unpack / lock / unlock

    Package any file (`.ags`, anything) for storage or transfer.
    `pack` / `unpack` are zstd; `lock` / `unlock` add an age passphrase envelope
    (zstd inside a standard age file — recoverable with stock `age` + `zstd`). The
    passphrase is NEVER a flag (argv leaks into `ps` + shell history): the
    precedence is `--password-file` → `$LAT_TRANSPORT_PASSWORD` → an interactive
    prompt. On by default; a `--no-default-features` build drops age/zstd entirely.

    pack <in> <out> [--level N]           zstd level 1–22 (default 9)
    unpack <in> <out>
    lock <in> <out> [--level N] [--log-n N] [--password-file <path>]
                     scrypt work factor log2(N) default 18 (the interop tier)
    unlock <in> <out> [--password-file <path>]

## excel <in> <out>

    Convert between AGS4 and Excel. The direction is inferred from the OUTPUT
    extension: `.xlsx` ⇒ export (AGS4 → Excel, one sheet per group), `.ags` ⇒
    import (Excel → AGS4). On by default; `--no-default-features` drops it.

    --export             force AGS4 → Excel (else inferred from the output)
    --import             force Excel → AGS4
    --no-format-numeric  (import) leave numeric-looking columns as text

## Dictionary auto-selection

By default the edition is chosen **per file from its `TRAN_AGS`**: an exact
bundled match wins (`4.0.3/4.0.4/4.1/4.1.1/4.2`), else the newest bundled patch
of that `major.minor` (`4.0`→4.0.4, `4.1.5`→4.1.1), else a fallback to 4.1.1
(matching python-ags4's `LATEST_DICT_VERSION`). AGS 3.x is refused (exit 4)
rather than silently validated against an AGS4 schema. `--dict-version` forces
one edition regardless.

## Output

Human table on a TTY (coloured unless `NO_COLOR`); `--json` for a nested
`{file, findings:{rule:[{line,group,desc}]}}` document; `--ndjson` for a stream.
Progress is on stderr; the report on stdout.

## Exit codes

    0  clean (no findings)
    1  findings present
    3  file not found / unreadable
    4  not UTF-8 / not an AGS4 file / unsupported AGS edition (3.x) / bad input
    5  bad arguments
    6  schema violation

## Examples

    lat delivery.ags                        # human table (= lat validate)
    lat validate delivery.ags --json | jq . # machine-readable
    lat validate delivery.ags --dict-version 4.2
    lat fix delivery.ags                    # → delivery.fixed.ags
    lat fix delivery.ags --in-place         # repair in place
    lat fix delivery.ags --risky --fix-out clean.ags
    lat read delivery.ags                   # list the group codes
    lat read delivery.ags LOCA --csv        # dump one group as CSV
    lat diff old.ags new.ags                # revision delta
    lat merge phase1.ags phase2.ags --out all.ags        # reconcile deliveries
    lat merge a.ags b.ags --out m.ags --on-type-clash promote  # keep max nDP
    lat certify delivery.ags                # → delivery.ags.idx (if clean)
    lat validate delivery.ags --index delivery.ags.idx  # skip the engine if fresh
    lat rules                               # the rule catalogue
    lat pack delivery.ags delivery.ags.zst  # zstd-compress for transport
    lat lock delivery.ags delivery.ags.age  # + age passphrase (prompts)
    lat unlock delivery.ags.age out.ags --password-file pw.txt
    lat excel delivery.ags delivery.xlsx    # AGS4 → Excel (one sheet per group)
    lat excel delivery.xlsx out.ags         # Excel → AGS4

> The clean-room boundary, licence, and bundled-dictionary provenance are
> documented in the crate's `README.md` and `data/PROVENANCE.md`.
