---
type: decision
title: "web has no component-test layer: e2e owns rendering, extraction owns the logic"
status: accepted
tags: [design, decision, testing, web]
decided: "2026-08-21"
supersedes: []
from_gap: []
repo_refs:
  unit_lane: "repo:web/vitest.config.ts"
  the_guard: "repo:web/src/components/fix/FixPane.tsx"
  the_e2e: "repo:web/e2e/app.spec.ts"
  deps: "repo:web/package.json"
related: [playwright-e2e, testing-strategy, dec-engine-tiering, validator-site, tech-stack-wasm, coverage-campaign]
sources: []
---

# web has no component-test layer: e2e owns rendering, extraction owns the logic

## Context

`web/` tests at two altitudes and nothing between them: `vitest` over plain
modules in a `node` environment, and Playwright over the built app. There is no
way to mount a component with a controlled resource — no `@solidjs/testing-library`,
no `jsdom`, no `happy-dom` in `repo:web/package.json`, and `repo:web/vitest.config.ts`
runs with no plugins by design.

That gap has a cost, and #412 is the case that named it. Its fix in
`repo:web/src/components/fix/FixPane.tsx` guards on `report.loading` beside
`report.error`, because a Solid resource keeps its **previous value across a
refetch** — without the guard, the second file you open is labelled from the
first file's report until the new one lands. A confident badge sourced from
different bytes. The guard is Solid resource state read inside a component, so
the unit lane cannot reach it; it is locked down only incidentally, by an e2e
that happens to exercise that beat.

**And it is not one guard.** Grep `web/src/components` for `.loading` and
`.error` and the answer is a list spanning the panes and the tools, not a
special case. Whatever is decided here will be re-met, which is why #431 asked
for it to be decided once.

**The decision already existed — in a config comment.** `vitest.config.ts` says
component rendering "is exercised end-to-end by the Playwright suite". That is
this decision, written where only someone editing the test lane would find it,
and it stops short of the useful half: what to do when you *do* want something
pinned below e2e.

## Options considered

1. **Adopt a component-test layer** — `@solidjs/testing-library` plus a DOM
   environment. Buys mounting with controlled resources; costs new dev
   dependencies, a second `vitest` environment (the unit lane's whole point is
   that it is plain `node`, fast, and needs no toolchain), and CI time on a job
   that #455 already finds is the least-filtered in the workflow.
2. **Record e2e as the sanctioned altitude** for resource-state behaviour, and
   stop re-litigating it per guard.
3. **Extract the decision and unit-test that.** `sevReport()` in FixPane is a
   pure function of the resource's error/loading/value triple; lifted out of the
   component it needs no DOM at all.

## Decision

**2, with 3 as the named escape hatch.** e2e is the sanctioned altitude for
resource-state *rendering*. When a guard is worth pinning below e2e, **extract
the decision into a pure function and test it in the unit lane** — do not reach
for a mounting layer. Option 1 stays open but unbought.

## Why

Options 2 and 3 are complements, not rivals: 2 says where rendering is proved,
3 says what to do when proving it there is too coarse. Together they cover the
class without a dependency.

**Extraction is this repo's established move, not an improvisation.** #211 did
exactly this to the wasm exports — moved the logic out of the fat boundary so it
could be measured and tested directly, rather than adding a harness that could
reach into the boundary. Same shape here: the untestable thing is not the
component, it is a decision *trapped inside* one.

**What option 1 would actually buy is narrower than it looks.** e2e already
covers rendering against the real bundle, which is the failure direction the web
stack is worst at ([[playwright-e2e]] — a wrong base path, a worker that never
loads, an Arrow cell that crashes a render). A mounting layer would duplicate
that against a simulated DOM, and its unique reach — the render, as opposed to
the decision behind it — is the part e2e is already good at.

**And it is reversible cheaply.** Nothing here is load-bearing for option 1: if a
guard turns up that genuinely resists extraction, adopting the layer is adding a
dependency and a lane, with this page's reasoning to argue against rather than
a silence to guess at.

## Consequences

- A resource-state guard gets **either** an e2e beat **or** an extracted pure
  function with a unit test. "It has no seam" stops being a reason to ship it
  unpinned — extraction is always available.
- New guards do not re-open the dependency question. That is what this page is
  for; point at it.
- The unit lane stays `environment: "node"` with no plugins. A test that needs a
  DOM is a signal that the logic wants extracting, not that the lane wants
  widening.
- e2e's coverage of this class is now **deliberate rather than incidental**, so
  removing an e2e beat that is the only thing holding a resource-state guard is
  a regression, not a tidy-up.
- Revisit if a guard is found whose decision genuinely cannot be lifted out of
  the component. That case has not appeared yet; #412's could be lifted.

## Related

- [[playwright-e2e]] — the altitude this hands the class to.
- [[testing-strategy]] — the invariant-first doctrine the extracted functions
  should then be tested under.
- [[dec-engine-tiering]] — where the resource-state traps in these panes come
  from, and #359/#391's history with them.
- repo: `web/vitest.config.ts` · `web/src/components/fix/FixPane.tsx` ·
  `web/e2e/app.spec.ts`
