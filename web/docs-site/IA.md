# laterite docs — information architecture (working plan, #201)

> Not published (outside `docs/`). The decided layout for the multi-surface site.

## Decision: extend MkDocs (not a new stack)

Material for MkDocs already does everything the multi-stack pitch needs —
including synced code tabs (`content.tabs.link`) — and keeps the `mkdocstrings`
Python autodoc. A Starlight prototype was built + compared, then discarded.

## Principle

**Write each shared task once (surface-tabbed); give each surface one focused
home for its entry + unique parts; let a capability matrix make the inequality
legible.** No surface is forced into another's mould; nothing shared is
duplicated.

## Structure (Diátaxis + a surfaces spine)

- **Home** — the pitch + the capability matrix + three doors (Learn / Surfaces / Cookbook).
- **Learn** — one Python-led tutorial (Python = the best teaching surface); each step points to the Cookbook for "same in your surface".
- **Cookbook** — task recipes, surface-tabbed for shared tasks; a per-recipe "Available in" badge; Python-only recipes (compat / Excel / AgsQuery) stay Python.
- **Surfaces** — one focused page per door: `index` (hub + matrix), `python`, `node`, `duckdb`, `cli`, `browser`. Entry + unique parts, links into the Cookbook.
- **Concepts** — surface-agnostic _why_ (born-typed, fluent, certificate, keys, dictionary selection, severity, dependency shape, **cross-surface parity**).
- **Reference** — per-surface API sized to each: CLI flags, Python API (autodoc), **Node API** (new), **DuckDB functions** (new), AGS types, group catalogue, **AGS4 rules + O-N catalogue** (new), cheatsheet.

## The capability matrix — three states

`✅` supported · `○` planned · `—` by design. The matrix doubles as a **parity
backlog**: every `○` is a real, tracked gap.

Target state (owner decision: Node + Browser reach parity except python-ags4
compat; DuckDB + CLI stay by-design; web transport is a by-design browser blank):

| Capability            | Python | Node | DuckDB | CLI | Browser |
| --------------------- | :----: | :--: | :----: | :-: | :-----: |
| validate              |   ✅   |  ✅  |   ✅   | ✅  |   ✅    |
| read (typed)          |   ✅   |  ✅  |   ✅   |  —  |   ✅    |
| query                 |   ✅   |  ✅  |   ✅   |  —  |   ✅    |
| build / emit          |   ✅   |  ✅  |   —    |  —  |   ✅    |
| fix                   |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| diff                  |   ✅   |  ✅  |   —    |  —  |   ✅    |
| certify               |   ✅   |  ✅  |   ✅   | ✅  |    ○    |
| Excel ↔ AGS4          |   ✅   |  ○   |   —    |  —  |    ○    |
| transport (pack/lock) |   ✅   |  ✅  |   —    |  —  |    —    |
| python-ags4 compat    |   ✅   |  —   |   —    |  —  |    —    |

## Plug-list (the `○` cells — docs ship now, plugs on their own cadence)

1. ~~**Node Excel I/O** — bind `laterite-ags4-excel`~~ — **shipped (#358 / PR #361):** `toExcel` / `fromExcel`.
2. **Browser Excel I/O** — wasm-bind `laterite-ags4-excel` (or a JS xlsx lib). #359, relates to #295.
3. **Browser certify** — mint `.ags.idx` client-side + download; wasm has the `index` code. #360, relates to #295.

Deferred/by-design: web transport (`age`→`getrandom` isn't wasm-clean; encryption
is a pipeline concern) — a by-design browser blank.

## Open follow-ups

- Retrofit the existing Cookbook recipes with surface tabs (incremental).
- File the three `○` items as issues (or attach to #295) so the matrix cells link.
- Reference pages for Node API, DuckDB functions, the O-N/rules catalogue.
