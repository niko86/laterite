# One engine, every stack

laterite is **one clean-room AGS4 engine** with several doors onto it. The rule
engine, the dictionary, and the born-typed decode are identical across all of
them — so you pick the surface that fits your workflow, not a different
validator.

## Pick your door

| Surface                                                  | Install                                | Best for                                                  |
| -------------------------------------------------------- | -------------------------------------- | --------------------------------------------------------- |
| **[Python](python.md)** &nbsp;(`laterite`, PyPI)         | `pip install laterite`                 | data pipelines, notebooks, the python-ags4 drop-in        |
| **[Node](../node/index.md)** &nbsp;(`laterite`, npm)     | `npm install laterite`                 | JS/TS tooling, servers, born-typed Arrow                  |
| **[DuckDB](../duckdb/index.md)** &nbsp;(`laterite_ags4`) | `INSTALL laterite_ags4 FROM community` | SQL-native analytics, querying files in place             |
| **[CLI](cli.md)** &nbsp;(`lat`)                          | the shipped binary                     | CI gates, shell one-liners, `fix` in place                |
| **[Browser](../reference/wasm-api.md)** &nbsp;(`@laterite/ags4-wasm`, npm) | `npm i @laterite/ags4-wasm`            | validate / read / fix inside the page — nothing uploaded  |

Every one of these is **[in beta](../reference/support.md)**. The Rust crate
(`cargo add laterite`) is the one surface that isn't — it runs the same engine, but
it is not yet at parity with these five.

The **[web app](browser.md)** is built on the browser package and nothing else, so
it is a worked example of that door rather than a sixth one — go and use it, but you
don't install it.

## What each door can do

The surfaces aren't equal — they're **different shapes**. Python is the fullest
library; Node mirrors it; DuckDB is a SQL idiom; the CLI is a CI tool; the
browser package runs the engine in the page. This grid is the honest map:

| Capability                  | Python | Node | DuckDB | CLI | Browser |
| --------------------------- | :----: | :--: | :----: | :-: | :-----: |
| **validate**                |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **read** — a group's rows   |   ✅   |  ✅  |   ✅   | ✅  |   ✅    |
| **query** across groups     |   ✅   |  ✅  |   ✅   |  —  |   ✅    |
| **build / emit** AGS4       |   ✅   |  ✅  |   —    |  —  |   ✅    |
| **fix**                     |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **diff** revisions          |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **certify** (`.ags.idx`)    |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **Excel** ↔ AGS4            |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **transport** (pack / lock) |   ✅   |  ✅  |   —    | ✅  |   ✅    |
| **python-ags4 compat**      |   ✅   |  —   |   —    |  —  |    —    |

**✅ supported&nbsp;&nbsp;·&nbsp;&nbsp;○ planned&nbsp;&nbsp;·&nbsp;&nbsp;— by design**

Every capability is now either supported (`✅`) or a deliberate by-design blank
(`—`). The browser reaches **everything except the `python-ags4` drop-in** — the
former Excel, `certify` and `transport` gaps are all closed (transport encrypts
in a Web Worker with the same `zstd + age` envelope the CLI reads). _By design
(`—`):_ the CLI is a validator + inspect/repair tool (no query/build); DuckDB is
a **read-only** SQL reader — it queries and joins but doesn't validate, certify,
or mutate (validation and certification live in the CLI + library; the extension
only _consumes_ an externally-minted `.ags.idx`); the `python-ags4` compat shim
is a Python-only concern.

## The shared vocabulary

Every surface that _can_ do a task uses the same verb, so knowledge transfers:

| Verb                        | What it does                                                                     |
| --------------------------- | -------------------------------------------------------------------------------- |
| `read`                      | load an AGS4 file → a handle whose groups are born-typed                         |
| `validate`                  | run the numbered-rules engine → a report (edition self-selected from `TRAN_AGS`) |
| `build_ags4` / `buildAgs4`  | produce byte-faithful AGS4 from data frames or a typed graph                     |
| `save` · `.text` · `.bytes` | write it out                                                                     |

See **[Validate a delivery](../cookbook/validate-a-delivery.md)** for the same
operation side-by-side across Python, Node, DuckDB and the CLI, with synced tabs.
The browser package does it too — its worked examples live in the
[browser API reference](../reference/wasm-api.md).

## Not re-implementations — one core, proven

The surfaces don't each re-implement AGS4; they wrap the same Rust core. A
cross-surface **compliance harness** runs every read surface — the Rust core,
Python, Node, wasm, DuckDB, and the python-ags4 incumbent — over a real
corpus and asserts they report **byte-identical findings**, as a per-PR gate plus
a monthly full-matrix report. So "the same verdict everywhere" is a tested
guarantee, not a claim — see [Cross-surface parity](../concepts/cross-surface-parity.md).
<!-- cadence: compliance -->
<!-- cadence: compliance-report -->
