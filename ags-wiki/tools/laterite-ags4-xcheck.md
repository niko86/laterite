---
type: tool
title: laterite-ags4-xcheck
status: drafted
tags: [tool]
tool_kind: crate
language: rust
artifact: laterite-ags4-xcheck
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite-ags4-xcheck"
related: [laterite-ags4-compliance, laterite-ags4-emit, crate-map]
sources: []
---
# laterite-ags4-xcheck

## What it is
> [!quote] **Implemented** (`repo:rust-packages/laterite-ags4-xcheck`). The
> cross-surface **output-value** gate: it holds every surface's *emitted
> values* (not just its findings) to a single authority, so a surface that
> parses/emits a cell differently is caught — the layer below findings
> agreement.

It's a deliberately **lean** crate — deps only `validator + emit + parse` — so
the gate builds fast and carries none of the findings-agreement harness's deps.
It was extracted out of the (larger) findings harness for exactly that reason;
see [[laterite-ags4-compliance]] for the split.

## Inputs / outputs
> [!quote] In: the shared **case manifest** (`cases/*.json` + `cases/inputs/`)
> — the single case-list SSOT this crate owns. Out: a pass/fail value-agreement
> verdict; non-zero exit on any unexplained split. **Zero normalisation** —
> JSON equality.

Two bins share one `#[path]`-included module (`xcheck_shared.rs`):
- `emit-cases` — the **authority** leg: drives the shared emit/format leaves
  directly (not through a binding), so its column is the reference every
  surface leg is held to.
- `xcheck` — the **comparator**: loads the case manifest + every leg's
  observation file, runs N-way equality + leaf authority + the spec invariants
  (`emit_reparses`), and exits non-zero on any unexplained split.

### The two anti-rot switches

Both are **CI-only**, and both exist because the same silence means different
things locally and on a runner:

| switch | locally | in CI |
|---|---|---|
| `--require-legs all` | a leg with no built artefact self-skips | a skipped leg is a **FAILURE** |
| `--require-engines all` | a leg that cannot name its engine is warned about | an unreported engine is a **FAILURE** |

The second is the newer half. Every leg reports the engine digest it is
*actually* running, and the comparator holds each to the authority's before any
case is compared — but a leg that stops reporting would otherwise degrade to a
warning printed above a green run. On your machine that is a fact about your
working tree; on a runner it means a surface dropped out of the identity check
while the gate still said OK.

Neither `engine-mismatch` nor `engine-unreported` is allowlistable. There is no
finding to triage: the comparison an entry would excuse did not happen between
the things it claims. Adding an entry for one does not silence it — it fails
*twice*, since the entry then trips the allowlist's own staleness check.

## Relationship to laterite-ags4-compliance
Separate concern, separate crate. [[laterite-ags4-compliance]] checks *findings*
agreement; this crate checks *output values*. The case-list SSOT lives here, and
compliance's harness bins never touch it — so the value gate stays slim and the
two are decoupled.

## Where it lives
`repo:rust-packages/laterite-ags4-xcheck`. Deps [[laterite-ags4-emit]] +
`laterite-ags4-validator` + `laterite-ags4-parse` — all lean leaves.

## Relationship to other components
```mermaid
flowchart LR
  emit[laterite-ags4-emit] --> xcheck[laterite-ags4-xcheck]
  val[laterite-ags4-validator lib] --> xcheck
  parse[laterite-ags4-parse] --> xcheck
  xcheck -->|value-agreement verdict| ci[output-value CI gate]
```

## Related
[[laterite-ags4-compliance]] · [[laterite-ags4-emit]] · [[crate-map]]
