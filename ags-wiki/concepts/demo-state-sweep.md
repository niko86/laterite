---
type: concept
title: "demo state sweep: what both engines say about every state the demo can reach"
status: drafted
tags: [concept, testing, parity, process]
volatile: [difference-shapes, count-differences]
volatile_asof: 2026-08-24
ags_editions: []
repo_refs:
  enumerator: "repo:web/scripts/sweep-demo-states.mjs"
  generator: "repo:tools/gen_demo_state_map.py"
  model: "repo:web/landing/demo/delivery.ts"
  map: "repo:web/landing/demo/state-map.json"
related: [parity-model, laterite-ags4-forge, validator-site]
sources: []
---
# demo state sweep: what both engines say about every state the demo can reach

## Definition

The landing demo tells a reader that laterite and python-ags4 answer
differently in places. The **state sweep** is what holds that claim to
evidence: it walks the states the demo can reach, runs **both** engines over
each one, and checks the answer into the repository as
`repo:web/landing/demo/state-map.json`. The demo can then explain a difference
without ever running python-ags4 in a browser, and a validator change that
moves a demo state cannot land silently.

It exists because the claim was being made from a handful of hand-checked
states. A handful is not evidence about a few hundred.

## The two halves, which are different in kind

`repo:web/scripts/sweep-demo-states.mjs` **enumerates and emits**. Every file it
produces goes through the demo's own `emit()`, loaded through Vite rather than
reimplemented — so each state is provably reachable rather than merely
plausible. A copy of the emitter would make the map a claim about the copy.

Two assertions hold that, rather than a comment claiming it. Before anything is
emitted, `emit(SEEDED)` is compared against the checked-in seed file and the run
stops if they differ. And deleting each editable group and restoring it must
return to the seed **exactly** — which is `restoreGroup`'s real claim, and the
reason it is not enumerated as a lever: it reaches no new state, it returns to
an old one, and that is the stronger thing to check.

`repo:tools/gen_demo_state_map.py` **dual-validates and triages**. It runs
[[laterite-ags4-forge]]'s `check` over the emitted directory — one command that
runs both engines and reports per-rule counts — matches every difference
against a known documented one, and renders the map.

## What "every state" can honestly mean

**Not everything in the file.** A lever is enumerated only where a reader can
pull it. The demo renders an editable table for the groups in its own schema and
no others, so the delivery's remaining groups are visible but not mutable — no
cell edit, no row add, no group delete. Enumerating them would put states in the
map that nothing can produce, which is the same failure as leaving reachable
ones out: membership has to mean something. They are named in the map's
`groups_not_enumerated`, because a silent exclusion is indistinguishable from an
oversight.

**Not the product of the levers.** Group-present x row-present alone is
exponential in the seeded delivery's group and row counts, and each of those
states costs a python-ags4 subprocess. Nothing exhausts that.

The counts live in the map, which recomputes them on every run, and not in this
sentence. An earlier draft of this page put a figure here; narrowing the
reachable set to the groups the demo actually renders as editable made it wrong
the same day, and nothing would have gone red.

The sweep is exhaustive to **depth 1**: every state one lever from the seed —
each group deleted, each row deleted, a row added to each group, and each cell
set to each value class that can apply to it. Deeper states are covered only by
a small **named** set of sequences: the ones the demo's own teach loops walk a
reader through, which are the deeper states a reader actually reaches.

That bound is printed on every run and carried in the map's `scope` and
`not_covered` blocks. It is not a footnote. A map that quietly stopped at depth
1 would read as "the demo has been swept" when it means something much
narrower — the failure mode CLAUDE.md's *a gate that drops input says what it
dropped* exists to prevent.

## Why cell VALUES are classes, not values

`setCell` takes free text, so the values are unbounded — dropping the editable
whole-file pane bounded the **file**, not the **cell**. What is bounded is what
a value can MEAN to the validator, and the findings key off that: two values in
the same class produce the same rule set. Each class is named for the rule it
exists to reach, and applicability is derived from the demo's own schema (KEY,
REQUIRED, the declared TYPE) rather than hardcoded per cell.

A value outside every class is outside the map, and the run says so.

## Why a difference must be triaged, not merely recorded

A map that lists differences without saying what they ARE is a list of things
to re-investigate every time someone reads it. Every difference **shape** — the
pair (rules only laterite raised, rules only python-ags4 raised) — is matched
against a known one carrying its O-N. An unrecognised shape **fails the run**
rather than being written down: it is either a new record to write or a defect
in one of the two engines, and both need a person.

The generator earned that guard twice on the day it was written. First it
rejected a shape that turned out to be the declined-parentage warning added
hours earlier: a validator change had moved four demo states, and the sweep is
what noticed.

Then it did the more useful thing. Asked why one FYI was firing on nearly every
state, the answer was that the seeded delivery carried its own wording for
eleven UNIT, TYPE and ABBR descriptions rather than the dictionary's — and two
of the ABBR ones were what raised that FYI. So the demo's claim to differ from
python-ags4 almost everywhere was an artifact of two typos in our own fixture.
Correcting them moved the map from one difference in nearly every state to five
in total. A sweep is worth having partly because it makes a claim like that
checkable at all; the number was never wrong, but what it MEANT was.

## Reading the map

Two numbers matter more than the state list, and both are in the map rather
than here so they cannot go stale in prose.

**Difference shapes** should stay a handful; if the count climbs, the two
engines are drifting apart in a way the demo will have to explain.

**Count differences** — the same rule key with a different count on each side —
should stay empty. The engines agreeing on which rules break but not on how
often is a louder signal than a tier difference, and no O-N explains one.

`python_ags4_version` is in the map because the map is a claim about a specific
version of the other engine, not a timeless one.

## Running it

```bash
# Two prerequisites, each refused by name with this exact line if missing:
(cd rust-packages && cargo build --release -p laterite-ags4-forge)  # runs BOTH engines
(cd web && npm ci)                                   # Vite resolves the demo's seed

uv run --no-sync python tools/gen_demo_state_map.py           # regenerate
uv run --no-sync python tools/gen_demo_state_map.py --check   # the drift gate
```

A drifted `--check` says which of three things happened, because they need
different responses: a state's answer moved (the `states` block changes), the
other engine moved under us (`python_ags4_version` changes), or the reachable
set itself changed (`counts` and `scope` change).

The gate is its own job in `parity.yml` rather than a step in `parity`: "we
regressed" and "a demo state moved" are different signals, and sharing a job
collapses them into one red X. It is also much lighter than `parity` — it needs
the `python_ags4` package, not the clone of upstream's test suite.

## Related
[[parity-model]] · [[laterite-ags4-forge]] · [[validator-site]] · [[laterite-ags4-parity]] · [[observations-coverage-map]] · [[mutation-sweep]]
