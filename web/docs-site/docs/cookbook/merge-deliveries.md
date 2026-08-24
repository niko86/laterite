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
digit you wrote is ever changed, and taking the **maximum** precision is the only
direction that cannot destroy data. That also makes the result independent of
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

**One caveat everywhere:** identity is KEY-based, so _correcting_ a KEY value
(a `LOCA_ID` typo `BH1` → `BH01`) reads as a different row, not an edit; both
persist. Fix KEY typos in the source before merging.

See also: [Diff two revisions](./diff-revisions.md) ·
[Born typed](../concepts/born-typed.md)
