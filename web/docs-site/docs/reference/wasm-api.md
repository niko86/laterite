# Browser API (wasm)

`@laterite/ags4-wasm` is the engine compiled to WebAssembly: the same validator,
reader, repairer and emitter the [Python wheel](./api.md), the
[Node addon](./node-api.md) and the [DuckDB extension](./duckdb-functions.md)
run, in a page, with no server.

```bash
npm i @laterite/ags4-wasm
```

The [web app](../surfaces/browser.md) is built on this package and nothing else,
which makes it a worked example rather than a demo — everything below is a
practice taken from its source, with the reason it is done that way.

## Init once, and await that one promise

```js
--8<-- "wasm/ex01_init.mjs"
```

```text
--8<-- "wasm/ex01_init.out"
```

Two things here are easy to get wrong and expensive to debug.

**Pass `module_or_path` explicitly.** Omitted, the glue falls back to fetching
relative to `import.meta.url` — which breaks the moment your app is served from a
non-root `base`. The app hit exactly that.

**Init once, at module scope, and let everything await the same promise.** Every
export throws before instantiation, so the alternative is a live-before-ready
race that only shows up when a user acts fast. Awaiting a shared promise makes
early calls queue instead.

!!! note "The one difference between these examples and your app"
    Everything on this page runs under Node so it can be tested, and Node has no
    `fetch` for a file path — so `module_or_path` gets the bytes. In a bundler it
    gets an asset URL: `import wasmUrl from "@laterite/ags4-wasm/ags4_wasm_bg.wasm?url"`.
    The **call is the same**; only that argument differs.

## Validate, and read severity correctly

```js
--8<-- "wasm/ex02_validate.mjs"
```

```text
--8<-- "wasm/ex02_validate.out"
```

**An absent `severity` means `error`.** The engine omits the field rather than
spelling it out, so the default you write is load-bearing — and it belongs in one
resolver that everything calls. The app defaulted to `"warning"` at five separate
sites, which silently reclassified every error in the browser: the summary banner
counted errors as warnings, and the severity filter hid them from the "error"
selection while showing them under "warning".

Note also `report.error` versus `report.findings`. A parseable file with problems
returns findings and a `null` error; only an input that is not AGS4 at all comes
back with `error` set.

## Take the types from the package

```ts
import type {
  FindingDto,
  RuleGroup,
  ValidationReport,
} from "@laterite/ags4-wasm";
```

`import type` is erased at compile time, so this costs **no runtime import** — a
module that only needs the shapes stays free of the wasm entirely, which is what
lets the app share types between its main thread and its worker.

Do not hand-mirror them. The app used to re-declare these interfaces because
wasm-bindgen once typed the returns as `any`; the mirror was wrong about
`severity`, and a mirror can only ever be right by accident. Since 0.9.0 the
crate publishes every result shape and there is no `any` left in the `.d.ts`.

## Read: the dataset is a handle, not a copy

```js
--8<-- "wasm/ex03_read_arrow.mjs"
```

```text
--8<-- "wasm/ex03_read_arrow.out"
```

`read()` returns a `ParsedDataset` that lives in wasm memory. Each group's Arrow
batch is built **lazily** by `arrow_ipc()` and dropped on return, so the dataset
has to outlive every pull — hold it rather than chaining off the call — and
**free it before the next parse**, or wasm memory holds two datasets at once.
`using dataset = read(…)` does that for you where `Symbol.dispose` is supported.

`keys: true` prepends the content-addressed `_id` / `_parent_id` columns — the
same UUIDv8s the wheel, Node and the DuckDB extension produce, from the one
shared keychain. That is what makes a cross-group join resolve when you feed
these batches into duckdb-wasm; leave it off for a plain typed frame.

## Repair: propose, then apply

```js
--8<-- "wasm/ex04_fix.mjs"
```

```text
--8<-- "wasm/ex04_fix.out"
```

`compute_fixes` and `apply_fixes` are separate calls so you can show the user
what will change before anything is rewritten — each fix carries its `kind`, the
`rule` it answers, the `line` and a `risk`. And because `apply_fixes` takes the
ledger back, you can hand it a **subset**: whatever the user actually ticked.

Notice what repair does _not_ do. Rule 4 is gone; Rules 13/14/15/17 remain,
because those are the mandatory catalogs this fragment never had and no repair
will invent them. "Fixed" means the defects a machine can settle are settled — it
does not mean valid.

## Produce AGS4

```js
--8<-- "wasm/ex05_build.mjs"
```

```text
--8<-- "wasm/ex05_build.out"
```

`synthesiseMetadata` derives `UNIT` and `TYPE` from your columns, and `ABBR` when
`PA` codes are used. It will not invent `PROJ`, `DICT` or `TRAN`, and that
asymmetry is deliberate: a stub `TRAN` reading `TBC` / `1900-01-01` still
_satisfies_ Rule 14, so a recipient could not tell an invented transmission
record from a real one and nothing downstream would flag it. Who produced a file,
for whom, when and at what status is knowable only to you — so state it via
`tran`, or let Rule 14 report the gap.

## Keep it off the main thread

The engine is synchronous and uninterruptible once entered. A pathologically
dirty file — millions of findings — will hold the thread for tens of seconds, so
the app puts every wasm call in a worker and talks to it over a small request /
response protocol where each message carries a monotonic `id`.

The consequence is worth stating plainly: **"cancel" can only mean discard the
stale result**, never abort mid-rule. But because the work happens off the main
thread, a superseded run never blocks the next paint — the UI stays live and
simply ignores the answer when it arrives.

Transfer the file's `ArrayBuffer` to the worker rather than copying it; the
worker views it as bytes and the main thread does not need it back.

## Related

[Browser (web app)](../surfaces/browser.md) ·
[One engine, many doors](../concepts/one-engine-many-doors.md) ·
[Cross-surface parity](../concepts/cross-surface-parity.md)
