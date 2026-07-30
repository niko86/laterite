# Build AGS4 from frames

**Available in:** Python · Node · Browser

**What / when:** you have your data as per-group tables (one per group, columns named
for the AGS headings) and want an AGS4 file back — optionally with the
`UNIT` / `TYPE` boilerplate derived for you and `TRAN` stamped from your own
transmission details.

=== "Python"

    ```python
    --8<-- "python/ex09a_build_from_frames.py"
    ```

    ```text
    groups: ['PROJ', 'LOCA']
    findings: 3
    ```

    `build_ags4` takes a `{code: frame}` mapping — each frame's columns *are* the AGS
    headings for that group — and returns a `BuildResult` containing exactly the
    groups you supplied. AGS4 also mandates the metadata catalogs, which your frames
    don't carry, so they are **reported, not invented**: the three findings are Rules
    14 (`TRAN`), 15 (`UNIT`) and 17 (`TYPE`). The result carries the file three ways:

    - `res.text` — the AGS4 as a `str` (in memory)
    - `res.bytes` — the byte-faithful encoding (what `read(data=…)` consumes above)
    - `res.save("out.ags")` — persist to disk

    **To get the catalogs:** pass `synthesise_metadata=True`. `UNIT` and `TYPE` are
    derived from your columns and `ABBR` from the standard table when `PA` codes are
    used. `PROJ`, `DICT` and `TRAN` are *never* synthesised: a project identity, a
    schema extension and a record of transmission are authorial facts. Inventing a
    `DICT` parent would turn a loud Rule 18 error into a silent false statement Rule
    10's relational checks then trust; inventing a `TRAN` would *satisfy* Rule 14
    while asserting a transmission that never happened.

    **To get a `TRAN`:** state it — `tran=TranStamp(issue=…, date=…, producer=…,
    recipient=…, status=…)`. All five are required together because all five are
    REQUIRED headings; the dataclass enforces that at your call site rather than
    letting a half-stamp become a Rule 10b finding. Pass nothing and Rule 14
    reports the gap instead.

    **Gotcha:** synthesis is independent of `mode`, and only `mode="autofix"` (the
    default) honours it. `mode="report"` emits unmodified and hands you the findings;
    `mode="strict"` is a hard gate that *rejects* the build outright if the output
    violates any error-severity rule — so a data-only build under `strict` raises
    rather than emitting. The emitter also writes **only the headings you supply**,
    never inventing data columns, so a sparse frame builds clean rather than padding
    out the full dictionary.

=== "Node"

    ```js
    --8<-- "node/ex09a_build_from_frames.mjs"
    ```

    ```text
    groups: [ 'PROJ', 'LOCA' ]
    findings: 3
    ```

    `buildAgs4` takes a `Map` (or array) of `[code, rows]` entries — each row a
    plain object whose **keys are the AGS headings** — or an arrow-js `Table` per
    group. Group order is preserved, so put `PROJ` first. It returns the same
    `BuildResult` as Python: `res.bytes` / `res.text` carry the document and
    `res.findings` is the residual the mode couldn't clear. Pass
    `{ synthesiseMetadata: true }` to derive `UNIT`/`TYPE` (and `ABBR` for `PA`
    codes) and `tran: { issue, date, producer, recipient, status }` to stamp a
    `TRAN`; without them those gaps are reported as Rules 14/15/17. No DuckDB
    peer needed — emit is pure.

=== "Browser"

    Open the [web app](../surfaces/browser.md)'s **Export** pane: assemble or
    paste your per-group data and export an AGS4 file. The same emitter runs
    compiled to WebAssembly, and nothing is uploaded to build it. Direct wasm
    callers take `synthesise_metadata` on `build_ags4` / `build_ags4_ipc` — the
    browser twin of the Python and Node flags.

Every door runs the same emitter over the same dictionary, and metadata synthesis
is opt-in on all of them — `synthesise_metadata=` in Python, `{ synthesiseMetadata }`
in Node, `synthesise_metadata` in the browser build. Without it you get exactly the
groups you supplied, plus findings naming the catalogs you didn't.

See also: [Build from a typed graph](./build-from-typed-graph.md) · [Produce AGS4](../learn/produce.md)
