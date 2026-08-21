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

## Executing them: two things the first run forced

**A seeded working directory, not the repo root.** Pages say `delivery.ags` — the
site's narrative filename, in 31 places — and rewriting every one to a fixture
path would trade the docs' voice for the gate's convenience. Each page program
runs in a temp directory holding `delivery.ags` and an `examples/` copy instead,
so page text and executed text stay identical and only the environment is
prepared. That is the same latitude `test_docs_examples.py` already takes by
running from the repo root.

It also closed a hazard nobody had noticed: `delivery.ags` EXISTS at the repo
root as a gitignored working artifact holding only `PROJ`, so a first run from
there gave `cookbook/read-a-group.md` a `KeyError` on a missing `LOCA` rather
than the `FileNotFoundError` CI would have produced. A gate whose result depends
on an untracked file is not a gate.

`delivery.ags` is seeded from `examples/sample_strata.ags` — `sample_site` plus a
`GEOL` group — because `cookbook/sql-across-groups.md` documents a three-way join
through `GEOL`, and a fixture without it would make a working capability look
broken. **The fixture serves the docs, not the reverse**: when the page selected
`GEOL_GEOL`, the fixture grew that heading rather than the page being edited to
match what the fixture happened to have.

**Three classes of failure, and only one is a bug.** The first run failed eight of
sixteen pages. They sorted into: snippets that are simply *wrong* and a reader
hits (fixed); snippets that are right but name files the docs describe rather
than ship, like `phase1.ags` (the defect inside them still fixed, the fence
skipped, because merging the fixture with itself would assert nothing); and
snippets that name placeholders *on purpose* to show an API shape, like
`read(data=raw_bytes)` (skipped — binding them would obscure the lesson).
Collapsing those three into "8 failures" would have produced eight edits, most
of them wrong.

## The SQL half: a second runner, and what it found

**Two runners, and a table saying which owns a language.** SQL page programs run
in `tests/test_docs_duckdb_examples.py`, not in `--run-pages`. That module already
owns the connection, the env gating and the "which extension is under test"
reporting; and the `duckdb` surface in the tool shells out to the DuckDB CLI,
which `pip install duckdb` does not ship — routing SQL through `--run-pages` would
print a SKIP line on every machine for a language that is in fact gated, which
reads as a hole where there is a handoff. `PAGE_RUNNER` names the owner per
language, `None` meaning pending, and the census prints it: **adding a language to
the runnable set forces the same edit to say who executes it.**

**Seeding superseded the plan to rewrite the SQL fences.** The grilling that
settled this decision agreed to rewrite the hand-written statements to the real
fixture before running them. Seeding made that unnecessary — the fences
run unedited — so the rewrite never happened, and the fences that *were* edited
were edited because they were wrong, not because the gate needed them to be.

**The split had to learn what a comment is.** Statements are separated on `;`, and
dropping any chunk that starts with `--` looked like the way to skip comments. It
drops the query *underneath* the comment, and introducing a query with a comment
is how these pages teach — so `duckdb/index.md`'s first example never ran. It was
one of the two zero-row queries: every borehole in the fixture is above sea level,
and the page asked for `loca_gl < 0`. A gate that skips its subject is worse than
no gate, because it reports green.

**Zero rows is reported, not failed.** "Does not raise" is a weaker bar here than
the `-- expect-rows: N` the example *files* carry — a query can bind and return
nothing, which is exactly how those two defects survived. The runner counts
statements that ask for rows and got none, and prints the count; only statements
that *ask* are counted, because `INSTALL` and `LOAD` return nothing by definition
and a report that cries wolf on its own preamble is one nobody reads the day it
means something.

## The Node half: a surface with two names, and a page that was broken

**A surface, not a language, is the unit.** Node answers to both ```js and
```javascript, and building one program per TAG would have handed the second half
to Node without the first half's imports — a failure about this tool rather than
about the page. The runner groups by surface and filters to the tags routed to
it, which are not the same cut: `ts` reaches the Node surface and is still
pending, so a surface can be half-claimed.

**`ts` stays pending, and not for want of a runner.** A TypeScript fence's
package is decided by the PAGE, not by the tag: the only one in the corpus is a
type-only import on `reference/wasm-api.md`, whose package is the browser one.
Running it under the Node surface would answer a question nobody asked, and a
page-to-surface rule built for a single type declaration would be machinery
bought for a case that does not exist. The census reports it as pending on every
run, which is the claim that stays true either way. #519 carries the three ways
to close it — page-to-surface routing, a type-only fence class, or type-CHECKING
rather than executing — and the choice between them is a decision rather than an
implementation detail, which is why it is on the tracker and not in this page.

**Two fences were marked because their twins already were.** Both Node tabs on
`cookbook/merge-deliveries.md`, and the Node tab of an already-marked block on
`concepts/certificate-lifecycle.md`, sit beside Python fences that opted out with
a reason in the previous step. Nobody had judged them differently — there was no
js runner to notice them. That is the opt-in argument made concrete: the marker
went on the half that a gate could see.

**One page was not illustrative, and was edited.** `node/index.md` declared
`const file` twice — a `SyntaxError` in ESM, so a reader following the page top to
bottom gets nothing at all. A skip marker there would have claimed the fence was
illustrative, and it is real, runnable code; the validated handle is named
separately instead, which is also what the section is about. **The marker records
that a fence is not meant to run. It is not a way to quieten a page that is
broken.**

The same page asked for ground level below sea level, which every borehole in the
fixture is above — the empty-result defect the SQL half found twice. **No gate
forced this one**: the Node bar is "does not raise", and a query returning nothing
raises nothing. It was fixed because it was found while fixing the fence beside
it, and it is recorded here because a threshold nobody explains reads as
arbitrary to whoever meets it next.

**What "does not raise" still cannot see, stated rather than discovered later.**
`reference/node-api.md` branched on `file.report.ok`. A `Report` has no `ok` — it
carries `isValid`, the separate `count == 0`; `ok` belongs to the fix report and
the un-validatable failure report. So `!file.report.ok` is `!undefined`, always
true, and the documented example rewrote `clean.ags` for a **clean** file. The
page program runs it without complaint, because taking the wrong branch raises
nothing. The SQL half bought a cheap partial answer to the same blind spot by
counting rows; there is no equally cheap equivalent here, and pretending
otherwise is what would make the gate's green misleading. It was found by reading
the shipped `index.d.ts` after the runner brought attention to the page — which
is the honest description of what this class of gate does: it gets you to the
page, not to the defect.

**Both ends of a failure, because runtimes disagree about which one matters.** The
failure display kept the last four lines of stderr, which is where a Python
traceback puts the exception — and where Node puts loader frames. The first run
of the Node programs printed four lines of `node:internal/modules/esm/loader` and
hid every `SyntaxError` that said what was wrong. It now keeps both ends and says
how many lines it dropped.

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
- SQL page programs run against the **community-published** extension, and Node
  ones against the **published npm package** — each in the nightly leg that
  already asks that reader's question, so a page and the artefact a reader
  installs are checked together. The Node step follows the same `node_modules`
  symlink its neighbour re-points, so there is no second swap to keep in step.
- What remains unrun is `ts` alone, and `PAGE_RUNNER` says so on every structural
  run rather than leaving it to be inferred from silence.
- **A runner that finds no pages fails.** Zero is the one result a green run
  cannot mean: a fence-regex change, a routing-table typo or a moved docs
  directory each empty the loop, and each would otherwise exit 0. This is
  `test_docs_examples.py`'s vacuous-glob guard, finally extended to the page
  half — the issue behind this work named that precedent and it had been left
  unapplied through three of the four steps.
- The nightly step is fatal only when the checkout matches the released tag. It is
  amnestied when the tree is AHEAD, because the leg's banner promises that and
  `test_nightly_wiring.py` enforces it — a page program breaking because this tree
  moved ahead is a fact about the run, not about the page. The skip marker records
  an author's intent; it is not for the calendar.
- This commits the tool to being more than a generator: it now also reports on
  input it never rewrites.

## Related

`repo: tools/gen_doc_outputs.py` · `repo: tests/test_doc_code_fences.py` ·
`repo: tests/test_docs_duckdb_examples.py` ·
`repo: examples/sample_strata.ags` · `repo: .github/workflows/nightly.yml` ·
`repo: tests/test_docs_examples.py` ·
`repo: rust-packages/laterite-node/test/docs-examples.test.ts`
