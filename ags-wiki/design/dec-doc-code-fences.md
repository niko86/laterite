---
type: decision
title: "A page's code fences are one program per surface, executed nightly"
status: accepted
tags: [design, decision]
decided: 2026-08-21
supersedes: []
from_gap: []
related: [design/_README, concepts/docs-site, dec-duckdb-extension]
sources: []
---

# A page's code fences are one program per surface, executed nightly

## Context

`gen_doc_outputs.py` polices the `text` block that shows what an example
**prints**: every one is an include or a declared opt-out, an opt-out carries a
reason, and orphans are counted. Nothing policed the fence that shows what a
reader **runs**.

A walkthrough of the docs site against the released packages found the
consequence. Two cookbook pages call `ags.sql(...)` where `ags` is bound nowhere
— not on the page, and not by the `--8<--` include above, which binds `rel` and
`df` and calls `laterite.read(...)` directly. A reader working top to bottom gets
`NameError`. A third page constructs `TranStamp(...)` with no import. All three
had been there long enough that nobody could say when they landed, because no
gate could see them.

Measured across the corpus at the time of writing: of the fences typed directly
onto pages rather than included from a file, **19 of 20 Python ones continue from
the fence above them** — they use names an earlier block bound. Only one stands
alone.

## Options considered

1. **One program per fence.** Each must be self-contained. Simplest to run, and
   better for a reader landing mid-page from a search engine.
2. **One program per page, per surface.** Concatenate the include and every
   following fence of that language, in document order, and run once.
3. **Leave it.** Accept that page snippets are illustrative.

## Decision

**Option 2**, with these commitments:

- **Opt-out, not opt-in.** An unmarked fence is meant to run. The escape hatch is
  `<!-- doc-code: skip — reason -->`, spelled to match the existing
  `doc-output: skip` so there is one convention with two halves, and the reason
  is enforced.
- **Executable set: python, sql, js/ts.** `bash` is excluded — see below.
- **Document order, ignoring tab boundaries.** A page's Python tab include and a
  Python fence further down the page are one program.
- **Structural check on every PR; execution nightly.** The structural half is
  buildless, so it rides the unfiltered `wiki-lint` lane.
- **"Does not raise" is enough** for a continuation. The gate reports how many
  ran without asserting anything rather than requiring assertions.

## Why

**Per page, not per fence**, because the corpus decided it: 19 of 20 continuations
means option 1 is not a policy but a rewrite of nearly every fence on the site.
The cost asymmetry is the whole argument — option 2 costs a marker on the rare
divergent fence, option 1 costs an edit to almost all of them.

**Opt-out**, because every defect this addresses exists through something not
being remembered. `test_docs_examples.py` already argues the same way about its
glob: *"Discovery is by glob, deliberately — a new example is covered the moment
it is added, with no list to forget to update."* An opt-in marker would let a new
fence land uncovered and silent, which is the state being left behind.

**bash excluded**, because a page's `bash` fence is an install instruction, and a
gate that executed `pip install laterite[compat]` would rewrite the machine it
runs on. The exclusion is recorded rather than implied so the next runnable
`lat …` fence does not land assuming cover — and that is not hypothetical. Of
the fourteen excluded fences, **five are real `lat` invocations** which fail to be
runnable only because they name placeholder paths. They carry a distinct reason
saying so; they are candidates for coverage, not permanent exclusions.

**Execution nightly**, because a broken snippet is not shipped code — nothing
downstream breaks — and the nightly `docs-vs-released-*` legs test against the
**released** artefact, which for documentation is the more meaningful target: a
reader installs the release, not the working tree. The structural half stays on
every PR because failing to classify a fence is a PR-time event, and finding it
the next morning means working out which PR introduced it.

## Consequences

- A new fence in a runnable language is covered from the moment it is added, and
  a new `bash` fence fails the build until it says why it is excluded.
- The gate prints its own coverage on every run — includes, inline, skipped,
  prose. **`inline` is the number that matters**: fences a reader can copy that no
  gate has yet executed. It falls as the per-surface runners land, and stating it
  is what stops "structurally classified" reading as "known to run".
- `cookbook/index.md` could no longer claim the snippet on the page is the exact
  file CI executes. It now says the opening block is, and that later snippets
  continue from it — which is also the more useful thing for a reader to know.
- SQL fences must name a real fixture before they can run. They currently use
  `delivery.ags`, which is not one.
- This commits the tool to being more than a generator: it now also reports on
  input it never rewrites.

## Related

`repo: tools/gen_doc_outputs.py` · `repo: tests/test_doc_code_fences.py` ·
`repo: tests/test_docs_examples.py` ·
`repo: rust-packages/laterite-node/test/docs-examples.test.ts`
