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
  >
  > **Answered 2026-08-30: BUILD.** See "The reopened question, answered" below.
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

## The reopened question, answered (2026-08-30)

Per-crate versioning landed and reopened this page, as #781 predicted. The
question narrowed to **build the nightly cut on `release_status.py`, or adopt
release-plz for it** — and the answer is **build**, tracked in #806.

Two reasons, the first structural rather than a preference.

**The publish stays approval-gated, so `publish_crates.py` stays with it.** That
spends release-plz's strongest advantage before the comparison starts. This page
already recorded why: crates.io shipped Trusted Publishing after the spike, so
the half-step that made release-plz attractive — someone to hold the credential —
stopped being needed at all. What remained was the cut alone.

**And it would supply the cut from weaker evidence.** release-plz derives the
version part from conventional commit subjects; #216's spike measured that a
substantial share of recent commits carry no conventional type, and that 0.x
semantics tighten under it. This repo's part comes from the `cargo-public-api`
snapshot — the public surface itself, and the only source that can see an
addition at all, since `cargo semver-checks` has no `function_added` lint.
Adopting would mean a second release automation replacing better evidence with a
proxy, and inheriting the spike's unverified list, chiefly whether
`--manifest-path` carries a workspace below the git root.

**What building costs, stated so it is not discovered later:** the cut logic
becomes ours to own, and it is subtler than a snapshot diff. Its baseline must be
the last **published** version rather than the last stamp — measuring from a
stamp that never published reports a bump as owed when the standing number
already covers it, which is a wrong verdict a human shrugs at and a machine acts
on. #806 carries that as a prerequisite rather than a follow-up.

**What the decision does not change.** The dispatch becomes automatic; the
`crates` environment approval does not. That gate is the reason this page's
earlier consequence could be struck through, and nothing here reopens it.

## Consequences

- `repo:tools/publish_crates.py` stays, and stays the only publish path for the
  engine tier. ~~`engine-version.toml` and its guard test stay with it.~~
  **Overtaken 2026-08-29:** #781's per-crate versioning (landed in #795)
  retired the lockstep pin file; bumps go through
  `repo:tools/release/bump_crate.py` now.
- ~~The highest-consequence manual step in the release process remains manual, on
  the one registry that cannot be corrected after the fact.~~ **Paid, in #463.**
  The publish runs from GitHub behind the `crates` environment's reviewer, so the
  append-only registry is now the one with an approval in front of it rather than
  the one without. What stays manual is the *dispatch* and the approval, which is
  the point of them.
- **The cut is ours to build (#806), and the nightly gains a second duty.**
  The same report that says a bump is owed also says a publish is owed, so one
  derivation drives both — opening a cut PR, and dispatching the approval-gated
  publish for anything stamped but absent from the registry.
- **The report reads the registry now (#801).** `repo:tools/release/release_status.py`
  looks each crate's stamp up in the crates.io sparse index, so *stamped here but
  never published* — the state the manual-dispatch shape leaves reachable, and
  which `laterite-ags4-emit` 0.12.0 sat in unnoticed — is reported rather than
  invisible. That is evidence for the build-vs-adopt question above, not an answer
  to it: it closes the detection gap, and says nothing about who should run the cut.
- **Built, 2026-08-30.** The cut ships as `repo:tools/release/engine_cut.py`
  over `repo:tools/release/release_status.py`'s derivation, run by the
  nightly's `engine-cut` job (which replaced `release-owed`, keeping its
  reporting duty). Three things the build settled beyond the design table:
  - **The baseline is the published tarball's own commit**
    (`.cargo_vcs_info.json` from static.crates.io), not the stamp commit — the
    prerequisite's first cut used the stamp and immediately over-reported emit
    (+7 −4 already on the registry). When the tarball's commit cannot be
    placed in history, an API/code delta is reported but never acted on.
  - **A fourth signal existed that no crate-local reading has**: a
    `[workspace.dependencies]` floor moved past the pins a published sibling
    carries (#809 — three crates stranded with every local gate green). The
    sparse-index rows carry each version's requirement ranges, so the cut
    derives it, and ci.yml's repo-gates run the same check per PR,
    failing only debt the PR introduces.
  - **Publish dispatch is automatic nightly** for stamped-but-absent crates,
    cancelling any stale pending run first — a pending approval publishes
    `main` as of *approval* time, so nightly re-dispatch bounds that staleness
    to one night. The `crates` environment approval is unchanged.
- **Rollout state: answer-only, as the design row requires.** The PR-opening
  path is built and armed behind the repo variable `ENGINE_CUT_MODE=pr`;
  promotion is that one variable, plus `ENGINE_CUT_TOKEN` (a fine-grained PAT,
  contents + pull-requests write) — with the default `github.token` a
  bot-created PR triggers **no** workflows, its required checks never report,
  and it cannot merge. That answers the spike's "can a bot PR satisfy every
  required check" unverified item: not with the default token, by design.
- **Pre-upload build verification is wired** (`check_package_contents.py
  --verify-buildable` in publish-crates.yml, before the first upload): a
  wave-3 compile failure no longer strands waves 1–2 on the append-only
  registry. Its stated ceiling stands — a crate whose deps' new floors are not
  yet on the registry is unverifiable before its wave, and the skip prints.

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
