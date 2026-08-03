# Browser

The **web app** is laterite's fourth surface: the whole engine compiled to
**WebAssembly** and run client-side. Drag an AGS4 file in and validate, repair,
explore, and convert it — with a guarantee no server-side tool can make:

> Runs entirely in your browser — your file never leaves your machine. No
> server, nothing uploaded.

That makes it safe for confidential ground-investigation data: nothing is
transmitted, so there's nothing to leak.

It is also a **showcase**, not a product. Everything below runs on
`@laterite/ags4-wasm`, the same package you can install — so the app doubles as
a worked example of integrating it. The practices it uses, and why, are in
[Browser API (wasm)](../reference/wasm-api.md).

## The panes

| Pane         | What it does                                                                                 |
| ------------ | -------------------------------------------------------------------------------------------- |
| **Validate** | run the numbered-rules engine; findings grouped by rule, with the edition and severity tiers |
| **Fix**      | apply the automatic repairs and preview the before/after diff                                |
| **Explore**  | browse groups as typed tables, follow the KEY chain, chart values, run SQL across groups     |
| **Tools**    | conversions (AGS4 ↔ Excel), merge, transport, the rule catalogue, the anonymiser             |
| **Export**   | produce byte-faithful AGS4 (or Excel) back out                                               |

## What each pane gives you

**Validate** — findings arrive grouped by rule, with the resolved dictionary
edition and the [severity tiers](../concepts/severity-tiers.md). The same file
carries straight into **Fix** or **Explore** without reloading. A clean file
offers its [`.ags.idx` certificate](../concepts/certificate-lifecycle.md) as a
download; because the cert format is shared, a file certified here opens on the
fast path in Python, Node or DuckDB.

**Fix** — computes the available repairs, shows a before/after diff so you can
review each one, and hands back the repaired file. The same safe-fix engine every
other surface runs.

**Explore** — every group as the same [born-typed](../concepts/born-typed.md)
table: sortable columns, numeric filters that compare as numbers, and charts over
the numeric columns. Follow the KEY chain to parent and child groups, or pick a
`LOCA` and see its samples and tests fan out together — one borehole's record set
without writing a query. There is also a SQL box: DuckDB-wasm fed by the wasm
reader's Arrow output, so the same
`SAMP s JOIN LOCA l USING (LOCA_ID)` you would write anywhere else runs over the
file you dropped in.

**Tools** — the rule catalogue (all 27 numbered rules with titles, severities and
fixable flags; a finding links back to its explainer), **Merge** (keep the loaded
file as the base, drop in the incoming delivery, get the per-row revision audit
and choose how to settle a type clash), the **transport** lock/unlock round-trip
(byte-compatible with `transport.unlock` elsewhere), Excel conversion, and the
anonymiser.

**Export** — assemble or paste per-group data and get a byte-faithful AGS4 file
back. Direct wasm callers get `synthesise_metadata` on `build_ags4` /
`build_ags4_ipc` to derive the `UNIT`/`TYPE` catalogs, plus the five `tran_*`
arguments to stamp a `TRAN` — the browser twin of the Python and Node flags.

## Same engine, in the browser

The wasm build is the _same_ validator as the Python wheel, the Node addon, and
the DuckDB extension — the cross-surface compliance harness asserts the browser's
findings are byte-identical to the others. The only difference is _where_ it
runs: on your machine, in the tab, with no install.

!!! tip "When to reach for it"
    Use the browser app for **ad-hoc, interactive** work — a quick check of a
    delivery, a repair, a look at what's inside a file. Reach for
    [Python](../learn/index.md), [Node](../node/index.md), or
    [DuckDB](../duckdb/index.md) when you want to **automate** it in a pipeline, and
    [the wasm package](../reference/wasm-api.md) when you want this engine inside
    your own browser app.
