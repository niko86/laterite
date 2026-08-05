---
type: concept
title: modality register
status: drafted
tags: [concept, architecture, api-parity]
ags_editions: []
repo_refs:
  register: "repo:modality.json"
  generator: "repo:tools/gen_modality.py"
  gate: "repo:packages/laterite/tests/test_modality_parity.py"
related: [crate-map, parity-model, pyo3-boundary, tech-stack-wasm, docs-site, surface-census, cert-trust-v2, start-here]
sources: []
---

# modality register

> **Generated** by `tools/gen_modality.py` from `modality.json` — do not hand-edit.
> Change the SSOT (prose lives in its `preamble`) and re-run the generator.

## Definition

Every laterite capability is one engine behind several doors. A 'modality' is the I/O *form* a capability is offered in — an input door (path / text / bytes / file-like / handle / stdin / cert) or an output door (file / bytes / text / handle / value / table / stdout). The behavioural-knob parity gates (test_free_chained_parity, test_cross_surface_parity) compare pairs that exist on BOTH sides and STRIP the modality-bearing params before comparing, so nothing there detects a capability offered in fewer forms on one surface — an *absence*, not a *drift*. This register is that missing axis: one cell per (capability, surface, spelling), each form tri-stated present|absent|divergent, each absence verdicted gap|by-design with a reason. It is the find-only deliverable AND the by-design allowlist the standing gate (test_modality_parity) checks reflected reality against. A second gate, gen_modality.py --check, holds the rendered page against this SSOT — the two guard different axes. The sibling baseline (which surface offers the richest form-set for a capability) is COMPUTED by the generator, never stored — a stored baseline is the multi-source-of-truth class #181 exists to kill.

This register is the **I/O-form** axis of cross-surface parity — does a capability exist in this SHAPE on this surface. [[surface-census]] is the **verb/table** axis of the same problem — does it exist AT ALL on this surface. Both share the reflect-don't-hand-list discipline (a form or verb list authored by hand is just a fourth thing to drift), and both exist because a *value*-comparison gate structurally cannot see an absence: feed identical input through every surface and diff the outputs, and a door that was never built produces no output to diff.

## Findings backlog (find-only — fixes are follow-ups)

- **🔴 P1** (0): —
- **🟠 P2** (4): read/cli (in.stdin); validate/cli (in.stdin); build/browser (in.text); emit/browser (out.bytes)
- **🟡 P3** (4): read/rust (in.file-like); validate/rust (in.file-like); read_typed/node (out.handle); read-output-view/python (out.table)
- **⚪ by-design** (12): intentional absences, rationale in each cell below.

## Excluded axes

- **async-vs-sync** — Node sql()/at() are Promise-based only because @duckdb/node-api is async; the returned form is identical. A calling convention, not a modality.
- **streaming-input** — raw .ags has no index/footer, so a full read is format-inherent, not a per-surface choice. Streaming-OUTPUT (DuckDB scan vector-chunks, wasm lazy arrow_ipc) is already captured by the table/bytes output forms.
- **frame-shaping-knobs** — keys / backend=polars|pandas / {arrow:true} are sub-form refinements of a present form — kept as knob-parity in test_cross_surface_parity. (xn='numeric', a Python-only output view with no sibling and no knob-gate, IS carried below as a documented divergence.)
- **output-value-agreement** — This register's grid asks whether a capability is OFFERED in a form; it is silent on whether two present forms agree on the ANSWER for identical bytes. That gap is exactly where an edition-resolution bug lived — the O-42 `guard_4_0_4` content guard ran on a path read (`check_file_with_dict`) but was skipped by laterite-py/laterite-node/wasm's hand-assembled bytes/text branches, so the same file resolved to a different dictionary edition by modality — until closed 2026-07-14 by a same-bytes-same-verdict gate: `repo:packages/laterite/tests/test_modality_output_parity.py` / `repo:rust-packages/laterite-node/test/modality-output-parity.test.ts`. See [[cert-trust-v2]] for the fix and the general 'surfaces reach past the door' pattern it closes a second instance of.

## The register

### read — Parse AGS4 into a read handle.

*Offered anywhere — in: bytes, cert, file-like, path, text · out: handle, stdout, table, value*

**Input**

| surface (spelling) | path | text | bytes | file-like | stdin | cert |
|---|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |   | ✓ |
| node (free) | ✓ | ✓ | ✓ | — |   | ✓ |
| rust (chained) | ✓ | ✓ | ✓ | — |   | ✓ |
| cli (free) | ✓ |   |   |   | — |   |
| browser (free) | — |   | ✓ |   |   | — |
| duckdb (free) | ✓ | ✓ |   |   |   | ✓ |

**Output**

| surface (spelling) | handle | value | table | stdout |
|---|---|---|---|---|
| python (free) | ✓ |   |   |   |
| node (free) | ✓ |   |   |   |
| rust (chained) | ✓ |   |   |   |
| cli (free) |   | ✓ |   | ✓ |
| browser (free) | ✓ |   |   |   |
| duckdb (free) |   |   | ✓ |   |

_Findings:_
- ⚪ by-design · **node** in.file-like — Node has no io.BytesIO-style universal file-like; a caller reads the stream to a Buffer and passes bytes.
- 🟡 P3 · **rust** in.file-like — Above the facade floor — node does not offer it either — so the floor does not owe it. Recorded anyway because node's by-design reason ('no universal Node file-like') cannot be borrowed here: Rust has one, `impl std::io::Read`. Cheap to add; the floor is a minimum, not a cap on what gets built.
- 🟠 P2 · **cli** in.stdin `cli-stdin` — no '-'/stdin door — a piped .ags must be spooled to a temp file first. The shell surface's bytes form IS stdin.
- ⚪ by-design · **browser** in.path — no filesystem in the browser — a File/upload is read to a Uint8Array, so bytes is the only sensible input door.

_Notes:_
- _rust_: Partial. `text` is `read_bytes(s.as_bytes())` behind a name. The `cert` door (2026-08-05, phase 4b) is not the python/node one: there a cert parked on the handle lets a later `.validate()` skip the engine, which this facade has no `Document::validate()` to do. Here it carries the BYTE INDEX, so `read(f).index(c).only(["LOCA"])` parses that group out of its byte range and never looks at the rest of the file — `Document::sliced()` says whether it did. It declines rather than risks: a stale cert, a group the index places in two sections, or a transcode that moved the offsets all fall back to the whole-file parse.

### validate — Run the numbered AGS4 rules and return a verdict.

*Offered anywhere — in: bytes, file-like, path, text · out: stdout, value*

**Input**

| surface (spelling) | path | text | bytes | file-like | stdin |
|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |   |
| node (free) | ✓ | ✓ | ✓ | — |   |
| rust (chained) | ✓ | ✓ | ✓ | — |   |
| cli (free) | ✓ |   |   |   | — |
| browser (free) | — |   | ✓ |   |   |

**Output**

| surface (spelling) | value | stdout |
|---|---|---|
| python (free) | ✓ |   |
| node (free) | ✓ |   |
| rust (chained) | ✓ |   |
| cli (free) |   | ✓ |
| browser (free) | ✓ |   |

_Findings:_
- ⚪ by-design · **node** in.file-like — same as read — no universal Node file-like; pass bytes.
- 🟡 P3 · **rust** in.file-like — Above the facade floor — node does not offer it either — so the floor does not owe it. Recorded anyway because node's by-design reason ('no universal Node file-like') cannot be borrowed here: Rust has one, `impl std::io::Read`. Cheap to add; the floor is a minimum, not a cap on what gets built.
- 🟠 P2 · **cli** in.stdin `cli-stdin` — no '-'/stdin door; a piped file must be spooled to disk first.
- ⚪ by-design · **browser** in.path — no filesystem in the browser.

_Notes:_
- _python_: validate exposes only the positional source sniff + text= keyword — no explicit path=/data= keyword doors like read/fix. All input FORMS are still reachable via the sniff (_resolve_source accepts path/bytes/file-like), so this is a keyword-ergonomics inconsistency, NOT a lost modality — recorded here, deliberately not a gap.
- _rust_: Partial. `text` is the same trivial door as read's.

### fix — Mechanically repair AGS4.

*Offered anywhere — in: bytes, file-like, path, text · out: bytes, file, value*

*Below the facade floor — the Rust crate does not yet offer in: bytes, path, text · out: file, value, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | path | text | bytes | file-like |
|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ | ✓ | ✓ |   |
| browser (free) |   |   | ✓ |   |
| rust (absent) |   |   |   |   |

**Output**

| surface (spelling) | file | bytes | value |
|---|---|---|---|
| python (free) | ✓ |   | ✓ |
| node (free) | ✓ |   | ✓ |
| browser (free) |   | ✓ | ✓ |
| rust (absent) |   |   |   |

_Notes:_
- _node_: #394 added inPlace/out write-back (the out.file form) + only/exclude rule selection — the latter shrank the test_cross_surface_parity _MATRIX allowlist to empty. Rule labels are typed as FixableRule, drift-gated to Python/the engine (test_typed_choices).
- _browser_: the browser deliberately SPLITS fix into compute_fixes (returns the Fix[] proposal for the UI to preview — a value form) and apply_fixes (returns the repaired bytes). The library surfaces one-shot fix() and offer no dry-run Fix[] preview form; whether to add one is P3 verb-decomposition (fix-dry-run-split), tracked in the backlog, not a browser defect.
- _rust_: Decided 2026-08-04: ADD. No new dependency — `compute_fixes`/`apply_fixes` are in laterite-ags4-validator, already a facade dep.

### build — Construct valid AGS4 from caller-supplied data (build_ags4).

*Offered anywhere — in: bytes, handle, text, value · out: bytes, value*

*Below the facade floor — the Rust crate does not yet offer in: handle, value · out: value, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | text | bytes | handle | value |
|---|---|---|---|---|
| python (free) |   |   | ✓ | ✓ |
| node (free) |   |   | ✓ | ✓ |
| browser (free) | ≈ | ✓ |   |   |
| rust (absent) |   |   |   |   |

**Output**

| surface (spelling) | bytes | value |
|---|---|---|
| python (free) |   | ✓ |
| node (free) |   | ✓ |
| browser (free) | ✓ |   |
| rust (absent) |   |   |

_Findings:_
- 🟠 P2 · **browser** in.text `wasm-build-text-outlier` — build_ags4 takes a JSON-TEXT groups payload — the lone text-in build door across all surfaces (Python/Node take a typed-graph root or (code, frame) rows). Bytes-in already exists as build_ags4_ipc, so the reconciliation is the JSON-text outlier, NOT adding a bytes door.

_Notes:_
- _rust_: Decided 2026-08-04: ADD. No new dependency — laterite-ags4-emit is already a facade dep, and `Document` already has `push_row`/`set_cell`; what is missing is the one-call door.

### diff — Compare two AGS4 revisions.

*Offered anywhere — in: bytes, file-like, handle, path, text · out: value*

*Below the facade floor — the Rust crate does not yet offer in: bytes, handle, path · out: value, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | path | text | bytes | file-like | handle |
|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ |   | ✓ | — | ✓ |
| browser (free) |   |   | ✓ |   |   |
| rust (absent) |   |   |   |   |   |

**Output**

| surface (spelling) | value |
|---|---|
| python (free) | ✓ |
| node (free) | ✓ |
| browser (free) | ✓ |
| rust (absent) |   |

_Findings:_
- ⚪ by-design · **node** in.file-like — no universal Node file-like; DiffSource is string|Uint8Array|Ags4File.

_Notes:_
- _rust_: Absent. Held for 0.2 by the published plan (dec-rust-api-crates-io): 0.1.x scope is read/validate/write and this is additive, so nothing has to move to admit it.

### merge — Reconcile N AGS4 deliveries of one project into one file.

*Offered anywhere — in: bytes, file-like, handle, path, text · out: file, stdout, value*

*Below the facade floor — the Rust crate does not yet offer in: bytes, handle, path · out: value, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | path | text | bytes | file-like | handle |
|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ |   | ✓ | — | ✓ |
| rust (absent) |   |   |   |   |   |
| cli (free) | ✓ |   |   |   |   |
| browser (free) |   |   | ✓ |   |   |

**Output**

| surface (spelling) | file | value | stdout |
|---|---|---|---|
| python (free) | ✓ | ✓ |   |
| node (free) | — | ✓ |   |
| rust (absent) |   |   |   |
| cli (free) | ✓ |   | ✓ |
| browser (free) | ✓ | ✓ |   |

_Findings:_
- ⚪ by-design · **node** in.file-like — no universal Node file-like; MergeSource is string|Uint8Array|Ags4File.
- ⚪ by-design · **node** out.file — Node merge returns a MergeResult carrying the merged bytes; the caller writes with fs (return-only, matching diff).

_Notes:_
- _rust_: Absent. Held for 0.2 by the published plan (dec-rust-api-crates-io): 0.1.x scope is read/validate/write and this is additive, so nothing has to move to admit it.

### censor — Anonymise an AGS4 file — scrub the classified sensitive cells (pseudonymise IDs, hash PROJ_ID, blank coordinates, tokenise names, strip free-text [units]) before sharing. Browser-only among shipped surfaces: the SAME laterite-ags4-censor engine backs the private laterite-ags4-corpus-qa `censor` dev tool, which is not a shipped py/node/cli API, so its cross-surface EXISTENCE is a surface-census matter, not a modality one.

*Offered anywhere — in: bytes · out: file, value*

**Input**

| surface (spelling) | bytes |
|---|---|
| browser (free) | ✓ |
| rust (absent) |   |

**Output**

| surface (spelling) | file | value |
|---|---|---|
| browser (free) | ✓ | ✓ |
| rust (absent) |   |   |

_Notes:_
- _rust_: Absent. Browser-only among shipped surfaces, so the floor is empty here — there is no python/node pair to be the least of.

### certify — Mint an .ags.idx validity certificate (cert as OUTPUT).

*Offered anywhere — in: bytes, handle, path · out: bytes, file, text*

**Input**

| surface (spelling) | path | bytes | handle |
|---|---|---|---|
| python (chained) |   |   | ✓ |
| node (chained) |   |   | ✓ |
| rust (chained) |   |   | ✓ |
| cli (free) | ✓ |   |   |
| browser (free) |   | ✓ |   |

**Output**

| surface (spelling) | file | bytes | text |
|---|---|---|---|
| python (chained) | ✓ | ✓ |   |
| node (chained) | ✓ | ✓ |   |
| rust (chained) | ✓ | ✓ |   |
| cli (free) | ✓ |   |   |
| browser (free) | — |   | ✓ |

_Findings:_
- ⚪ by-design · **browser** out.file `certify-bytes-output` — certify returns the cert as a String because the browser has no filesystem — a PRESENT capability, not a shape gap. It proved the in-memory shape that Python/Node now match via certify_bytes/certifyBytes (#390).

_Notes:_
- _python_: certify_bytes() (#390) returns the .ags.idx bytes in memory — the certify analog of transport.lock_bytes; same cert as certify() (bar the mint timestamp), so it interops with read(index=)/--index/the browser.
- _node_: certifyBytes() (#390) mirrors laterite-py's certify_bytes — same in-memory cert form.
- _rust_: Added 2026-08-05 (phase 4b). Mints over the ORIGINAL source bytes, before any transcode, with the encoding recorded alongside — the same rule laterite-py follows. Note the engine limit this exposes: bytes that are not UTF-8 validate but cannot be certified at all, because a certificate carries byte offsets the engine will not record into bytes it cannot address. That holds on every surface.

### cert-input — Consume an .ags.idx certificate to skip revalidation (cert as INPUT).

*Offered anywhere — in: cert · out: handle, stdout, table*

**Input**

| surface (spelling) | cert |
|---|---|
| python (free) | ✓ |
| node (free) | ✓ |
| duckdb (free) | ✓ |
| rust (chained) | ✓ |
| cli (free) | ✓ |
| browser (free) | — |

**Output**

| surface (spelling) | handle | table | stdout |
|---|---|---|---|
| python (free) | ✓ |   |   |
| node (free) | ✓ |   |   |
| duckdb (free) |   | ✓ |   |
| rust (chained) | ✓ |   |   |
| cli (free) |   |   | ✓ |
| browser (free) |   |   |   |

_Findings:_
- ⚪ by-design · **browser** in.cert — the browser read handle is transient (one in-memory ParsedDataset per upload); there is no persisted-cert reuse flow to consume.

_Notes:_
- _python_: explicit opt-in — autodiscovery is deliberately refused so naming index= asserts the cert is for THIS file.
- _node_: explicit opt-in, mirrors Python.
- _duckdb_: implicit — a sibling <path>.idx is auto-consumed (the divergent FORM of the same cert-input capability: autodiscovery vs explicit).
- _rust_: Added 2026-08-05 (phase 4b). Spelled on VALIDATE rather than on read, because this facade's validate is a free function and has no `Document::validate()` for a cert attached at read time to reach. Same modality as python/node's `read(index=)` — cert as an input form — and the same refusal to auto-discover. NOTE the asymmetry this reveals in the siblings: a vouched cert with no world check lets the engine skip PARSING entirely, which python/node cannot reach because building a handle parses by definition.
- _cli_: --index (#393) consumes a fresh, same-engine, profile-covering .ags.idx to SKIP the rule engine (mirrors the library read(index=) short-circuit); a stale/foreign/insufficient cert is re-validated. Explicit opt-in like Python/Node, not autodiscovery like DuckDB.

### emit — Get spec-correct AGS4 back OUT of a read handle.

*Offered anywhere — in: handle · out: bytes, file, table, text*

**Input**

| surface (spelling) | handle |
|---|---|
| python (chained) | ✓ |
| node (chained) | ✓ |
| browser (chained) | ✓ |
| rust (chained) | ✓ |

**Output**

| surface (spelling) | file | bytes | text | table |
|---|---|---|---|---|
| python (chained) | ✓ | ✓ | ✓ |   |
| node (chained) | ✓ | ✓ | ✓ |   |
| browser (chained) |   | — | — | ✓ |
| rust (chained) | ✓ | ✓ | ✓ |   |

_Findings:_
- 🟠 P2 · **browser** out.bytes `wasm-read-emit` — the wasm read handle (ParsedDataset) exposes only group_codes/meta/arrow_ipc (typed table-out) — no .text/.bytes AGS4 re-emit, so there is no round-trip AGS4 out of a browser read (Python/Node Ags4File have .text/.bytes/.save). Compute the fix instead via the separate build/apply verbs.

_Notes:_
- _rust_: Partial. `Written` exposes bytes only. The earlier note here claimed a String door needed a lossy/strict decision first — that was wrong: the concern applies to READING arbitrary files, not to our own emitter's output, which is UTF-8 by construction. Python already treats it that way, with `Ags4File.text` primary and `.bytes` its UTF-8 encoding.

### to_excel — Convert AGS4 to an .xlsx workbook.

*Offered anywhere — in: bytes, file-like, path, text · out: bytes, file*

*Below the facade floor — the Rust crate does not yet offer in: bytes, path · out: bytes, file, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | path | text | bytes | file-like |
|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ |   | ✓ |   |
| browser (free) |   |   | ✓ |   |
| rust (absent) |   |   |   |   |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) |   | ✓ |
| rust (absent) |   |   |

_Notes:_
- _python_: to_excel(output=None) (#391) returns the .xlsx bytes in memory — the FS-free door the browser's ags4_to_xlsx already offered.
- _node_: #391 added bytes-in/bytes-out (omit xlsxPath → Buffer) + the Ags4File.toExcel() handle method — mirrors Python's to_excel.
- _rust_: Reversed 2026-08-04 (dec-facade-parity): TO ADD, behind an optional `excel` feature. The earlier DO-NOT-ADD rested on a Rust caller being able to `cargo add laterite-ags4-excel` directly — a door that was never open, since the crate is publish = false and has never been on crates.io. The dependency cost survives the reversal because an optional dep is not compiled, downloaded or locked by anyone who leaves the feature off, so the calamine + rust_xlsxwriter weight the crate map extracted stays off every consumer that does not ask for it.

### from_excel — Convert an AGS4-shaped .xlsx back to AGS4.

*Offered anywhere — in: bytes, path · out: bytes, file, handle*

*Below the facade floor — the Rust crate does not yet offer in: bytes, path · out: file, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | path | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) |   | ✓ |
| rust (absent) |   |   |

**Output**

| surface (spelling) | file | bytes | handle |
|---|---|---|---|
| python (free) | ✓ |   | ✓ |
| node (free) | ✓ | ✓ | — |
| browser (free) |   | ✓ |   |
| rust (absent) |   |   |   |

_Findings:_
- ⚪ by-design · **node** out.handle — Node fromExcel(bytes) returns the AGS4 Buffer, not an Ags4File; the handle is one read() away (read(fromExcel(bytes))) — the Node idiom, where read IS the handle constructor. Python returns the handle directly as a convenience.

_Notes:_
- _python_: from_excel(source) (#391) accepts raw .xlsx bytes — an uploaded workbook needn't hit disk first.
- _rust_: Reversed 2026-08-04 with to_excel (dec-facade-parity): TO ADD, behind the same optional `excel` feature — same false premise, same reasoning.

### transport-pack — zstd-only compress/decompress (pack/unpack).

*Offered anywhere — in: bytes, path · out: bytes, file*

**Input**

| surface (spelling) | path | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| rust (chained) | ✓ | ✓ |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| rust (chained) | ✓ | ✓ |

_Notes:_
- _node_: the *Bytes forms (#389) mirror laterite-py's pack_bytes/unpack_bytes — same shared-leaf envelope, so a Node-sealed blob interops with the file API and pyrage/the browser.
- _rust_: Added 2026-08-05 (phase 4a of dec-facade-parity) as `laterite::transport`, at the crate ROOT rather than under `ags4` — the envelope is zstd over arbitrary bytes and understands no format. Level and (for lock) work factor are builder knobs; unpack/unlock are plain functions because they have nothing to configure.

### transport-lock — zstd + age passphrase encrypt/decrypt (lock/unlock) — the motivating capability.

*Offered anywhere — in: bytes, path · out: bytes, file*

**Input**

| surface (spelling) | path | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) | — | ✓ |
| rust (chained) | ✓ | ✓ |
| cli (free) | ✓ | — |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) | — | ✓ |
| rust (chained) | ✓ | ✓ |
| cli (free) | ✓ |   |

_Findings:_
- ⚪ by-design · **browser** in.path — no filesystem in the browser — bytes-in/bytes-out is the only form. This surface (web/src/lib/transportClient.ts, JS zstd/age) is where the motivating bug lived; the wasm CRATE has zero transport code.
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat lock/unlock take an input path and write an output file (transport::lock(input, output)); there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs.

_Notes:_
- _python_: the *_bytes forms were added in 0.6.2-dev to close the exact path-only-vs-browser-bytes-only gap this whole audit is named after.
- _node_: the *Bytes forms (#389) close the remaining leg of the motivating gap — lockBytes never writes plaintext to disk; same shared-leaf envelope as the Python/browser forms.
- _rust_: Added 2026-08-05 (phase 4a). Same shared-leaf envelope as every other surface, so a Rust-sealed blob opens with `lat unlock`, Python, Node, the browser and stock age. The password-carrying builders redact in `Debug` — a derived one would put the passphrase in any log line that renders the builder.

### read_typed — Read AGS4 into the typed-graph object model.

*Offered anywhere — in: bytes, file-like, path, text · out: handle*

**Input**

| surface (spelling) | path | text | bytes | file-like |
|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |
| node (free) |   |   |   |   |
| rust (absent) |   |   |   |   |

**Output**

| surface (spelling) | handle |
|---|---|
| python (free) | ✓ |
| node (free) | — |
| rust (absent) |   |

_Findings:_
- 🟡 P3 · **node** out.handle `node-read-typed` — Node has the 174 typed-graph classes (for buildAgs4) but no read_typed to populate them FROM a file — the biggest port lift, a real typed-graph reader. Last in the backlog.

_Notes:_
- _rust_: Absent. Held for 0.2 by the published plan (dec-rust-api-crates-io): 0.1.x scope is read/validate/write and this is additive, so nothing has to move to admit it.

### read-output-view — Output-shaping of a read (xn numeric coercion).

*Offered anywhere — in: path · out: table*

**Input**

| surface (spelling) | path |
|---|---|
| python (free) | ✓ |
| rust (absent) |   |

**Output**

| surface (spelling) | table |
|---|---|
| python (free) | ≈ |
| rust (absent) |   |

_Findings:_
- 🟡 P3 · **python** out.table `xn-numeric-view` — read(xn=) casts XN columns to Float64 — a Python-only output-shaping view with no sibling and (unlike keys/backend) no knob-parity gate. An open port-or-document decision, so it is a P3 gap/divergence, not a settled by-design.

_Notes:_
- _rust_: Absent. python-only (a dataframe-backend view), so the floor is empty. A Rust caller works with the engine's own types.

## Legend

Grid: ✓ present · — absent · ≈ divergent (present but shape differs from siblings). Findings: 🔴 P1 · 🟠 P2 · 🟡 P3 · ⚪ by-design. Generated from [[crate-map|the crate map]]'s surfaces via `repo:tools/gen_modality.py`; the SSOT is `repo:modality.json`. Two standing gates: `repo:packages/laterite/tests/test_modality_parity.py` holds the SSOT against the live surfaces, and `gen_modality.py --check` holds this page against the SSOT.
