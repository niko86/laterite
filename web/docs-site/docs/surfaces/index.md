# One engine, every stack

laterite is **one clean-room AGS4 engine** with several doors onto it. The rule
engine, the dictionary, and the born-typed decode are identical across all of
them — so you pick the surface that fits your workflow, not a different
validator.

## Pick your door

| Surface | Install | Best for |
|---|---|---|
| **[Python](python.md)** &nbsp;(`laterite`, PyPI) | `pip install laterite` | data pipelines, notebooks, the python-ags4 drop-in |
| **[Node](../node/index.md)** &nbsp;(`laterite`, npm) | `npm install laterite` | JS/TS tooling, servers, born-typed Arrow |
| **[DuckDB](../duckdb/index.md)** &nbsp;(`laterite_ags4`) | `INSTALL laterite_ags4 FROM community` | SQL-native analytics, querying files in place |
| **[CLI](cli.md)** &nbsp;(`lat-check`) | the shipped binary | CI gates, shell one-liners, `--fix` in place |
| **[Browser](browser.md)** &nbsp;(the web app) | open the app | drag-and-drop validate / fix / explore — nothing uploaded |

## What each door can do

The surfaces aren't equal — they're **different shapes**. Python is the fullest
library; Node mirrors it; DuckDB is a SQL idiom; the CLI is a CI tool; the
browser is a product. This grid is the honest map:

| Capability | Python | Node | DuckDB | CLI | Browser |
|---|:--:|:--:|:--:|:--:|:--:|
| **validate** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **read** — typed groups | ✅ | ✅ | ✅ | — | ✅ |
| **query** across groups | ✅ | ✅ | ✅ | — | ✅ |
| **build / emit** AGS4 | ✅ | ✅ | — | — | ✅ |
| **fix** | ✅ | ✅ | — | ✅ | ✅ |
| **diff** revisions | ✅ | ✅ | — | — | ✅ |
| **certify** (`.ags.idx`) | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Excel** ↔ AGS4 | ✅ | ✅ | — | — | ✅ |
| **transport** (pack / lock) | ✅ | ✅ | — | — | ✅ |
| **python-ags4 compat** | ✅ | — | — | — | — |

**✅ supported&nbsp;&nbsp;·&nbsp;&nbsp;○ planned&nbsp;&nbsp;·&nbsp;&nbsp;— by design**

Every capability is now either supported (`✅`) or a deliberate by-design blank
(`—`). The browser reaches **everything except the `python-ags4` drop-in** — the
former Excel, `certify` and `transport` gaps are all closed (transport encrypts
in a Web Worker with the same `zstd + age` envelope the CLI reads). *By design
(`—`):* the CLI is a validator (no read/query/build); DuckDB is a SQL surface (no
mutate-and-write ops); the `python-ags4` compat shim is a Python-only concern.

## The shared vocabulary

Every surface that *can* do a task uses the same verb, so knowledge transfers:

| Verb | What it does |
|---|---|
| `read` | load an AGS4 file → a handle whose groups are born-typed |
| `validate` | run the numbered-rules engine → a report (edition self-selected from `TRAN_AGS`) |
| `build_ags4` / `buildAgs4` | produce byte-faithful AGS4 from data frames or a typed graph |
| `save` · `.text` · `.bytes` | write it out |

See **[Validate a delivery — any surface](../validate-anywhere.md)** for the same
operation side-by-side across Python, Node, and SQL, with synced tabs.

## Not re-implementations — one core, proven

The surfaces don't each re-implement AGS4; they wrap the same Rust core. A
cross-surface **compliance harness** runs every read surface — Python, Node,
wasm, DuckDB, the `lat-check` CLI, and the python-ags4 incumbent — over a real
corpus and asserts they report **byte-identical findings**, as a per-PR gate plus
a monthly full-matrix report. So "the same verdict everywhere" is a tested
guarantee, not a claim — see [Cross-surface parity](../concepts/cross-surface-parity.md).
