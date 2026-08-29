---
type: decision
title: "How the engine tier publishes: not release-plz, yet — and the one trigger that changes it"
status: accepted
tags: [design, decision, release, crates-io, versioning, ci]
decided: 2026-08-19
supersedes: []
from_gap: []
related: [dec-rust-api-crates-io, dec-monorepo-structure, crate-map, laterite-cli, reliquary]
sources:
  - "https://release-plz.dev/docs/config"
  - "https://github.com/release-plz/release-plz/tree/release-plz-v0.3.160"
---

# How the engine tier publishes: not release-plz, yet

## Context

Three registries carry this project, and they are not equally guarded. PyPI and
npm publish from `release.yml` behind trusted publishers and an environment
approval. crates.io — the **append-only** one, where a version can never be taken
back — is the only tier published from a checkout on the maintainer's machine, by
`repo:tools/publish_crates.py`. `cargo publish` appears nowhere in
`.github/workflows/`.

Beta brings outside attention to releases, and whatever the path is at that
moment becomes "the process". `release-plz` is the obvious candidate to automate
it, so the question was asked before the answer calcified: adopt it, or record why
not.

Spiked read-only against release-plz `release-plz-v0.3.160` and this tree — see
[#216](https://github.com/niko86/laterite/issues/216). **The findings live in that
issue's spike comment**, which is the primary source; this page records the
decision and the conditions that would reopen it.

## Options considered

1. **Full adoption** — `release-plz release-pr` derives the version from commit
   subjects, `release-plz release` publishes.
2. **The publisher half only** — keep `bump-version.sh engine` deciding the
   number and `changelog.json` owning the prose; use `release-plz release` purely
   to publish what the manifests already say.
3. **Neither, for now** — keep `publish_crates.py`.

## Decision

**Option 3 — not yet.** Option 2 is recorded as the reversible half-step, and is
what the reopening trigger below should be measured against.

## Why

The mechanical fit turned out **substantially better** than the issue that asked
the question assumed, and that is the part most at risk of being lost:

- **Subset scoping needs zero configuration.** The product crates carry
  `publish = false`, and release-plz's package set is publishable packages only.
- **Lockstep is native, not something to fight.** `version.workspace = true` is a
  first-class case; adopting it would not force per-crate semver.
- **`changelog.json` can stay sole SSOT** — `changelog_update = false` writes no
  file.

So the objection is not fit. It is that full adoption changes what a version
*means* here: the number stops being a decision and becomes a derivation from
commit subjects, on a history where a substantial share of commits carry no
conventional type and several house-invented ones are in use. That is recoverable
with custom increment regexes — a new configuration surface which is silent when
wrong. It also tightens 0.x semantics in a way the current scheme does not
(`feat:` would take the patch, not the minor), and it puts **two** release
automations on one repo where today there is one script with a required first
argument.

Against that, the cost of waiting is one hand-run publish per engine release.

## What would reopen it

Two conditions were considered; only one needs watching.

- **The engine stops being lockstep.** Structurally impossible today — every
  engine crate is `version.workspace = true`. It cannot arrive by accident, only
  as a deliberate decision, and that decision reopens this question by itself.
  Nothing needs to watch for it.

  > [!note] This trigger fired: per-crate landed 2026-08-30 (#781)
  > Every published crate now carries its own `version`
  > (`tools/release/bump_crate.py` moves one; the lockstep files are retired),
  > so this page's question is REOPENED as #781 predicted — and #781 already
  > frames it: the owner's model wants a **nightly per-crate cut decision**
  > (derive each crate's part from its API snapshot, cut when warranted,
  > fail fast on a compile error and publish nothing on a partial build),
  > which is release-plz's native shape. The decision to make is now
  > **build vs adopt**: `release_status.py` already derives per-crate verdicts
  > nightly (the build half's report exists), and Trusted Publishing already
  > owns the upload, so what release-plz would add is the cut itself. Decide it
  > on #781's step 4, and record the answer here.
- **The crates.io publish needs to leave the maintainer's laptop.** The stronger
  trigger, and the one that was being tracked — see
  [#463](https://github.com/niko86/laterite/issues/463), which carries the
  evidence and records the publisher-half-step as the alternative it was measured
  against. That framing is deliberate: adopting release-plz stays **one decision**
  away rather than one investigation away.

  > [!note] The trigger fired, and release-plz was not what answered it
  > #463 moved the publish into `repo:.github/workflows/publish-crates.yml`,
  > authenticated by crates.io Trusted Publishing — an OIDC exchange with no
  > stored token, the same shape `pypi-publish` and `npm-publish` already use.
  > What changed under this page is the registry, not the tool: crates.io shipped
  > trusted publishing after the spike, so the half-step that made release-plz
  > attractive here (someone to hold the credential) stopped being needed at all.
  >
  > So the condition is **spent**, not pending. Adopting release-plz is now a
  > plain decision with nothing waiting on it, which is a weaker case than the
  > one this page deferred — the manual step it would have refunded is gone.

**"When there are N crates" is NOT the trigger**, and recording that was half the
point of the spike. `publish_crates.py` derives its waves from the manifests on
every run — deliberately, so the dependency order is never stated twice — so
another engine crate costs nothing release-plz would refund.

## Consequences

- `repo:tools/publish_crates.py` stays, and stays the only publish path for the
  engine tier. `repo:tools/release/engine-version.toml` and its guard test stay
  with it.
- ~~The highest-consequence manual step in the release process remains manual, on
  the one registry that cannot be corrected after the fact.~~ **Paid, in #463.**
  The publish runs from GitHub behind the `crates` environment's reviewer, so the
  append-only registry is now the one with an approval in front of it rather than
  the one without. What stays manual is the *dispatch* and the approval, which is
  the point of them.
- **If this is revisited, start from #216's spike comment, not from that issue's
  body** — the body was written before any of it was checked against the source,
  and several of its premises did not survive contact.

## Unverified

Stated rather than assumed, because a future adoption should start by proving it:

- Whether `--manifest-path` works end to end for a workspace **below** the git
  root, which is this repo's shape (`rust-packages/`). The flags exist and the
  types support it; no documentation covers the case and the spike ran nothing.
  **The single highest-value thing to prove before any adoption.**
- Whether a bot-authored release PR can satisfy every required check.
- Whether any post-update hook could keep a file like `engine-version.toml` in
  step — absent from the config reference and the JSON schema, which is weaker
  than a doc saying so.
- Behaviour on a **squash-merged** release PR, which is how this repo merges.
- What crates.io trusted publishing requires, and whether it composes with
  environment approvals the way the PyPI and npm ones do.

## One hazard to carry forward

release-plz's default tag template flips to `v{{ version }}` for a
**single**-package workspace. This repo's PyPI release train already owns `v*`
(`repo:RELEASING.md`), so any adoption must set `git_tag_name` explicitly rather
than rely on the default — a tag collision here would fire the wrong publish
workflow.
