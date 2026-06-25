# Validate a delivery

Run the numbered AGS Format Rules over a transfer file and get a verdict —
in-process from Python, or at the shell with `lat-check`.

=== "Python"

    ```python
    --8<-- "python/ex02_validate.py"
    ```

    ```text
    is_valid=True count=0 dict_version='4.1.1' resolution='exact'
    ```

    `read(path).validate()` runs the rule engine and returns the
    [`Ags4File`](../learn/read.md), so it chains; the verdict rides on `.report`.
    `is_valid` and `count` are the headline. `dict_version` and `resolution`
    tell you *which* edition the rules came from — both read straight off the
    file: the edition is taken from `TRAN_AGS`, and `resolution='exact'` means
    that edition was matched on the nose (otherwise laterite falls back to the
    nearest dictionary it ships). You never pass an edition.

    Because `.validate()` hands the file back, keep going on the same handle —
    query it, slice a group, or emit it — without re-reading from disk.

=== "CLI"

    A clean file prints one line and exits `0`:

    ```bash
    lat-check examples/sample_site.ags
    ```

    ```text
    examples/sample_site.ags: clean (0 findings)
    ```

    A file with problems prints a count line, then a
    `Rule | Line | Group | Description` table — one row per violation — and
    exits `1`. File-level findings (a missing group) carry no line, shown as `-`:

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

    Add `--json` for machine-readable findings keyed by rule. The exit code is
    unchanged — `0` for an empty `findings`, `1` when it's populated — so a CI
    gate can branch without parsing stdout:

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

Both doors run the same clean-room rule engine and pick the dictionary edition
from the file's `TRAN_AGS` — the Python `.report` and the CLI's `--json` carry
identical findings. Reach for the CLI in a build gate (exit `1` fails the
build); reach for `.validate()` when you want to keep working with the parsed
file in the same process.

See also: [lat-check CLI](../reference/cli.md) · [Certify a file](./certify.md)
