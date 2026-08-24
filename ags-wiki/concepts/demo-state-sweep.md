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
  notes: "repo:web/landing/demo/divergence-notes.json"
  counts: "repo:web/landing/demo/python-counts.json"
  lookup: "repo:web/landing/demo/divergence.ts"
related: [parity-model, laterite-ags4-forge, validator-site, O-53]
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

Five of the classes are each **engineered** to reach a named rule, and none of
them is the thing a reader actually does, which is type a phrase into a cell.
The sixth is: ordinary ASCII prose, inert by its content, so that any answer it
moves is attributable to the COLUMN it was typed into. That surface is four
things, all knowable from the demo's own schema — **KEY** columns (the
relational cascade), **typed** columns (Rule 8), **PA** columns (Rule 16), and
any column at all if the text is non-ASCII (Rule 1, which stays its own class
because pushing non-ASCII into a numeric column trips Rule 8 on the way).

Everywhere else arbitrary text is inert, and the sweep **names those columns**
rather than enumerating them — the exclusion rests on a prior byte-exact
measurement, not on the run, which is exactly why it is written down. An
unstated exclusion reads as a surface that was swept.

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

Then it did the more useful thing, twice, on the same finding.

Asked why one FYI was firing on nearly every state, the answer was that the
seeded delivery carried its own wording for eleven UNIT, TYPE and ABBR
descriptions rather than the dictionary's — and two of the ABBR ones were what
raised that FYI. Two typos in our own fixture, correctable, and corrected.

The second lesson is the one worth carrying, because the first reading of that
FYI recorded it as a state where laterite reported something python-ags4 did
not — and that was never true. python-ags4 raises the same FYI, with the same
message. What differed was the BRIDGE: `tools/py_ags4_check_json.py` filters
its side to `AGS Format Rule N` keys, which is exactly right for the parity gate
(whose contract is error-key parity, see [[O-45]]) and exactly wrong for a sweep
that also compares warnings and FYIs. Filtering one side and not the other does
not under-report a difference. It INVENTS one, in the direction of whichever
side was left whole, and every state the map called a divergence there was an
artifact of the instrument.

So the sweep compares both engines unfiltered (`LAT_PY_AGS4_ALL_KEYS=1`, an
opt-in the parity gate does not take), and the two remaining TRAN_AGS shapes —
[[O-45]] and [[O-53]] — are tier differences on findings both engines raise,
not silences. A difference between two engines means nothing until you can say
that both were asked the same question.

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

## The second output: what the reader is told

The map is evidence, and it is about two orders of magnitude too large to ship
to a landing page. So the same run writes a second, small file beside it —
`web/landing/demo/divergence-notes.json` — carrying one reader-facing note per
difference shape, which the demo renders at the moment someone reaches that
state (#660).

A shape with no note **fails the run**, on the same reasoning as an untriaged
shape: a difference the demo shows nothing about is indistinguishable, on the
page, from a state that has nothing to explain.

The two cases are not matched the same way, and the asymmetry is the point:

- **We report it, they do not** (and **both, at different tiers**) match on the
  RULE KEY. The finding is already on the page carrying that key, so the note
  hangs off it — and it generalises past the swept states for free, because any
  state raising the key gets the explanation.
- **They report it, we do not** has no finding of ours to hang off; that is
  precisely the case. It matches instead on the CELL the state was reached by,
  against the literal value the enumerator wrote — which is why the enumerator
  records the value and not only its equivalence class. One cell, one string
  comparison. The alternative is deciding *is this value non-ASCII for its
  declared type* in the browser, which would put a second validator on the page
  to disagree with the first.

## The third output: what the other engine COUNTED

A note says the two engines differ. It does not say what python-ags4 actually
reported, and the demo showing its own four findings beside no number at all
leaves a reader to guess. The same run writes
`repo:web/landing/demo/python-counts.json`, and the page reads its total from
there — python-ags4 is a dev-only dependency and never runs in a browser
(#673).

It is keyed on **laterite's own finding signature**: the rule-key-to-count map
the demo already has from its own run. That is a legitimate key only because
the sweep measured it to be one — the swept states collapse to far fewer
signatures than states, and python's answer is constant across all but one of
them.

Three things make it honest rather than merely convenient:

- **The one collision is resolved, not smoothed over.** Clearing `PROJ_ID` or
  any of several `TRAN` cells leaves laterite saying exactly the same thing, and
  only a blank `TRAN_AGS` earns python's extra FYI ([[O-53]]). It resolves on
  the same cell match the `they report it, we do not` notes use. A *second*
  collision that no single cell can tell apart **fails the run**, because a
  page cannot show a number it has two of.
- **A signature the sweep never measured says so.** Silence is
  indistinguishable from the two engines agreeing, which is the confusion this
  whole line of work exists to remove.
- **The signature is keyed on the tiers the DEMO shows, not the tiers the sweep
  measured.** The sweep runs both engines with every tier on, because that is
  what makes a difference mean anything; the demo's own `validate` call takes
  the wasm default of FYI off. A swept state that raised a laterite FYI would
  put python's total beside a laterite total the page is not showing, so it
  **fails the run** rather than being written. Nothing checked that before, and
  it was true only by accident.

Two designs were measured and rejected, so they do not get retried. **Hashing
the delivery text** is exact but over-strict: typing a project name changes the
bytes while changing neither engine's findings. **Per-lever addition** is simply
wrong — levers do not compose, and orphaning a `SAMP` row then deleting the
`LOCA` group predicts one more finding than the measured answer.

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
