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

Every laterite capability is one engine behind several doors. A 'modality' is the I/O *form* a capability is offered in — an input door (path / text / bytes / file-like / handle / stdin / cert) or an output door (file / bytes / text / handle / value / table / stdout). The behavioural-knob parity gates (test_free_chained_parity, test_cross_surface_parity) compare pairs that exist on BOTH sides and STRIP the modality-bearing params before comparing, so nothing there detects a capability offered in fewer forms on one surface — an *absence*, not a *drift*. This register is that missing axis: one cell per (capability, surface, spelling), each form tri-stated present|absent|divergent, each absence verdicted gap|by-design with a reason. It is the find-only deliverable AND the by-design allowlist the standing gate (test_modality_parity) checks reflected reality against. A second gate, gen_modality.py --check, holds the rendered page against this SSOT — the two guard different axes. A third, check_duckdb_manifest.py, covers the one surface reflection cannot reach: duckdb is a separate extension, so its cells are hand-authored, and that gate cross-checks the verbs they name against a pinned copy of the extension's own function manifest in both directions — a function renamed upstream leaves a cell naming nothing, one added upstream appears in no cell. The sibling baseline (which surface offers the richest form-set for a capability) is COMPUTED by the generator, never stored — a stored baseline is the multi-source-of-truth class #181 exists to kill.

This register is the **I/O-form** axis of cross-surface parity — does a capability exist in this SHAPE on this surface. [[surface-census]] is the **verb/table** axis of the same problem — does it exist AT ALL on this surface. Both share the reflect-don't-hand-list discipline (a form or verb list authored by hand is just a fourth thing to drift), and both exist because a *value*-comparison gate structurally cannot see an absence: feed identical input through every surface and diff the outputs, and a door that was never built produces no output to diff.

## Findings backlog (find-only — fixes are follow-ups)

- **🔴 P1** (0): —
- **🟠 P2** (4): read/cli (in.stdin); validate/cli (in.stdin); build/browser (in.text); emit/browser (out.bytes)
- **🟡 P3** (5): read/rust (in.file-like); validate/rust (in.file-like); transport-pack/browser (in.bytes); read_typed/node (out.handle); read-output-view/python (out.table)
- **⚪ by-design** (17): intentional absences, rationale in each cell below.

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
| duckdb (n/a) | — | — | — | — | — |

**Output**

| surface (spelling) | value | stdout |
|---|---|---|
| python (free) | ✓ |   |
| node (free) | ✓ |   |
| rust (chained) | ✓ |   |
| cli (free) |   | ✓ |
| browser (free) | ✓ |   |
| duckdb (n/a) | — | — |

_Findings:_
- ⚪ by-design · **node** in.file-like — same as read — no universal Node file-like; pass bytes.
- 🟡 P3 · **rust** in.file-like — Above the facade floor — node does not offer it either — so the floor does not owe it. Recorded anyway because node's by-design reason ('no universal Node file-like') cannot be borrowed here: Rust has one, `impl std::io::Read`. Cheap to add; the floor is a minimum, not a cap on what gets built.
- 🟠 P2 · **cli** in.stdin `cli-stdin` — no '-'/stdin door; a piped file must be spooled to disk first.
- ⚪ by-design · **browser** in.path — no filesystem in the browser.

_Notes:_
- _python_: validate exposes only the positional source sniff + text= keyword — no explicit path=/data= keyword doors like read/fix. All input FORMS are still reachable via the sniff (_resolve_source accepts path/bytes/file-like), so this is a keyword-ergonomics inconsistency, NOT a lost modality — recorded here, deliberately not a gap.
- _rust_: Partial. `text` is the same trivial door as read's.
- _duckdb_: **by-design.** A read-only reader. `reference/duckdb-functions.md` states it: there is no `validate_ags` in SQL. Removed deliberately in laterite-dev#446. <!-- retired: validate_ags -->

### fix — Mechanically repair AGS4.

*Offered anywhere — in: bytes, file-like, path, text · out: bytes, file, stdout, value*

**Input**

| surface (spelling) | path | text | bytes | file-like |
|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ | ✓ | ✓ |   |
| browser (free) |   |   | ✓ |   |
| rust (chained) | ✓ | ✓ | ✓ |   |
| cli (free) | ✓ |   | — |   |
| duckdb (n/a) | — | — | — | — |

**Output**

| surface (spelling) | file | bytes | value | stdout |
|---|---|---|---|---|
| python (free) | ✓ |   | ✓ |   |
| node (free) | ✓ |   | ✓ |   |
| browser (free) |   | ✓ | ✓ |   |
| rust (chained) | ✓ |   | ✓ |   |
| cli (free) | ✓ |   | ✓ | ✓ |
| duckdb (n/a) | — | — | — |   |

_Findings:_
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat fix takes an input path and writes an output file; there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs (see transport-lock/cli, the same posture).

_Notes:_
- _node_: #394 added inPlace/out write-back (the out.file form) + only/exclude rule selection — the latter shrank the test_cross_surface_parity _MATRIX allowlist to empty. Rule labels are typed as FixableRule, drift-gated to Python/the engine (test_typed_choices).
- _browser_: the browser deliberately SPLITS fix into compute_fixes (returns the Fix[] proposal for the UI to preview — a value form) and apply_fixes (returns the repaired bytes). The library surfaces one-shot fix() and offer no dry-run Fix[] preview form; whether to add one is P3 verb-decomposition (fix-dry-run-split), tracked in the backlog, not a browser defect.
- _rust_: Added 2026-08-05 (phase 4c of dec-facade-parity). One builder over the three source doors, ending at `Fixed` — the value form. The file form is `to_path`, and in-place is the source path named as the destination rather than a separate flag: there is nothing a flag would express that the path does not. Rule selection (`only`/`exclude`) speaks the same short labels as python and node, read from the engine via `fixable_rules()` so a new fix cannot leave the list behind.
- _cli_: writes a sibling by default; `--in-place` overwrites the source and `--fix-out <path>` names the destination, so the destructive form is opt-in rather than the default. `--json` puts the machine-readable report of what was repaired on stdout.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.

### build — Construct valid AGS4 from caller-supplied data (build_ags4).

*Offered anywhere — in: bytes, handle, text, value · out: bytes, file, value*

*Below the facade floor — the Rust crate does not yet offer out: file, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | text | bytes | handle | value |
|---|---|---|---|---|
| python (free) |   |   | ✓ | ✓ |
| node (free) |   |   | ✓ | ✓ |
| browser (free) | ≈ | ✓ |   |   |
| rust (chained) |   |   | ✓ | ✓ |
| duckdb (n/a) | — | — | — | — |
| cli (absent) |   |   |   |   |

**Output**

| surface (spelling) | file | bytes | value |
|---|---|---|---|
| python (free) | ✓ |   | ✓ |
| node (free) | ✓ |   | ✓ |
| browser (free) |   | ✓ |   |
| rust (chained) |   |   | ✓ |
| duckdb (n/a) |   | — | — |
| cli (absent) |   |   |   |

_Findings:_
- 🟠 P2 · **browser** in.text `wasm-build-text-outlier` — build_ags4 takes a JSON-TEXT groups payload — the lone text-in build door across all surfaces (Python/Node take a typed-graph root or (code, frame) rows). Bytes-in already exists as build_ags4_ipc, so the reconciliation is the JSON-text outlier, NOT adding a bytes door.

_Notes:_
- _python_: The `file` output form is the #855 to-disk rider (`build_ags4(out=)` / `buildAgs4({ out })`): the judged document is staged to a temp file beside the destination and moved into place only after the verdict allows, and the result (`BuildSaved`) carries the path and the verdict, deliberately no bytes. The browser cell has no counterpart for the same reason fix's doesn't — no filesystem; a modality fact, not a gap.
- _node_: The `file` output form is the #855 to-disk rider (`build_ags4(out=)` / `buildAgs4({ out })`): the judged document is staged to a temp file beside the destination and moved into place only after the verdict allows, and the result (`BuildSaved`) carries the path and the verdict, deliberately no bytes. The browser cell has no counterpart for the same reason fix's doesn't — no filesystem; a modality fact, not a gap.
- _rust_: Added 2026-08-05 (phase 4c). The value door takes `GroupData` rows of a first-party `Cell` enum, NOT the engine's `serde_json::Value` — the facade's no-third-party-type rule is load-bearing here, and the enum is what preserves the typed formatting python and node get from an Arrow frame (a number goes through its heading's declared TYPE, a string is written verbatim). The handle door is `build_document`, which reuses the same emit pipeline as `write` through one shared call rather than a second copy of it.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.
- _cli_: **by-design.** "Construct from caller-supplied data" has no shell shape. The surface promise is the reason: the CLI is a file tool — path in, file out, no in-memory objects, no caller-supplied data structures.

### build-unchecked — Construct AGS4 from caller-supplied data with NO validity verdict — build's assembly minus the judge (#858).

*Offered anywhere — in: handle, text, value · out: bytes, file*

*Below the facade floor — the Rust crate does not yet offer in: handle, value · out: bytes, file, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | text | handle | value |
|---|---|---|---|
| python (free) |   | ✓ | ✓ |
| node (free) |   | ✓ | ✓ |
| browser (free) | ✓ |   |   |
| rust (absent) |   |   |   |
| duckdb (n/a) |   |   |   |
| cli (absent) |   |   |   |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) |   | ✓ |
| rust (absent) |   |   |
| duckdb (n/a) |   |   |
| cli (absent) |   |   |

_Notes:_
- _python_: Byte-identical to build_ags4(mode="report") — pinned by test — with the verdict skipped; the docstring is the consent form ("you are choosing to ship unchecked bytes"). The judge-coupled knobs are gone, not defaulted: no mode, no synthesise_metadata/tran; edition/units/types stay. Returns plain bytes, deliberately NOT a BuildResult — an empty findings list would read as "judged clean". The file form is build's staged write minus the verdict gate in front of it.
- _node_: Landed via #881, one release behind Python's door. Byte-identical to buildAgs4({ mode: "report" }) — pinned by test — returning a plain Buffer (deliberately not a BuildResult), or the path with out= via the same staged rename minus the verdict gate. The judge-coupled knobs are refused at runtime by name, never silently ignored — absence from the TS type alone would let a JS caller's mode: "strict" be dropped on the floor.
- _browser_: Landed via #881. Takes the judged door's own groups_json (the build capability's recorded JSON-text outlier rides along unchanged — reconciling it is that cell's gap, not this one's) and returns a Uint8Array, byte-identical to the judged report build's text. dictVersion is the only option; the decode_opts KEYS guard refuses mode/synthesiseMetadata/tran by name. No filesystem, so no file rider — bytes being the universal output form is what lets this door exist in a browser at all.
- _rust_: Absent from the FACADE. The engine crate the facade wraps ships both entries (laterite-ags4-emit::emit_ags4_unchecked / emit_ags4_from_arrow_unchecked — they are what every surface binds), so a facade spelling is cheap, but the floor (python n node) does not owe it until node's #881 half lands; adopt deliberately then, not by reflex now.
- _duckdb_: **by-design.** Same as build: the extension is a read-only reader (its canonical manifest declares read_only: true), and this capability writes.
- _cli_: **by-design.** Same as build: "construct from caller-supplied data" has no shell shape — and a shell caller who wants a no-verdict write has nothing to feed it anyway; the CLI's doors start from files.

### diff — Compare two AGS4 revisions.

*Offered anywhere — in: bytes, file-like, handle, path, text · out: stdout, value*

**Input**

| surface (spelling) | path | text | bytes | file-like | handle |
|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ |   | ✓ | — | ✓ |
| browser (free) |   |   | ✓ |   |   |
| rust (chained) | ✓ |   | ✓ |   | ✓ |
| cli (free) | ✓ |   | — |   |   |
| duckdb (absent) |   |   |   |   |   |

**Output**

| surface (spelling) | value | stdout |
|---|---|---|
| python (free) | ✓ |   |
| node (free) | ✓ |   |
| browser (free) | ✓ |   |
| rust (chained) | ✓ |   |
| cli (free) | ✓ | ✓ |
| duckdb (absent) |   |   |

_Findings:_
- ⚪ by-design · **node** in.file-like — no universal Node file-like; DiffSource is string|Uint8Array|Ags4File.
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat diff takes an input path and writes its delta to stdout; there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs (see transport-lock/cli, the same posture).

_Notes:_
- _rust_: Added 2026-08-06 (phase 4d of dec-facade-parity). The value form is a typed `Delta` of first-party handles, not the engine's `Serialize` structs — the same rule that made `Cell` its own enum. NOTE the handle door compares each document AS IT STANDS, edits included, by re-emitting it: python's handle form resolves to `Ags4File.bytes` (the re-emit) for the same reason, and diffing the file on disk instead would silently ignore an edit. One asymmetry worth knowing about the shared engine: a group present on only one side is reported whole via `groups_added`/`groups_removed` and its rows do NOT reach the totals, so summing them is the wrong way to ask whether anything changed.
- _cli_: both sides are paths and the delta goes to stdout — there is no file output form here, unlike the other four CLI verbs in this table.
- _duckdb_: **by-design — output shape, not `read_only`.** `RevisionDelta` is three levels deep (groups → rows → cells) with group-level totals and heading changes, while every existing extension function returns a flat table of one subject; a SQL `ags_diff()` would be a new relational projection, not a port. The read-only manifest is NOT the reason — `ags_rules`, `ags_dictionary` and `ags_relationships` already read no user file.

### merge — Reconcile N AGS4 deliveries of one project into one file.

*Offered anywhere — in: bytes, file-like, handle, path, text · out: file, stdout, value*

**Input**

| surface (spelling) | path | text | bytes | file-like | handle |
|---|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ | ✓ |
| node (free) | ✓ |   | ✓ | — | ✓ |
| rust (chained) | ✓ |   | ✓ |   | ✓ |
| cli (free) | ✓ |   |   |   |   |
| browser (free) |   |   | ✓ |   |   |
| duckdb (n/a) | — | — | — | — | — |

**Output**

| surface (spelling) | file | value | stdout |
|---|---|---|---|
| python (free) | ✓ | ✓ |   |
| node (free) | — | ✓ |   |
| rust (chained) |   | ✓ |   |
| cli (free) | ✓ |   | ✓ |
| browser (free) | ✓ | ✓ |   |
| duckdb (n/a) | — | — | — |

_Findings:_
- ⚪ by-design · **node** in.file-like — no universal Node file-like; MergeSource is string|Uint8Array|Ags4File.
- ⚪ by-design · **node** out.file — Node merge returns a MergeResult carrying the merged bytes; the caller writes with fs (return-only, matching diff).

_Notes:_
- _rust_: Added 2026-08-06 (phase 4d). Wraps `merge_parsed`'s `&[ParsedFile]` door — the shape dec-ags4-merge-semantics and laterite#162 endorse — so no provenance typestate reaches the facade. `Merged::save` writes, but the door's own out-form is the value: node offers no file form, so the floor does not ask for one. Two new `ErrorKind` variants (`TypeConflict`, `UnitConflict`) carry the siblings' exact wire tokens rather than collapsing onto `Other`, because the engine draws that distinction deliberately — widen/promote settles the first and nothing settles the second.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.

### censor — Anonymise an AGS4 file — scrub the classified sensitive cells (pseudonymise IDs, hash PROJ_ID, blank coordinates, tokenise names, strip free-text [units]) before sharing. Browser-only among shipped surfaces: the SAME laterite-ags4-censor engine backs the private laterite-ags4-corpus-qa `censor` dev tool, which is not a shipped py/node/cli API, so its cross-surface EXISTENCE is a surface-census matter, not a modality one.

*Offered anywhere — in: bytes · out: file, value*

**Input**

| surface (spelling) | bytes |
|---|---|
| browser (free) | ✓ |
| rust (absent) |   |
| duckdb (n/a) | — |
| cli (absent) |   |
| python (absent) |   |
| node (absent) |   |

**Output**

| surface (spelling) | file | value |
|---|---|---|
| browser (free) | ✓ | ✓ |
| rust (absent) |   |   |
| duckdb (n/a) | — | — |
| cli (absent) |   |   |
| python (absent) |   |   |
| node (absent) |   |   |

_Notes:_
- _rust_: Absent. Browser-only among shipped surfaces, so the floor is empty here — there is no python/node pair to be the least of.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.
- _cli_: **by-design.** There is no `lat censor` — censor is browser-only among shipped surfaces, so there is no CLI spelling for this capability to be absent from. Matches the `rust` and `duckdb` cells on this row.
- _python_: **by-design.** Censor is browser-only among shipped surfaces, so there is no Python spelling for this capability to be absent from. Matches the `rust`, `duckdb` and `cli` cells on this row.
- _node_: **by-design.** Censor is browser-only among shipped surfaces, so there is no Node spelling for this capability to be absent from. Matches the `rust`, `duckdb` and `cli` cells on this row.

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
| duckdb (n/a) | — | — | — |

**Output**

| surface (spelling) | file | bytes | text |
|---|---|---|---|
| python (chained) | ✓ | ✓ |   |
| node (chained) | ✓ | ✓ |   |
| rust (chained) | ✓ | ✓ |   |
| cli (free) | ✓ |   |   |
| browser (free) | — |   | ✓ |
| duckdb (n/a) | — | — | — |

_Findings:_
- ⚪ by-design · **browser** out.file `certify-bytes-output` — certify returns the cert as a String because the browser has no filesystem — a PRESENT capability, not a shape gap. It proved the in-memory shape that Python/Node now match via certify_bytes/certifyBytes (#390).

_Notes:_
- _python_: certify_bytes() (#390) returns the .ags.idx bytes in memory — the certify analog of transport.lock_bytes; same cert as certify() (bar the mint timestamp), so it interops with read(index=)/--index/the browser.
- _node_: certifyBytes() (#390) mirrors laterite-py's certify_bytes — same in-memory cert form.
- _rust_: Added 2026-08-05 (phase 4b). Mints over the ORIGINAL source bytes, before any transcode, with the encoding recorded alongside — the same rule laterite-py follows. Note the engine limit this exposes: bytes that are not UTF-8 validate but cannot be certified at all, because a certificate carries byte offsets the engine will not record into bytes it cannot address. That holds on every surface.
- _duckdb_: **by-design.** A read-only reader. `reference/duckdb-functions.md` states it: there is no `certify_ags` in SQL. Removed deliberately in laterite-dev#446. <!-- retired: certify_ags -->

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
- _python_: explicit opt-in — autodiscovery is deliberately refused so naming index= asserts the cert is for THIS file. Spelled on BOTH doors since #271, and they are not two spellings of one thing: `validate(index=)` answers the verdict without PARSING, while `read(index=)` must parse to build the handle and so skips only the rules. A stale cert raises StaleCertError from either door — naming one is an assertion — but a cert merely INHERITED by `Ags4File.validate()` asserts nothing there and falls back with `revalidate_reason`.
- _node_: explicit opt-in, mirrors Python — including #271's second door: `validate(file, {index})` skips the parse, `read(file, {index}).validate()` skips only the rules, and a stale cert throws StaleCertError from either while an inherited one falls back with `revalidateReason`.
- _duckdb_: implicit — a sibling <path>.idx is auto-consumed (the divergent FORM of the same cert-input capability: autodiscovery vs explicit).
- _rust_: Added 2026-08-05 (phase 4b). Spelled on VALIDATE rather than on read, because this facade's validate is a free function and has no `Document::validate()` for a cert attached at read time to reach. Same modality as python/node's `read(index=)` — cert as an input form — and the same refusal to auto-discover. The asymmetry this revealed in the siblings — a vouched cert with no world check lets the engine skip PARSING entirely, which python/node could not reach because building a handle parses by definition — was closed by #271, which gave both of them the same validate-side door. This facade still has only the one, because it still has no `Document::validate()`.
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
| duckdb (n/a) | — |
| cli (absent) |   |

**Output**

| surface (spelling) | file | bytes | text | table |
|---|---|---|---|---|
| python (chained) | ✓ | ✓ | ✓ |   |
| node (chained) | ✓ | ✓ | ✓ |   |
| browser (chained) |   | — | — | ✓ |
| rust (chained) | ✓ | ✓ | ✓ |   |
| duckdb (n/a) | — | — | — | — |
| cli (absent) |   |   |   |   |

_Findings:_
- 🟠 P2 · **browser** out.bytes `wasm-read-emit` — the wasm read handle (ParsedDataset) exposes only group_codes/meta/rows_json — and arrow_ipc where the `arrow` feature is built (#330; the published package is the slim build, which has rows_json only). All are typed table-out: no .text/.bytes AGS4 re-emit on the handle itself, so there is no round-trip AGS4 out of a browser read the way Python/Node Ags4File have .text/.bytes/.save. Assemblable rather than absent since #330 — meta() + rows_json() together ARE build_ags4's input shape, so read -> edit -> write composes through the separate build verb (held by `rows_json_feeds_build_ags4_straight_back` in the crate's tests); the gap is that the handle does not do it for you.

_Notes:_
- _rust_: Partial. `Written` exposes bytes only. The earlier note here claimed a String door needed a lossy/strict decision first — that was wrong: the concern applies to READING arbitrary files, not to our own emitter's output, which is UTF-8 by construction. Python already treats it that way, with `Ags4File.text` primary and `.bytes` its UTF-8 encoding.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.
- _cli_: **by-design — but NOT because of the file-tool promise, which does not exclude it**: a normalising re-emit reads a path and writes a file. The reason is that emit is a *component* on this surface rather than a user-facing verb — there is no handle to emit from, and `lat fix --fix-out`, `lat merge --out` and `lat excel` already do the emitting. A reader who sees the promise cited elsewhere on this row will correctly notice it does not fit here.

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
| cli (free) | ✓ |   | — |   |
| duckdb (n/a) | — | — | — | — |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) |   | ✓ |
| rust (absent) |   |   |
| cli (free) | ✓ |   |
| duckdb (n/a) | — | — |

_Findings:_
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat excel takes an input path and writes an output file; there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs (see transport-lock/cli, the same posture).

_Notes:_
- _python_: to_excel(output=None) (#391) returns the .xlsx bytes in memory — the FS-free door the browser's ags4_to_xlsx already offered.
- _node_: #391 added bytes-in/bytes-out (omit xlsxPath → Buffer) + the Ags4File.toExcel() handle method — mirrors Python's to_excel.
- _rust_: Reversed 2026-08-04 (dec-facade-parity): TO ADD, behind an optional `excel` feature. The earlier DO-NOT-ADD rested on a Rust caller being able to `cargo add laterite-ags4-excel` directly — a door that was never open, since the crate is publish = false and has never been on crates.io. The dependency cost survives the reversal because an optional dep is not compiled, downloaded or locked by anyone who leaves the feature off, so the calamine + rust_xlsxwriter weight the crate map extracted stays off every consumer that does not ask for it.
- _cli_: one subcommand carries both directions: the direction is inferred from the OUTPUT extension (`.xlsx` ⇒ export), overridable with `--export`. So this cell and from_excel/cli describe the same `lat excel` verb read two ways, not two subcommands.
- _duckdb_: **by-design.** The Excel crate is deliberately extracted so its `calamine`/`rust_xlsxwriter` deps do not ride into every consumer. A SQL reader of AGS4 is not the place to reverse that.

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
| cli (free) | ✓ | — |
| duckdb (n/a) | — | — |

**Output**

| surface (spelling) | file | bytes | handle |
|---|---|---|---|
| python (free) | ✓ |   | ✓ |
| node (free) | ✓ | ✓ | — |
| browser (free) |   | ✓ |   |
| rust (absent) |   |   |   |
| cli (free) | ✓ |   |   |
| duckdb (n/a) | — | — | — |

_Findings:_
- ⚪ by-design · **node** out.handle — Node fromExcel(bytes) returns the AGS4 Buffer, not an Ags4File; the handle is one read() away (read(fromExcel(bytes))) — the Node idiom, where read IS the handle constructor. Python returns the handle directly as a convenience.
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat excel takes an input path and writes an output file; there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs (see transport-lock/cli, the same posture).

_Notes:_
- _python_: from_excel(source) (#391) accepts raw .xlsx bytes — an uploaded workbook needn't hit disk first.
- _rust_: Reversed 2026-08-04 with to_excel (dec-facade-parity): TO ADD, behind the same optional `excel` feature — same false premise, same reasoning.
- _cli_: the import half of the same `lat excel` verb — `.ags` output ⇒ import, overridable with `--import`. `--no-format-numeric` leaves numeric-looking columns as text.
- _duckdb_: **by-design.** The Excel crate is deliberately extracted so its `calamine`/`rust_xlsxwriter` deps do not ride into every consumer. A SQL reader of AGS4 is not the place to reverse that.

### transport-pack — zstd-only compress/decompress (pack/unpack).

*Offered anywhere — in: bytes, path · out: bytes, file*

**Input**

| surface (spelling) | path | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| rust (chained) | ✓ | ✓ |
| cli (free) | ✓ | — |
| duckdb (n/a) | — | — |
| browser (absent) |   | — |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| rust (chained) | ✓ | ✓ |
| cli (free) | ✓ |   |
| duckdb (n/a) | — | — |
| browser (absent) |   | — |

_Findings:_
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat pack/unpack takes an input path and writes an output file; there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs (see transport-lock/cli, the same posture).
- 🟡 P3 · **browser** in.bytes — The zstd implementation is already loaded on this surface (`@bokuweb/zstd-wasm`, in the transport worker) and `lock` already runs it at level 9, so exposing the zstd-only `pack`/`unpack` door is wiring rather than new capability — the surface is incomplete here, not deliberately narrow.

_Notes:_
- _node_: the *Bytes forms (#389) mirror laterite-py's pack_bytes/unpack_bytes — same shared-leaf envelope, so a Node-sealed blob interops with the file API and pyrage/the browser.
- _rust_: Added 2026-08-05 (phase 4a of dec-facade-parity) as `laterite::transport`, at the crate ROOT rather than under `ags4` — the envelope is zstd over arbitrary bytes and understands no format. Level and (for lock) work factor are builder knobs; unpack/unlock are plain functions because they have nothing to configure.
- _cli_: declared four lines from `lock`/`unlock` in the same `#[cfg(feature = "transport")]` block and identical in posture — which is why this cell being absent while transport-lock's was present was an omission rather than a judgement (#771).
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes.
- _browser_: The crate's wasm-hostile `age`/`zstd` deps are NOT the reason this is empty: this surface reimplements transport in JS (`web/src/lib/transportClient.ts`) and does not use `laterite-transport` at all — the wasm CRATE has zero transport code. A reader who takes the absence as "zstd does not work in the browser" has it backwards. Distinct from the `gzip not zstd` choice on the report download (`ags-wiki/design/validator-site.md`), which is about a throwaway artifact and no native browser zstd ENCODER, not about this envelope.

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
| duckdb (n/a) | — | — |

**Output**

| surface (spelling) | file | bytes |
|---|---|---|
| python (free) | ✓ | ✓ |
| node (free) | ✓ | ✓ |
| browser (free) | — | ✓ |
| rust (chained) | ✓ | ✓ |
| cli (free) | ✓ |   |
| duckdb (n/a) | — | — |

_Findings:_
- ⚪ by-design · **browser** in.path — no filesystem in the browser — bytes-in/bytes-out is the only form. This surface (web/src/lib/transportClient.ts, JS zstd/age) is where the motivating bug lived; the wasm CRATE has zero transport code.
- ⚪ by-design · **cli** in.bytes — the CLI is a file tool: lat lock/unlock take an input path and write an output file (transport::lock(input, output)); there is no in-memory bytes door on the CLI, matching lat's other file-in/file-out verbs.

_Notes:_
- _python_: the *_bytes forms were added in 0.6.2-dev to close the exact path-only-vs-browser-bytes-only gap this whole audit is named after.
- _node_: the *Bytes forms (#389) close the remaining leg of the motivating gap — lockBytes never writes plaintext to disk; same shared-leaf envelope as the Python/browser forms.
- _rust_: Added 2026-08-05 (phase 4a). Same shared-leaf envelope as every other surface, so a Rust-sealed blob opens with `lat unlock`, Python, Node, the browser and stock age. The password-carrying builders redact in `Debug` — a derived one would put the passphrase in any log line that renders the builder.
- _duckdb_: **by-design.** The extension is a read-only reader: its canonical manifest (`../laterite-duckdb/functions.json`, gated against the `register_table()` calls by that repo's `tests/functions_manifest.rs`) declares `read_only: true`, and this capability writes. `lock` is additionally passphrase crypto, which a SQL function signature is a poor place to take.

### read_typed — Read AGS4 into the typed-graph object model.

*Offered anywhere — in: bytes, file-like, path, text · out: handle*

**Input**

| surface (spelling) | path | text | bytes | file-like |
|---|---|---|---|---|
| python (free) | ✓ | ✓ | ✓ | ✓ |
| node (free) |   |   |   |   |
| rust (absent) |   |   |   |   |
| duckdb (n/a) | — | — | — | — |
| cli (absent) |   |   |   |   |
| browser (absent) |   |   |   |   |

**Output**

| surface (spelling) | handle |
|---|---|
| python (free) | ✓ |
| node (free) | — |
| rust (absent) |   |
| duckdb (n/a) | — |
| cli (absent) |   |
| browser (absent) |   |

_Findings:_
- 🟡 P3 · **node** out.handle `node-read-typed` — Node has the 174 typed-graph classes (for buildAgs4) but no read_typed to populate them FROM a file — the biggest port lift, a real typed-graph reader. Last in the backlog.

_Notes:_
- _rust_: Absent. The 0.2 milestone this cell used to cite was RETIRED by dec-facade-parity on 2026-08-04 — the facade goes to parity once and then joins the product line, so there is no waypoint release left to hold this for. It is also outside the floor: `read_typed` has no Node sibling to take the intersection with, so parity never owes it. A Rust-native typed-graph reader stays an open question rather than a scheduled one.
- _duckdb_: **by-design.** This capability is defined as reading into the typed-graph object model — the generated group classes. `read_ags` returns a typed RELATION, which is a different thing and is already recorded under `read`. A SQL surface has no object model to return.
- _cli_: **by-design.** `read_typed` hands back an in-process object model, and a CLI has no process to hold the graph in. The surface promise is the reason: the CLI is a file tool — path in, file out, no in-memory objects, no caller-supplied data structures.
- _browser_: **by-design.** The typed-graph object model is a Python-only surface; every other cell on this row is absent for the same reason. The browser reads to a handle, not to a graph.

### read-output-view — Output-shaping of a read (xn numeric coercion).

*Offered anywhere — in: path · out: table*

**Input**

| surface (spelling) | path |
|---|---|
| python (free) | ✓ |
| rust (absent) |   |
| duckdb (n/a) | — |
| cli (absent) |   |
| node (absent) |   |
| browser (absent) |   |

**Output**

| surface (spelling) | table |
|---|---|
| python (free) | ≈ |
| rust (absent) |   |
| duckdb (n/a) | — |
| cli (absent) |   |
| node (absent) |   |
| browser (absent) |   |

_Findings:_
- 🟡 P3 · **python** out.table `xn-numeric-view` — read(xn=) casts XN columns to Float64 — a Python-only output-shaping view with no sibling and (unlike keys/backend) no knob-parity gate. An open port-or-document decision, so it is a P3 gap/divergence, not a settled by-design.

_Notes:_
- _rust_: Absent. python-only (a dataframe-backend view), so the floor is empty. A Rust caller works with the engine's own types.
- _duckdb_: **gap.** Defined as output-shaping of a read (`xn` numeric coercion). `XN` canonicalises to `string`, so the extension emits it VARCHAR with no opt-in to coerce, where Python offers `read(xn='numeric')`. Nothing prevents an `xn=` named parameter on `read_ags` — it already takes `encoding` — so this reads as not-yet rather than declined. The weakest verdict on this surface, and the only one here that is not `by-design`.
- _cli_: **by-design.** `lat read` emits strings; there is no numeric coercion to offer a view over. The surface promise is the reason: the CLI is a file tool — path in, file out, no in-memory objects, no caller-supplied data structures.
- _node_: **by-design.** `xn` numeric coercion is a python-only dataframe-backend view, as the `rust` cell already records — there is no Node sibling for it to be a view over.
- _browser_: **by-design.** `xn` numeric coercion is a python-only dataframe-backend view, as the `rust` cell already records. The browser's read returns Arrow, whose typing is settled at the read, not shaped after it.

### read-output-arrow — A group's born-typed raw Arrow output from the read handle.

*Offered anywhere — in: handle · out: bytes, table*

*Below the facade floor — the Rust crate does not yet offer in: handle · out: table, which python and node both do. A minimum to clear, not a gate*

**Input**

| surface (spelling) | handle |
|---|---|
| python (free) | ✓ |
| node (free) | ✓ |
| rust (absent) |   |
| duckdb (n/a) |   |
| cli (absent) |   |
| browser (free) | ✓ |

**Output**

| surface (spelling) | bytes | table |
|---|---|---|
| python (free) |   | ✓ |
| node (free) |   | ✓ |
| rust (absent) |   |   |
| duckdb (n/a) |   |   |
| cli (absent) |   |   |
| browser (free) | ✓ |   |

_Notes:_
- _python_: The capsule-bearing pyo3-arrow table, born-typed from the file's TYPE row, zero-copy over the Arrow PyCapsule interface — the exact shape build_ags4 consumes, so the read half of the zero-copy round trip #852 opened the write half of (#860). keys= is the same tri-state as .table()'s; no frame is materialised and the SQL engine is never touched.
- _node_: Arrow by construction — table() returns an arrow-js Table — but a DECODE of the boundary's IPC bytes, not a capsule hand-over: napi has no capsule analog (the Buffers ARE the boundary; the perf ledger's node lane records the cost). **Settled by #871 (2026-09-02): by-design — each boundary's Arrow shape is its own answer, and node's is the decoded arrow-js Table.** A public un-decoded-IPC door (`arrowIpc`) was spiked and REJECTED: arrow-js decode/re-encode are zero-copy and cost loose milliseconds per whole-file sweep, while the door would hand out raw MUTABLE Buffers beside a zero-copy decoder (aliasing and postMessage-detach hazards against every decoded view), be uncached by contract (silent keychain rebuilds on repeat keyed calls — caching instead would alias mutable memory across callers, worse), and make the IPC framing itself public contract. The spike — accessor, time bench, peak-RSS/retention probe — is preserved on `prototype/node-arrow-ipc`; the measured cells are on #871.
- _rust_: Absent. The facade reads to string-row AgsGroups; a typed-Arrow group accessor rides the same open question as its read_typed cell — no scheduled waypoint holds it.
- _duckdb_: **by-design.** read_ags already returns a typed RELATION — the engine's own Arrow-speaking object, recorded under read; a separate raw-table door would duplicate it.
- _cli_: **by-design.** The CLI hands back files and text, not in-process objects; lat read --csv/--json is its data-out door.
- _browser_: Arrow IPC bytes — the browser boundary's Arrow shape (a capsule cannot cross wasm). **Settled by #871 (2026-09-02): by-design** — each boundary's Arrow shape is its own answer: Python the capsule, node the decoded arrow-js Table, the browser these IPC bytes. No surface owes another surface's shape; the node cell's note records the spiked-and-rejected pass-through.

## Legend

Grid: ✓ present · — absent · ≈ divergent (present but shape differs from siblings). Findings: 🔴 P1 · 🟠 P2 · 🟡 P3 · ⚪ by-design. Generated from [[crate-map|the crate map]]'s surfaces via `repo:tools/gen_modality.py`; the SSOT is `repo:modality.json`. Two standing gates: `repo:packages/laterite/tests/test_modality_parity.py` holds the SSOT against the live surfaces, and `gen_modality.py --check` holds this page against the SSOT.
