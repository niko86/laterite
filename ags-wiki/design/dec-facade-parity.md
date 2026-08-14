---
type: decision
title: "The Rust facade reaches parity and joins the product line — Excel joins it, the CLI does not"
status: accepted
tags: [design, decision, api, rust, crates-io, versioning]
decided: 2026-08-04
supersedes: []
from_gap: []
related: [dec-rust-api-crates-io, modality-register, api-surface-1.0, crate-map, dec-ags4-merge-semantics, surface-census, dec-beta-claim]
sources:
  - "https://doc.rust-lang.org/cargo/commands/cargo-install.html"
  - "https://doc.rust-lang.org/cargo/reference/publishing.html"
repo_refs:
  facade: "repo:rust-packages/laterite/Cargo.toml"
  register: "repo:modality.json"
  floor: "repo:tools/gen_modality.py"
  version_gate: "repo:tests/test_version_faithful.py"
  publish_tool: "repo:tools/publish_crates.py"
  cli: "repo:rust-packages/laterite-cli/Cargo.toml"
  excel: "repo:rust-packages/laterite-ags4-excel/Cargo.toml"
---

# The Rust facade reaches parity and joins the product line — Excel joins it, the CLI does not

> **DECIDED 2026-08-04.** Revises [[dec-rust-api-crates-io]], which otherwise
> stands: the Angle C API design, the publish waves, the crate metadata work and
> the `laterite-duckdb` migration are all unchanged. Reopened here are the
> **0.2 milestone**, the **CLI publish**, and — via the [[modality-register]]
> floor — the **Excel exclusion**, which brings a crate rename with it.

## Context

[[modality-register]] gave the facade's absences a shape. Its floor — owner-set
2026-08-04 — is that the crate exposing the engine directly should offer at least
what the **weaker of its two dependents** offers, per capability: the
intersection of Python and Node. Computed, never stored.

That turned "the facade is incomplete" from a feeling into a list. At the point
this page was written:

```
2 clear · 11 below · 9 to add · 2 deliberately not adding
```

Three facts fell out of building against that list, and each changes the plan.

**First: `diff` and `merge` are not a coding task.** They are ready — metadata,
public-API snapshots, packaging allowlist, dependency waves all in place — and
held only by two deliberate flags (`publish = false` in each manifest, and the
`DEFERRED` set in the publish tool). Nothing needs building; a decision needs
taking.

**Second: the facade's version plan and its parity rule disagree.**
[[dec-rust-api-crates-io]] says diff and merge ship in a facade **0.2**. The
version gate says something else, and it is executable:

> When it reaches parity with the Python and Node surfaces it joins the PRODUCT
> line, jumping 0.1.x straight onto that number. […] the jump off 0.1.x is a
> one-way door and should not happen by accident.

Both hold only if the work stops halfway.

**Third: the two by-design exclusions were justified by a door that is shut.**
The register's note on the rust `to_excel` cell tells a Rust caller to
`cargo add laterite-ags4-excel` — a crate that is `publish = false` and has never been
on crates.io. An exclusion resting on a false premise is not an exclusion, and
re-examining it is what produced decision 4.

## Decisions

### 1 — Complete all eleven capabilities, then jump

`transport-pack`, `transport-lock`, `certify`, `cert-input`, `read`-from-cert,
`fix`, `build`, `diff`, `merge`, `to_excel`, `from_excel`. When the facade is
level with its floor it leaves 0.1.x for the product line.

Eleven, not nine, because decision 4 below retires the two exclusions this page
was originally written to accommodate.

### 2 — There is no facade 0.2

Superseding *"diff and merge ship in 0.2, not 0.1"*. Shipping diff and merge
alone would leave `fix`, `build`, `certify`, `cert-input` and both transport
verbs still missing — a 0.2.0 that exists as a waypoint for as long as the
remaining work takes, on a crate whose whole purpose is to be a stable surface.
The parity rule already describes the destination; honouring it means going
there once.

### 3 — `diff` and `merge` publish at engine 0.9.0

They are new crates at the existing engine number, so no tier moves to admit
them. Flip both manifests and empty `DEFERRED`.

The cost this incurs is the one the deferral was buying, and it is worth stating
precisely rather than as "the API gets frozen", which is not what happens.
crates.io makes a **version** immutable, not an API: `laterite-ags4-diff 0.9.0`
will exist with exactly that content forever (`cargo yank` stops new dependents
resolving to it but does not remove it, and deletion is possible only within 72
hours and under download thresholds). Publishing 0.10.0 with a different API is
entirely allowed.

What actually changes is that **we** start policing it. Semver is a convention;
`cargo semver-checks` and the public-API snapshot are gates this repo chose to
run. After publishing, altering these two crates stops being a free in-tree edit
and becomes a versioned, reviewed act — and once anything depends on them, a
break costs someone outside this repo.

That is the real reason to be careful here rather than casual: merge's surface is
still under design. [[dec-ags4-merge-semantics]] holds the semantics, and
laterite#162 records a constraint the typed merge API must respect — no
provenance typestate, because it forces every document in one merge to share a
provenance and rules out merging a disk file with bytes off a network. What
publishes now is `merge_parsed`, the engine entry point the facade wraps, whose
`&[ParsedFile]` shape is exactly the one #162 endorses. The typed surface it
warns about is not being published and stays free to design.

### 4 — `laterite-ags4-excel` is published, and the facade gates it behind `excel`

Superseding the register's own `by-design` verdict on `to_excel`/`from_excel`.
That verdict rested on a premise that is **not true**:

> "A Rust caller can `cargo add laterite-ags4-excel` directly, which is a door a
> python or node user has no equivalent of, so the floor's premise (the facade
> is the only way in) does not hold here."

`laterite-ags4-excel` is `publish = false` and has never been on crates.io. The escape
hatch the exclusion was justified by does not exist, so the note had to change
whichever way this went — either the crate gets published and the sentence
becomes true, or the sentence goes.

**Both, as it turns out.** The crate is published *and* the facade gates it:

```toml
[dependencies]
laterite-ags4-excel = { workspace = true, optional = true }

[features]
excel = ["dep:laterite-ags4-excel"]
```

The cost of an optional dependency is not what "dep heavy" suggests. A consumer
who does not enable `excel` never downloads, compiles or locks `calamine` and
`rust_xlsxwriter` — the weight [[crate-map]] extracted the crate to avoid stays
avoided, by the same mechanism, for the same people. The one caveat is Cargo's
feature unification: if *any* crate in a build turns the feature on, everything
in that build gets it. For an application graph that is a non-issue.

The real cost lands on **us**, not on consumers: `laterite-ags4-excel` joins
`PUBLISH_SET`, and with it the semver gate, the public-API snapshot and the
packaging-contents gate, permanently, on an eleventh engine crate. (Written as
"a tenth crate"; the engine tier was already ten crates when this was decided —
`CHANGELOG.md` 0.10.0 says so — with diff and merge in `PUBLISH_SET` but held
from the registry. Excel is the eleventh either way you count.)

Three findings make that cost affordable:

- **No third-party type is public.** Every signature is `&Path`, `&[u8]`,
  `Vec<u8>`, `bool`, `Option<Vec<String>>`, its own `ExcelStats`, or
  `CliError`/`ReadOptions` from the already-published `laterite-ags4-core`. The
  Angle C rule holds without a wrapper, so prep is metadata, not a rewrite.
- **The dependency shape is already right.** `[workspace.dependencies]` carries
  `default-features = false` on core, so excel's two `path`-only sites become
  `{ workspace = true }` and inherit correctly — no inline version to go stale.
- **Wave 5.** It depends on core (wave 3) and emit (wave 4), so it lands beside
  `laterite-ags4-merge` and adds no wave.

**What is knowingly accepted.** The crate's own header flags it *"FLAGGED FOR
REWRITE: the current contents are AGS4-specific; the intent is to grow this into
a proper general-purpose Excel library."* Publishing means publishing a surface
already called wrong, and the rewrite will be a breaking version. That is
accepted rather than overlooked, on two grounds: the facade wraps it opaquely, so
the rewrite is invisible to every facade user, and a breaking bump on a crate
with no external dependents costs a version number. Its `description` says so
plainly, so nobody adopts it expecting stability.

**A consequence for the API gate.** `cargo public-api` and `cargo semver-checks`
see only the features they are run with, so a default-off `excel` would put
`to_excel`/`from_excel` outside the snapshot that guards the crate — and outside
the reflector that #250 taught to read it. The facade therefore keeps **two**
snapshots, `laterite.txt` (default) and `laterite.all-features.txt`, both gated,
with the modality reflector reading the union.

### 5 — The crate is renamed to `laterite-ags4-excel` before it publishes

A crates.io name is free until its first publish and irreversible after, so this
is a now-or-never edit — and the workspace's own convention says it is wrong
today. `-ags4-` marks the engine tier: `laterite-ags4-core`, `-parse`, `-emit`,
`-diff`, `-merge`, `-validator`, `-types`, `-reference`, `-trust`. Bare
`laterite-*` is for format-agnostic crates, of which `laterite-transport` is the
only genuine one. `laterite-excel` sits in the second group holding contents its
own header calls AGS4-specific, under a name chosen for a general-purpose
destination it has not reached.

This repo has made this exact decision once already. #199 renamed
`laterite-ags4-check` to `laterite-cli` with the same argument — *"a name is free
until its first publish and irreversible after"* — and stated the convention
being applied here: *"`-ags4-` marks the engine tier, which a product binary is
not."* This crate is engine tier, so the rule points the other way for it.

**The old name is not preserved anywhere.** Owner-decided 2026-08-04: rename
repo-wide, including the historical records — `changelog.json`'s past release
entries, `observations.json`'s O-49 decision field, and the `mutation-sweep.json`
ledger row. The cost is accepted knowingly: release notes for 0.9.x will name a
crate that shipped under the other name, and a past O-N decision field is edited
in place. The gain is that one grep gives one answer and there is no exception
list to carry forward. #199 froze its history by contrast, but had almost none to
freeze — the name never reached a release note, an observation or a ledger row.

Two mechanics this constrains:

- **Edit the SSOT, never the render.** `CHANGELOG.md`, `OBSERVATIONS.md` and
  `mutation-sweep.md` are generated. The rename edits the three JSON files and
  regenerates; a blanket substitution across rendered files is reverted by the
  next generator run and fails its `--check`.
- **`CHANGELOG.md:64` needs more than a rename.** It records `laterite-ags4-excel` as
  *"never considered for the registry"*, which phase 6 falsifies whatever the
  crate is called. The repair is a new entry at the phase-6 release, not a
  rewrite of the old one.

### 6 — `laterite-cli` is NOT published to crates.io

Superseding *"the CLI ships with 0.2, alongside the diff/merge publish it already
depends on"*. Decided against, not deferred — not-publishing is the reversible
direction, and leaving it "deferred" keeps dragging the CLI tier into every
future scope conversation.

**The capability already exists.** `publish = false` blocks `cargo publish`, not
`cargo install --git`. This works today, against this repo, with nothing
published:

```
cargo install --git https://github.com/niko86/laterite laterite-cli
```

**And what publishing adds is the mode we would least recommend.** `cargo
install` fetches source and compiles locally — 25 direct dependencies including
`clap`, `ratatui` and `crossterm`. The other two channels ship prebuilt:

| channel | what the user gets |
|---|---|
| `pip install laterite` | prebuilt wheel — no compiler |
| `npm i laterite` | prebuilt native addon |
| `cargo install laterite-cli` | source, compiled locally |

So crates.io would not be a missing route to `lat`; it would be the worst of the
three. For a target the release matrix does not build, the better answer is to
add the matrix line and ship a **binary** — a few lines of YAML, on request —
rather than publishing so that someone can compile it themselves.

**The price avoided.** Publishing the CLI means publishing `laterite-cliutil`
and `laterite-ags4-excel` too (a binary's dependencies are still dependencies, and
publishing strips `path`). Neither is prepared: no `repository`, no `keywords`,
no public-API snapshot, absent from `PUBLISH_SET` — so both would also enter the
semver and public-API gates, which is a standing maintenance cost on two crates
nobody outside this repo has asked for. It also needs a **gates
exemption**, because `cargo public-api` and `cargo semver-checks` both require a
lib target and `laterite-cli` is bin-only — and `PUBLISH_SET` is the single list
those gates and the packaging gate share, so the exemption gives one list two
meanings. And it would put `clap`/`ratatui`/`crossterm`/`calamine` on a published
dependency surface, which is the weight the crate split exists to keep off
library consumers.

`lat` continues to reach people as it does today: per-target binaries from the
release workflow, and the `lat` console script that rides in the wheel.

### 7 — "Parity" means `0 below`, with no by-design exclusions

This decision was first drafted the other way round. Because `to_excel` and
`from_excel` were permanent exclusions, the facade would sit *always* two below
its floor, so a gate asserting `0 below` could never pass and parity had to be
defined as the narrower `to add == 0` — a carve-out written into the rule to
accommodate exactly two cells.

Having to write that carve-out is what sent us back to the exclusion, and the
exclusion did not survive the reading (decision 4). With it gone the census has
no `deliberately not adding` column left to consult, and parity is the plain
statement: **the facade is level with the intersection of Python and Node.**

That is worth the extra crate on its own terms. A gate with an exemption is a
gate that has to be explained every time it is read, and an exemption sized to
today's exclusions is one that quietly stops matching when the exclusions change.

## Phasing

One constraint orders everything: **a published crate cannot depend on an
unpublished one.** Four of the eleven capabilities sit behind a crates.io
publish, and a publish is the owner's act, not a PR. Two independent publish
tracks fall out of that — diff/merge, which need only a flag flip, and
Excel, which needs a rename and full prep first.

| phase | what | caps | blocked by | done |
|---|---|---|---|---|
| **0** | this page · register cells · modality drift gate | — | #250 | 2026-08-04 |
| **1** | `publish = true` on `diff` + `merge`, empty `DEFERRED` | — | — | 2026-08-04 |
| **2** | **publish `diff` + `merge`** at 0.9.0 — owner | — | 1 | 2026-08-05 |
| **3** | rename `laterite-excel` → `laterite-ags4-excel`, repo-wide | — | — | 2026-08-05 |
| **4a** | facade `transport`: `pack` / `lock` | 2 | — | 2026-08-05 |
| **4b** | facade cert trio: `certify`, cert-input, read-from-cert | 3 | — | 2026-08-05 |
| **4c** | facade `fix` + `build` | 2 | — | 2026-08-05 |
| **4d** | facade `diff` + `merge` | 2 | 2 | 2026-08-06 |
| **5** | Excel publish prep · second facade snapshot | — | 3 | — |
| **6** | **publish `laterite-ags4-excel`** at 0.9.0 — owner | — | 5 | — |
| **7** | facade `excel` feature: `to_excel` + `from_excel` | 2 | 6 | — |
| **8** | **the jump** — facade onto the product line | — | 0–7 | — |

Phases 1 and 3 are independent of each other and of everything in 4; 4a–4c are
independent of both publish tracks.

**Phase 0 carries the drift gate.** `repo:tools/gen_modality.py` renders
`ags-wiki/concepts/modality-register.md` with nothing asserting the two agree —
`gen_observations.py` has `--check-wiki` in CI and this generator has no
equivalent, so the rendered register can diverge from its SSOT silently. It goes
in the PR that regenerates the file, falsified by hand-editing the render and
watching the gate fail.

**Phase 1 is a flag flip, which is why it goes first.** Both crates were prepared
in full — metadata, snapshots, packaging allowlist, wave placement — and held
only by `publish = false` and the `DEFERRED` set. Nothing is built here.

**Phase 3 must precede phase 5, and edits SSOT rather than renders.** See
decision 5: `changelog.json`, `observations.json` and `mutation-sweep.json` are
the files that change, then regenerate. The wiki tool page moves to
`ags-wiki/tools/laterite-ags4-excel.md` with its inbound `[[links]]`, and the
index is regenerated. Roughly 46 files.

**Phase 5** is one PR — the wave plan changes as a unit, and a third of it would
not show whether it still resolves. It adds the metadata (`repository`,
`keywords`, `categories`, `readme`), converts the two `path` dep sites to
`{ workspace = true }`, flips `publish = true`, adds the crate to `PUBLISH_SET`
and generates its public-API snapshot. Proof is `publish_crates.py --dry-run`
placing it in wave 5, beside `laterite-ags4-merge`.

The second facade snapshot (`laterite.all-features.txt`, decision 4) belongs
**here**, not in phase 7. At phase 5 the facade has no features, so the new file
is byte-identical to the existing one — which is the point: the gate goes in
while it is provably inert, and phase 7's diff then shows nothing but the Excel
surface. A gate arriving in the same PR as the thing it guards has never been
observed failing, and this repo has been bitten by that once already — the rust
cells joined the register in #241 with no reflector at all.

**Phases 2 and 6 are the owner's.** `repo:tools/publish_crates.py` needs a
crates.io token, which is not the agent's to hold. Everything around them is
reviewable in PRs; these two are commands run by someone with credentials.

**Phase 4d** carries the [[dec-ags4-merge-semantics]] and laterite#162
constraint: the door wraps `merge_parsed`'s `&[ParsedFile]` shape, and no
provenance typestate reaches the facade.

**Phase 6 needs a changelog entry that corrects a claim**, not just announces a
publish. `CHANGELOG.md` records `laterite-ags4-excel` as *"never considered for the
registry"*; that becomes false here and the correction belongs in this release's
notes.

**Phase 8 is a one-way door** and stays its own reviewed PR: the facade version
moves to the product number, `repo:tests/test_version_faithful.py` converts its
`startswith("0.1.")` exemption into a parity assertion, `bump-version.sh product`
gains the facade as a stamped surface, and the floor gate starts asserting
`0 below`. None of that should ride in on the back of the last capability
landing.

**Phase 8 also retires four documents**, added 2026-08-14 with the versioning
revision on [[dec-rust-api-crates-io]]. That revision says the facade has its own
clock and is outside the beta claim — true while three clocks run, and *actively
wrong* the moment this phase makes it two. It is a deletion list, not a rewrite:

- the facade paragraph in `repo:web/docs-site/docs/reference/support.md`
  ("How versions move") goes, and the crate's row joins the beta table above it;
- the caveat block at the top of `repo:rust-packages/laterite/README.md`;
- the second clause in `repo:rust-packages/laterite/src/lib.rs` — the "nothing
  absorbs this crate" paragraph beneath the four API rules;
- the facade folds into `repo:changelog.json`, which is also where laterite#319
  (a facade `CHANGELOG.md`) lands rather than becoming a second file.

Left in place: the qualifier on the *first* clause ("a promise about the engine"),
which stays true after the jump — what changes is the crate's clock, not what a
facade is for.

## Consequences

**What moves:** the facade's version, once, from 0.1.2 to the product number.

**What does not:** the product tier (wheel, npm, `lat`, browser) and the engine
tier both stay where they are. Publishing diff, merge and excel adds crates at
0.9.0; it does not bump anything.

**The engine tier grows to eleven published crates**, and the gates that ride on
`PUBLISH_SET` — semver, public-API snapshot, packaging contents — grow with it.
That is the standing cost decision 4 accepts. As of 2026-08-05 it is **ten**:
phase 2 published diff and merge, and excel is the one still to come.

**The version gate changes shape at the jump.** `test_version_faithful` currently
asserts the facade starts with `0.1.` and explains why. At parity that assertion
inverts: the facade must equal PRODUCT, and the exemption becomes a parity
assertion. It is a one-way door and should be its own reviewed change, not a
side effect of the last capability landing.

**`bump-version.sh product` gains a surface.** Once the facade is on the product
line it must be stamped with the others, or the first release after the jump
leaves it behind.

**The register loses a verdict.** With no by-design exclusions left,
`facade_verdict: by-design` has no remaining user. Whether
`repo:tools/gen_modality.py` keeps the tri-state for a future exclusion or drops
it is a phase-5 question, not one to answer now — but it should not be left
sitting there unexercised without a decision either way.

**What this rules out:** `cargo install laterite-cli` as a supported route, and
any reading of "0.2" as a facade milestone. If the CLI is ever wanted on
crates.io, this page is what has to be revisited — along with preparing two
crates and finding an honest answer to the bin-only gates problem. Note that
decision 4 removes one of the reasons given against it: `laterite-ags4-excel` was
named as an unprepared blocker, and after phase 3 it is prepared. What remains
against the CLI is `laterite-cliutil`, the bin-only gates problem, and the
dependency weight — the argument is narrower now, and honesty requires saying so.

## Related

[[dec-beta-claim]] is the other side of this page: until these phases land, the
facade is the one surface carved out of the beta claim — on the completeness axis
named here, not a quality one. It joins when this reaches parity.

[[dec-rust-api-crates-io]] · [[modality-register]] · [[api-surface-1.0]] ·
[[crate-map]] · [[dec-ags4-merge-semantics]] · [[surface-census]] ·
[[dec-beta-claim]] · `repo:tools/gen_modality.py` ·
`repo:tests/test_version_faithful.py` · laterite#241 · laterite#162
