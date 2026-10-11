# Merge two deliveries

**Available in:** Python · Node · CLI · [Browser](../surfaces/browser.md)

**When:** two deliveries of _one_ project need to become one file (a phased
site investigation, or a resubmission that revises some boreholes and adds
others) and you want the engine to reconcile them, not a manual copy-paste.

Merging is **KEY-aware**, not a file concatenation. Rows are matched on each
group's dictionary **KEY headings**, so a re-sorted borehole list still merges
onto its prior self. Files merge in **argument order** (a later file wins a KEY
conflict), and the result is a **union**: a row present in only one file is kept
(silence is not deletion). Every surface reports the same audit: the per-row
_revisions_ a later file made, plus any warnings.

=== "CLI"

    `lat merge <files…> --out <merged.ags>` reconciles two or more deliveries on
    disk. The optional `--tran-issue` / `--tran-date` stamp the merged file's own
    transmission record (it genuinely *is* a new transmission):

    ```bash
    --8<-- "cli/merge_deliveries.sh:cmd"
    ```

    ```text
    --8<-- "cli/merge_deliveries.out"
    ```

    The summary names each **revision** (here the second delivery changed
    `PROJ_NAME`, matched on the `PROJ_ID` KEY `LAT-DEMO`) and flags that a revised
    parent (`PROJ`) has child groups worth re-checking. When two files declare a
    column with *different* AGS types, `merge` errors (exit `6`) rather than guess;
    `--on-type-clash promote` or `--on-type-clash widen` settles it (see
    [When the two files disagree on a type](#when-the-two-files-disagree-on-a-type)).

=== "Python"

    `laterite.merge(*sources, …)` takes two or more of anything `read` accepts
    (paths, text, bytes, `Ags4File` handles) and returns a `MergeResult` carrying
    the merged `bytes` plus the `warnings` and per-row `revisions` audit:

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    import laterite

    res = laterite.merge(
        "phase1.ags", "phase2.ags",
        tran=laterite.TranStamp(
            issue="3", date="2024-01-15", producer="Us",
            recipient="Client", status="Merged",
        ),
    )
    for rev in res.revisions:
        print(rev["group"], rev["key"], "changed", rev["changed"])
    res.save("merged.ags")
    ```

    An AGS-type clash between two files raises `MergeConflictError` by default;
    `on_type_clash="promote"` or `"widen"` settles it (see
    [below](#when-the-two-files-disagree-on-a-type)). `res.text` decodes the merged
    bytes, and `res.save(path)` writes them.

=== "Node"

    `merge(sources, opts)` runs the same leaf and returns the same
    `{ bytes, warnings, revisions, text }`:

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    import { merge } from "laterite";

    const res = merge(["phase1.ags", "phase2.ags"], {
      tran: {
        issue: "3", date: "2024-01-15", producer: "Us",
        recipient: "Client", status: "Merged",
      },
    });
    for (const rev of res.revisions) {
      console.log(rev.group, rev.key, "changed", rev.changed);
    }
    ```

    A bare `string` is a **path** in Node, so pass a `Buffer` / `Uint8Array` when a
    delivery only exists in memory. A type clash throws `MergeConflictError`;
    `{ onTypeClash: "promote" }` or `"widen"` settles it (see
    [below](#when-the-two-files-disagree-on-a-type)).

## When the two files disagree on a type

One delivery types `LOCA_GL` as `2DP`, the next types it `5DP`. Merge will not
guess: by default it **errors** (exit `6` / `MergeConflictError`). You choose how
to settle it.

| mode                | what the merged column becomes                     | your values                                      |
| ------------------- | -------------------------------------------------- | ------------------------------------------------ |
| `error` _(default)_ | not produced                                       | merge refuses                                    |
| `widen`             | `X` (free text)                                    | kept byte-for-byte                               |
| `promote`           | the **greatest precision**: `2DP` + `5DP` → `5DP`  | coarser values zero-padded: `10.00` → `10.00000` |

`widen` is lossless on the bytes but **throws the type away**, and `X` is the
least informative answer available. `promote` keeps the column _numeric_.

=== "CLI"

    <!-- doc-code: skip — illustrative `lat` usage over placeholder paths; the CLI examples that ARE executed live in examples/cli/ (#513) -->
    ```bash
    lat merge phase1.ags phase2.ags --out merged.ags --on-type-clash promote
    ```

=== "Python"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    res = laterite.merge("phase1.ags", "phase2.ags", on_type_clash="promote")
    ```

=== "Node"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    const res = merge(["phase1.ags", "phase2.ags"], { onTypeClash: "promote" });
    ```

**`promote` never rounds and never demotes.** It only ever _appends zeros_, so no
digit you wrote is ever changed. That also makes the result independent of
argument order (unlike a KEY conflict, where the later file deliberately wins). A
value it cannot pad losslessly is kept verbatim and warned about, never rounded.

**It is deliberately limited to `nDP`.** Significant figures (`3SF`) and scientific
notation (`2SCI`) fall back to `widen`, because decimal places are a formatting
convention but significant figures are a claim about _measured_ precision:
padding `3SF` to `5SF` would assert two digits the instrument never resolved.

!!! tip "Why promote matters downstream"

    `_content_hash` fingerprints a row's *values*, canonicalised through the declared
    type, so `10.00` hashes as a **number** under `2DP` but as a **string** under
    `X`. A `widen`ed column therefore stops matching its own typed source, while a
    `promote`d one still dedups against it.

!!! warning "A conflicting UNIT is fatal in every mode"

    `TYPE` has a universal absorber (`X`); `UNIT` has none. There is no supertype of
    metres and millimetres, and merge will never *convert*, so if two files declare
    different UNITs for one heading, merge refuses in **every** mode, `promote`
    included. Reconcile the `UNIT` row at source.

## Write the rows in KEY order

Merged rows are written in the order they were first seen: every row of the
first delivery, then the rows only the second delivery has, and so on. A
borehole the later delivery adds therefore lands after the earlier delivery's
last row, wherever its ID would sort. `row_order` chooses the order instead.

| mode                | what order the merged rows are written in                                                                                                                                                      |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `input` _(default)_ | first-seen order across the deliveries                                                                                                                                                         |
| `key`               | each group sorted by its KEY headings: numeric columns by value, the rest in natural order (`BH2` before `BH10`), blanks last. `TRAN`, `TYPE`, `UNIT`, `ABBR` and `DICT` stay as given         |

Sorting happens after the deliveries are reconciled, so it never changes which
row wins, and the revisions and warnings are the same in both modes.

=== "CLI"

    Here the second delivery adds a borehole, `BH03A`, beside `BH03`. In input
    order it would be written after `BH14`; `--row-order key` puts it where its
    ID sorts:

    ```bash
    --8<-- "cli/merge_row_order.sh:cmd"
    ```

    ```text
    --8<-- "cli/merge_row_order.out"
    ```

=== "Python"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    res = laterite.merge("phase1.ags", "phase2.ags", row_order="key")
    ```

=== "Node"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    const res = merge(["phase1.ags", "phase2.ags"], { rowOrder: "key" });
    ```

## Prune DICT rows that restate the standard

Some exporters write a `DICT` row for every heading in the file, standard
headings included. Merge unions the deliveries' `DICT` groups like any other
group, so those rows pile up. `dict_rows` decides what to keep.

| mode               | what happens to the merged `DICT`                                                                                                                                                                                                                                                           |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `keep` _(default)_ | the unioned rows are written as reconciled                                                                                                                                                                                                                                                 |
| `prune`            | every row that restates a group or heading the edition's standard dictionary already defines is dropped, and so is every row defining a group or heading the merged file does not contain. `DICT` is written after the other groups, and left out when no row remains |

Pruning never adds a finding.

=== "CLI"

    Here the second delivery carries a `DICT` group whose one row restates
    `LOCA_GL`, a standard heading. With `--dict-rows prune` nothing of it
    survives, so the merged file's group list has no `DICT`:

    ```bash
    --8<-- "cli/merge_dict_rows.sh:cmd"
    ```

    ```text
    --8<-- "cli/merge_dict_rows.out"
    ```

=== "Python"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    res = laterite.merge("phase1.ags", "phase2.ags", dict_rows="prune")
    ```

=== "Node"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    const res = merge(["phase1.ags", "phase2.ags"], { dictRows: "prune" });
    ```

## Settle a code's letter case

`ABBR` codes are matched exactly, so `CP` and `cp` under one heading are two
codes to the validator, while many importers key the list ignoring case and
reject the pair as a duplicate. When two deliveries spell one code in different
letter case, merge warns in every mode (`abbr_code_case`), and `on_code_case`
decides whether it also settles the spelling.

| mode               | what the merged file does with the set                                                                                                                                                                                                                      |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `keep` _(default)_ | every spelling is left as written                                                                                                                                                                                                                          |
| `standard`         | the set is rewritten to the edition's standard code when exactly one standard code matches it ignoring case, in the `ABBR` rows and in every `PA` cell under that heading. A set with no standard match, or with several, is left as written |

The rewrite happens before the deliveries are reconciled, so a settled case
difference is not a revision. A rewrite that would give two rows the same KEY is
refused (exit `6` / `MergeConflictError`).

=== "CLI"

    Here the second delivery writes `BH01`'s `LOCA_TYPE`, and the `ABBR` row
    defining it, as `cp` where the first delivery and the standard list say
    `CP`. `--on-code-case standard` rewrites both, so the `ABBR` list carries
    one row and `BH01` is not reported as revised:

    ```bash
    --8<-- "cli/merge_on_code_case.sh:cmd"
    ```

    ```text
    --8<-- "cli/merge_on_code_case.out"
    ```

=== "Python"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    res = laterite.merge("phase1.ags", "phase2.ags", on_code_case="standard")
    ```

=== "Node"

    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    const res = merge(["phase1.ags", "phase2.ags"], { onCodeCase: "standard" });
    ```

## Rename a code

When a delivery uses its own code for something the standard list already
names (`SA` for a cable-percussion borehole the other delivery codes `CP`), the
two are different codes to merge: the later delivery's value wins each row it
appears in, and the `ABBR` list carries both. `recode` names the rewrite
outright, as `{heading: {from_code: to_code}}`. There is no default; without a
mapping nothing is renamed.

A mapping applies to the `ABBR` rows and to every `PA` cell under that heading.
It runs before the deliveries are reconciled, so a renamed code is not a
revision, and before `on_code_case`, so it wins for the codes it names. A
heading no delivery types `PA`, or a code no delivery uses under it, is refused
(exit `5` / `BadDictError`). So is a rewrite that would give two rows the same
KEY (exit `6` / `MergeConflictError`): renaming a `SAMP_TYPE` code, which is
part of `SAMP`'s KEY, can make one delivery's sample match the other's, and
merge refuses rather than lose a row.

=== "CLI"

    `--recode` takes a JSON file. Here the second delivery codes its
    cable-percussion boreholes `SA`; the mapping sends `SA` to `CP`, and the
    `ABBR` list before and after shows the rename:

    ```bash
    --8<-- "cli/merge_recode.sh:cmd"
    ```

    ```text
    --8<-- "cli/merge_recode.out"
    ```

=== "Python"

    <!-- doc-snippet: skip — both inputs are seeded from one fixture that never uses `SA`, so the mapping is refused -->
    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```python
    res = laterite.merge(
        "phase1.ags", "phase2.ags", recode={"LOCA_TYPE": {"SA": "CP"}},
    )
    ```

=== "Node"

    <!-- doc-snippet: skip — both inputs are seeded from one fixture that never uses `SA`, so the mapping is refused -->
    <!-- doc-code: skip — needs two separate deliveries, which the docs describe rather than ship; merging the fixture with itself would assert nothing -->
    ```js
    const res = merge(["phase1.ags", "phase2.ags"], {
      recode: { LOCA_TYPE: { SA: "CP" } },
    });
    ```

The accepted values of every mode above are the ones `lat merge --help` prints
(see the [CLI reference](../reference/cli.md)); an unknown value is refused
before any work, with a message listing them.

**One caveat everywhere:** identity is KEY-based, so _correcting_ a KEY value
(a `LOCA_ID` typo `BH1` → `BH01`) reads as a different row, not an edit; both
persist. Fix KEY typos in the source before merging.

See also: [Diff two revisions](./diff-revisions.md) ·
[Born typed](../concepts/born-typed.md)
