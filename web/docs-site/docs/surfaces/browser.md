# Browser

The **web app** is laterite's fourth surface: the whole engine compiled to
**WebAssembly** and run client-side. Drag an AGS4 file in and validate, repair,
explore, and convert it — with a guarantee no server-side tool can make:

> Runs entirely in your browser — your file never leaves your machine. No
> server, nothing uploaded.

That makes it safe for confidential ground-investigation data: nothing is
transmitted, so there's nothing to leak.

## The panes

| Pane         | What it does                                                                                 |
| ------------ | -------------------------------------------------------------------------------------------- |
| **Validate** | run the numbered-rules engine; findings grouped by rule, with the edition and severity tiers |
| **Fix**      | apply the automatic repairs and preview the before/after diff                                |
| **Explore**  | browse groups as typed tables, follow the KEY chain, chart values, run SQL across groups     |
| **Tools**    | conversions (AGS4 ↔ Excel), the anonymiser, and other one-off utilities                      |
| **Export**   | produce byte-faithful AGS4 (or Excel) back out                                               |

## Same engine, in the browser

The wasm build is the _same_ validator as the Python wheel, the Node addon, and
the DuckDB extension — the cross-surface compliance harness asserts the browser's
findings are byte-identical to the others. The only difference is _where_ it
runs: on your machine, in the tab, with no install.

!!! tip "When to reach for it"
Use the browser app for **ad-hoc, interactive** work — a quick check of a
delivery, a repair, a look at what's inside a file. Reach for
[Python](../learn/index.md), [Node](../node/index.md), or
[DuckDB](../duckdb/index.md) when you want to **automate** it in a pipeline.
