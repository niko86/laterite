---
type: decision
title: "laterite is in beta — one sentence, five surfaces, and the Rust crate carved out"
status: accepted
tags: [design, decision, release, versioning, docs, crates-io]
decided: 2026-08-13
supersedes: []
from_gap: []
related: [dec-facade-parity, dec-rust-api-crates-io, surface-census, docs-site, crate-map, modality-register]
sources: []
repo_refs:
  tier: "repo:RELEASING.md"
  classifier: "repo:packages/laterite/pyproject.toml"
  facade: "repo:rust-packages/laterite/README.md"
  support_page: "repo:web/docs-site/docs/reference/support.md"
  surfaces_page: "repo:web/docs-site/docs/surfaces/index.md"
  template: "repo:.github/ISSUE_TEMPLATE/bug.md"
---

# laterite is in beta — one sentence, five surfaces, and the Rust crate carved out

## Context

The beta announcement needed to know what it was claiming and where the claim
physically lives. Two problems had to be solved before it could:

**The repo did not agree with itself about what the surfaces are.** Four
enumerations existed. `README.md` and `web/docs-site/docs/surfaces/index.md` said
**five**, counting the web app as one. `web/docs-site/docs/feedback.md` said a
different **six**, naming the wasm package and an engine crate but not the app.
`.github/ISSUE_TEMPLATE/bug.md` said **seven** and carried two factual naming errors
(`@laterite/*` for the Node package, which is unscoped `laterite`; "the `laterite`
extension" for `laterite_ags4`). `RELEASING.md`'s `product` version tier said a
fifth thing — and was the only one that was right.

**Only one status field exists anywhere.** `Development Status` on PyPI. npm,
crates.io and GitHub Releases have no equivalent, so on every other surface the
claim can only be prose.

## Options considered

1. **Tiered claim** — headline surfaces / also available / not yet. Accurate about
   maturity differences, but invites "so which parts do I trust?" on day one.
2. **Uniform claim over the whole project**, with the Rust crate named as the one
   exception.
3. **Per-artifact claims**, each surface stating its own status against its own
   version number.

## Decision

**Option 2.** One sentence — *laterite is in beta* — covering everything shipped,
the engine crates included, because it all compiles from the same source. No tier
table, no per-surface grading.

**Five surfaces carry it**: the Python wheel, npm `laterite`, npm
`@laterite/ags4-wasm`, the `laterite_ags4` DuckDB extension, and the `lat` binary.
This is `RELEASING.md`'s existing `product` tier, not a new list.

**The Rust crate `laterite` is the one exception** — excluded for *completeness*, not
quality: it is not yet at parity with the other five ([[dec-facade-parity]] phase 4
of 9). Its README says so in one line. It joins when it is finished.

**The web app is not a surface.** It is a worked example of the browser package,
built on it and nothing else — a call already recorded in `mkdocs.yml` when thirteen
near-identical Browser cookbook tabs were deleted for the same reason. `README.md`
and the surfaces page had not caught up.

**Beta attaches to the project, never to a version.** Three clocks run (product
0.10.1, engine 0.9.0, facade 0.1.2); attaching beta to a number would mean three
betas. Never written `0.11.0-beta`, and no pre-release tags — the `rc` tag cut before
the beta tag is a rehearsal of `release.yml`'s machinery, user-invisible, and must
not be read as the label.

**Placement is README-first, never `description` strings.** The PyPI classifier flips
`3 - Alpha` → `4 - Beta`; npm, crates.io and Releases get a README or release-notes
line; `laterite_ags4` gets `extended_description`; the web app gets a footer line;
the engine crates and the `@laterite/native-*` addons stay silent. The definition
lives once, on the docs' `reference/support.md`, and everything else links to it.

## Why

**Uniform, because tiering by trust would be false.** The cross-surface compliance
harness asserts byte-identical findings across every read surface on each PR, so
"the same verdict everywhere" is tested, not claimed. There is no quality axis to
tier on, and inventing one would advertise a distinction the tests deny.

**The Rust crate is carved out on the honest axis.** Saying "not yet at parity"
describes an unfinished roadmap; saying "not beta quality" would describe the engine,
which is the same engine the other five run. The first is true and is a plan; the
second is false and is an apology.

**Not `description` strings**, because a description is what a stranger reads while
deciding whether to click. "Beta" there filters out exactly the cautious users the
announcement is trying to reach, at the moment of the click, with no room for the
explanation that makes it fair. Explicitly **not** `lat --version` either — that
output is machine-parsed by [[surface-census]] and the xcheck gate, and a decorative
suffix breaks a merge check for a cosmetic win.

**Defined once**, because two independent definitions of "beta" is precisely how four
disagreeing surface lists happened, and prose drifts faster than lists do.

**One 1.0 across the product tier, justified as "thin *and* gated".** Node's maturity
rides on Python's usage because what usage exercises is the shared engine, and the
language layer is a thin wrapper. "Thin" alone is not a sufficient reason and this
repo's own history disproves it as a sole argument: `lat merge` shipped in the binary,
reached neither the uvx nor npx launcher, and every cross-surface gate stayed green —
which is why [[surface-census]] exists. The reason written down therefore names the
three gates that check the wrapper where usage cannot: the census (does the verb
exist), xcheck (do output values match), and the compliance harness (do findings
match). That gap was about **completeness**, not thickness — a missing door does not
make a wrapper thick — and the census catching it is the system working.

## Consequences

- The product tier moves to 1.0 together; the Rust crate keeps its own clock and is
  not carried along by it.
- **Platform parity becomes a rule**: a platform shipped by one artifact is shipped by
  all. `aarch64-unknown-linux-gnu` therefore belongs in `release.yml`'s **binaries**
  matrix, not only the wheel matrix and `napi.targets`.
- `reference/support.md` must exist **before** the beta tag — the classifier flip and
  every README line land in the release the announcement points at, so nothing claims
  beta before there is a page explaining it.
- The web app **stamps** the wasm version it was built from rather than pinning a
  published one. It builds wasm from workspace source, making it the only continuous
  dogfood of HEAD wasm; pinning would give it a version at the cost of making the
  highest-traffic front door lag.
- Adding a surface later means adding it to the `product` tier and this page's list —
  and nowhere else, because everywhere else links rather than restates.

## Related

[[dec-facade-parity]] · [[dec-rust-api-crates-io]] · [[surface-census]] ·
[[docs-site]] · repo:RELEASING.md · repo:web/docs-site/docs/reference/support.md ·
repo:packages/laterite/pyproject.toml · repo:rust-packages/laterite/README.md
