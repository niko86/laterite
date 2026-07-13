# Merge two deliveries

**Available in:** Python · Node · CLI · Browser

**When:** two deliveries of *one* project need to become one file — a phased
site investigation, or a resubmission that revises some boreholes and adds
others — and you want the engine to reconcile them, not a manual copy-paste.

Merging is **KEY-aware**, not a file concatenation. Rows are matched on each
group's dictionary **KEY headings**, so a re-sorted borehole list still merges
onto its prior self. Files merge in **argument order** — a later file wins a KEY
conflict — and the result is a **union**: a row present in only one file is kept
(silence is not deletion). Every surface reports the same audit: the per-row
*revisions* a later file made, plus any warnings.

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

    The summary names each **revision** — here the second delivery changed
    `PROJ_NAME`, matched on the `PROJ_ID` KEY `LAT-DEMO` — and flags that a revised
    parent (`PROJ`) has child groups worth re-checking. When two files declare a
    column with *different* AGS types, `merge` errors (exit `6`) rather than guess;
    pass `--lenient` to widen that column to `X` (text), keeping every raw value.

=== "Python"

    `laterite.merge(*sources, …)` takes two or more of anything `read` accepts
    (paths, text, bytes, `Ags4File` handles) and returns a `MergeResult` — the
    merged `bytes` plus the `warnings` and per-row `revisions` audit:

    ```python
    import laterite

    res = laterite.merge(
        "phase1.ags", "phase2.ags",
        tran_issue="3", tran_date="2024-01-15",
    )
    for rev in res.revisions:
        print(rev["group"], rev["key"], "changed", rev["changed"])
    res.save("merged.ags")
    ```

    A strict AGS-type clash between two files raises `MergeConflictError`; pass
    `lenient=True` to widen the column to `X` instead. `res.text` decodes the merged
    bytes, and `res.save(path)` writes them.

=== "Node"

    `merge(sources, opts)` runs the same leaf and returns the same
    `{ bytes, warnings, revisions, text }`:

    ```js
    import { merge } from "laterite";

    const res = merge(["phase1.ags", "phase2.ags"], {
      tranIssue: "3", tranDate: "2024-01-15",
    });
    for (const rev of res.revisions) {
      console.log(rev.group, rev.key, "changed", rev.changed);
    }
    ```

    A bare `string` is a **path** in Node, so pass a `Buffer` / `Uint8Array` when a
    delivery only exists in memory. A strict type clash throws `MergeConflictError`;
    `{ lenient: true }` widens the column to `X`.

=== "Browser"

    Open the [web app](../surfaces/browser.md)'s **Tools → Merge** tool, keep the
    file you already loaded as the base, and drop in the incoming delivery. The same
    reconciliation runs compiled to WebAssembly, entirely client-side: you get the
    per-row revision audit and download the merged `.ags` — neither file leaves your
    machine.

**One caveat everywhere:** identity is KEY-based, so *correcting* a KEY value
(a `LOCA_ID` typo `BH1` → `BH01`) reads as a different row, not an edit — both
persist. Fix KEY typos in the source before merging.

See also: [Diff two revisions](./diff-revisions.md) ·
[Born typed](../concepts/born-typed.md)
