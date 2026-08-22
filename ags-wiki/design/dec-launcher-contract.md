---
type: decision
title: "The launcher contract: machine output is exact, human output is content-bound and layout-free"
status: accepted
tags: [design, decision]
decided: 2026-08-22
supersedes: []
from_gap: []
related: [design/_README, concepts/agent-first-cli-contract, concepts/surface-census, tools/laterite-ags4-xcheck, tools/laterite-cliutil, tools/laterite-cli]
sources: []
---

# The launcher contract: machine output is exact, human output is content-bound and layout-free

## Context

`lat` ships behind three launchers — the native Rust binary, `uvx --from laterite
lat`, and `npx laterite`. `surfaces/cli.md` opened with "It is the same tool
however you launch it", and the strong claim under it was scoped to `--json` /
`--ndjson`. Nothing said what the *human* output promised, so the sentence was
read as promising everything.

It does not deliver everything. The three launchers render human output three
ways: the binary and `uvx` draw comfy-table boxes (`uvx` by a deliberate hand-port
of the glyphs), and `npx` draws `padEnd` columns joined by `" | "`. On `rules` the
column order differs; on `validate` the shape differs entirely.

Which of those are defects depended on a promise nobody had written down.

**"Parity" could not carry the answer.** It already names six distinct things
here: rule-key presence against python-ags4 ([[parity-model]]), per-class
confidence ([[parity-confidence-model]]), the four surfaces' numbered-rule floor
([[laterite-ags4-compliance]]), the four surfaces' emitted values
([[laterite-ags4-xcheck]]), the three launchers' verb set
([[surface-census]]), and the Rust facade's API level ([[dec-facade-parity]]).
A seventh sense would have been unusable.

Note also that **surface** and **launcher** are different sets: four surfaces
(rust / python / node / wasm) and three launchers. Rendering is a launcher
property — wasm has no CLI — and [[surface-census]] is named for the first while
measuring the second.

## Options considered

1. **Scope the claim in prose only.** Say the byte-level promise covers the
   scriptable outputs, and qualify the cookbook CLI tabs.
2. **Bring `npx` onto the shared rendering.** Either take `comfy-table` +
   `indicatif` into the napi addon, or hand-port as `uvx` did.
3. **Split the promise by kind** — exact on machine output, content-bound and
   layout-free on human output.

Option 2 has no cheap form. `comfy-table` lives in [[laterite-cliutil]], which is
bin-side deliberately: the validator library's lean dep-graph (no
walkdir/rayon/ratatui) is a hard guarantee of the [[agent-first-cli-contract]].
Sharing the renderer would either break that guarantee or put the UI crates in
the addon, so option 2 reduces to a *third* hand-written box-drawing
implementation.

## Decision

The **launcher contract**, in three tiers:

| tier | promise | held by |
| --- | --- | --- |
| **Verbs and flags** | identical across the three launchers | [[surface-census]] |
| **Machine output** (`--json` / `--ndjson` / `--csv`) | **byte-exact** | one renderer in the engine, called by all three; [[laterite-ags4-xcheck]]'s CLI legs compare raw stdout |
| **Human output** | **content** is identical; **layout** is each launcher's own | a content gate across the three launchers, plus a byte gate on the binary ↔ `uvx` pair |

A reader must not learn *less* from one launcher than another. A reader may see
it drawn differently.

The binary ↔ `uvx` pair is additionally held **byte-identical on every verb's
human stdout**. That is not a contract term — it is a tripwire on a deliberate
implementation choice, and it is recorded as exceeding the contract so nobody
reads it as evidence the contract says more than it does.

## Why

Splitting by kind is the only division that matches what each output is *for*.
Machine output has a consumer that breaks on a byte; human output has a reader
who does not care whether the rule is drawn with `-` or `─`, and each language's
conventions and tooling differ enough that forcing one layout buys nothing and
costs a hand-port that decays.

Binding **content** is what keeps that from becoming a licence to drift. Layout
is free; facts are not. It also makes the promise checkable, which "we aim to
keep them close" was not.

Keeping the binary ↔ `uvx` byte gate despite layout being unpromised is
deliberate: the hand-port was expensive and rots silently. A tripwire on a choice
is not the same as a term of the contract, and the difference is written into the
gate.

## Consequences

**Committed to:**

- One engine renderer per machine output, called by every launcher — extending
  the treatment `read` and `validate` already have to `diff` and `merge`.
- A content gate over the three launchers' human output. It needs all three built,
  so it reports what it could not reach rather than skipping quietly.
- The binary ↔ `uvx` byte gate covering every verb, not only `read`.
- Human-output content divergences are defects. Three are known: `npx validate`
  states the dictionary edition and resolution where the other two do not (the
  binary gains it — knowing which dictionary judged the file is worth having);
  `npx diff` omits the header, the `groups added` / `groups removed` lines and
  the `total:` line, and lists only changed groups; `npx fix` prints its result
  to **stderr**, where the others use stdout — a straight departure from the
  [[agent-first-cli-contract]]'s "resolved-mode results to stdout".

**Ruled out:**

- A third box-drawing implementation in TypeScript.
- `comfy-table` / `indicatif` as dependencies of the napi addon.
- Reading the byte gate as a contract term.

**Left open:** whether a fixture carrying non-ASCII splits `diff --json` and
`merge` across launchers. `uvx` renders both through `json.dumps` without
`ensure_ascii=False`, which ASCII-escapes what `serde_json` and `JSON.stringify`
emit raw — the defect laterite-dev#545 fixed on `fix` alone. The xcheck CLI legs
compare raw stdout and would catch it, but every current fixture is pure ASCII,
so the gate is green over a case it cannot construct. The fixture comes first;
what it shows decides whether the shared renderer is a repair or hygiene.

## Related

[[agent-first-cli-contract]] · [[surface-census]] · [[laterite-ags4-xcheck]] ·
[[laterite-cliutil]] · [[dec-facade-parity]] · [[parity-model]] ·
repo:web/docs-site/docs/surfaces/cli.md ·
repo:rust-packages/laterite-node/ts/cli.ts ·
repo:rust-packages/laterite-cli/src/render.rs ·
repo:packages/laterite/python/laterite/_cli.py ·
repo:tools/xcheck/emit_cli.py
