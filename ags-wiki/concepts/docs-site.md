---
type: concept
title: "laterite docs site (MkDocs, example-led)"
status: drafted
tags: [concept, architecture, ci, docs, web]
ags_editions: []
repo_refs:
  config: "repo:web/docs-site/mkdocs.yml"
  examples: "repo:web/docs-site/examples"
  fixture: "repo:examples/sample_site.ags"
  node_gate: "repo:rust-packages/laterite-node/test/docs-examples.test.ts"
  link_gate: "repo:.github/workflows/ci.yml (docs job)"
  deploy: "repo:.github/workflows/deploy-validator.yml"
  catalogue: "repo:web/docs-site/scripts/gen_groups.py"
  glossary: "repo:web/docs-site/scripts/gen_types.py"
  catalogue_data: "repo:web/docs-site/scripts/catalogue_data.py"
  catalogue_js: "repo:web/docs-site/docs/javascripts/catalogue.js"
  theme_overrides: "repo:web/docs-site/overrides"
  theme_css: "repo:web/docs-site/docs/stylesheets/laterite.css"
  token_sync: "repo:web/scripts/sync-docs-tokens.mjs"
  band_gate: "repo:tests/test_docs_band_containment.py"
  duckdb_gate: "repo:tests/test_docs_duckdb_examples.py"
  header_gate: "repo:tests/test_docs_example_headers.py"
  type_gate: "repo:tools/check_doc_types.py"
  em_dash_gate: "repo:tools/check_docs_em_dash.py"
  released_legs: "repo:.github/workflows/nightly.yml"
  released_crates: "repo:tools/check_released_crate_readmes.py"
  released_crates_gate: "repo:tests/test_released_crate_readmes.py"
related: [validator-site, playwright-e2e, dec-landing-build-shared-tokens, dec-example-header-environment]
sources: []
---

# laterite docs site (MkDocs, example-led)

## Definition

The laterite suite's documentation website (#201): **MkDocs + Material**, sources
under `repo:web/docs-site/`, published at **`/laterite/docs/`** on the *same*
GitHub Pages artifact as the [[validator-site]] app — `mkdocs.yml` sets
`site_dir: ../docs-dist`, so the static site lands in its **own** directory
beside the app's rather than inside it, and the deploy step carries it to
`/laterite/`. (This line said `../dist/docs` until #588, which is a directory
`mkdocs.yml` has not named for some time; the config is the authority.)

Key facts:

- **Example-led / single-sourced — but not universally.** *Most* code on a page is
  `--8<--`-included (pymdownx.snippets, `base_path: [examples]`) from
  `repo:web/docs-site/examples/{python,node,cli,duckdb,wasm}/` — and each tree
  has a **runtime gate** so page and test are the same bytes (#373 built the
  three non-Python trees, #228 added the wasm one; a changed return shape / dtype / printed format / method
  name turns a doc snippet **red in CI** — the example-first analogue of the
  OBSERVATIONS / `.pyi` drift gates):
  - *python* — `repo:tests/test_docs_examples.py` runs each `ex*.py` as a
    subprocess against the installed wheel + the committed
    `repo:examples/sample_site.ags` fixture (`cwd` = repo root, so the literal
    `"examples/sample_site.ags"` path resolves); in-file `assert`s pin outputs.
  - *node* — `repo:rust-packages/laterite-node/test/docs-examples.test.ts`
    (vitest, existing `node` job) runs each `ex*.mjs` the same way; the
    examples' literal `import … from "laterite"` resolves via a gitignored
    `node_modules/laterite` symlink beside them (the pack-smoke trick — ESM
    ignores `NODE_PATH`, and self-reference only works inside the package dir).
  - *cli* — `tests/test_docs_cli_examples.py` (dev satellite) runs each `*.sh` with the
    release `lat` on `PATH` (skipif not built, the `test_laterite.py`
    idiom), from a **temp dir** (scripts mint dirty files / certs), asserting
    the `# expect-exit: N` code and **byte-equality of stdout vs a committed
    sibling `.out`** — pages include the `[start:cmd]` section + the `.out`
    verbatim, so even the *output* blocks can't lie (the old hand-typed CLI
    table had already drifted from the binary's real box-drawing output). It
    drifted the same way a SECOND time, in the blocks this gate never covered
    because no example backed them: `learn/install.md` — the first page a new
    user reads — printed its findings table as plain ASCII pipes, which is what
    the WHEEL's Python `lat` renders, on a page documenting the Rust binary. The
    row's content was right, so somebody ran it once; nothing re-ran it. Wiring
    a block to an example is therefore not bookkeeping, and an *unwired* output
    block is the actual risk surface — see the orphan-block note below.
  - *duckdb* — `tests/test_docs_duckdb_examples.py`, env-gated, and the one gate
    here whose subject is **not** built from this tree: `LATERITE_DUCKDB_COMMUNITY=1`
    installs the PUBLISHED community extension, `LATERITE_DUCKDB_EXT=<path>` loads a
    local build, and which one ran is printed rather than assumed. Per-PR the `.sql`
    files are include-checked only (`--strict` + `check_paths`); **nightly's
    `docs-vs-released-duckdb`** runs them against the published extension, and a
    local-build twin of this file runs on-demand in the dev satellite's
    `compliance-report.yml`. Fail-soft only where the failure is not about the docs:
    ABI drift in local-build mode is a visible skip, and so is "the community
    repository has no build for this DuckDB yet" — a broken snippet is red either
    way. `_`-prefixed files (`_install.sql`) are
    include-only boilerplate. That cadence is a claim about another repo's
    workflow, and [[stated-cadences-faithful]] is what holds it to it.
    <!-- cadence: ci --><!-- cadence: compliance-report -->
  Browser tabs are **prose** (the web app has no user-facing code API).
- **The js corpus is type-checked, not just executed** (#565).
  `repo:tools/check_doc_types.py` runs `tsc` (resolution-only — no
  `noImplicitAny`/`strictNullChecks`, the #565 decision) over the assembled
  inline-bearing page programs *and* the node/wasm example files, against each
  package's **shipped** types (`dist/index.d.ts` via the `exports` map; the
  wasm-pack `.d.ts`). Execution's bar is "does not raise", which cannot see a
  page running the wrong branch — #518's `report.ok` was always `undefined`, so
  the example took the else branch and passed. The gate shares
  `gen_doc_outputs.page_program` (assembly, not execution), starts every leg
  with a #518-shaped positive control that must go red, reports an unbuilt leg
  by name, and holds its allowlist entries stale-checked. Node leg in the
  `node` job, wasm leg in `ts-lint`.
- **A Python example is `uv run`-able on its own** — all 18, plus the marimo tour.
  Each carries a PEP 723 `# /// script` header pinning
  `requires-python` and an exact `laterite==<product>`, and the 15 that read a
  file also carry a fixture arm that
  fetches `repo:examples/sample_site.ags` from the raw GitHub URL when the
  repo-relative path is absent — so a reader with `uv` and no checkout gets a
  working run, and the code on the page stays the code you would type in a
  checkout rather than an absolute path. Two details the rollout settled that the
  prototype could not show, because `ex01` is polars-only and reads the fixture:
  `ex09a`, `ex09b` and `ex15` build from frames / a typed graph / inline text and
  get **no** fixture arm (there is nothing to fetch for), and `ex05` + `ex11`
  import pandas — which is NOT in the base install — so they pin
  `laterite[compat]==<product>`. A uniform bare pin makes both die under
  `uv run` on `ModuleNotFoundError: pandas`, the exact failure the header exists
  to prevent. **`ex06` + `ex21` are the same case one library along, and it took
  a gate to find them** (#514): both finish on `rel.pl()`, and the relation
  `.sql()` returns is DuckDB's, so the materialiser is DuckDB's and imports
  pyarrow. Nothing on the page imports it and polars is in the base install, so
  the bare pin looked right; they now pin `laterite[pyarrow]==<product>`. The
  extras a header needs are not derivable from the imports it shows —
  [[dec-example-header-environment]]. Header and fixture arm live ABOVE a
  `--8<-- [start:code]` marker and the page includes `…py:code`, so the rendered
  snippet is byte-identical to before the header existed; the machinery is in the
  file, not on the page. That is the CLI tree's `[start:cmd]` trick
  (`repo:web/docs-site/examples/cli/validate_clean.sh`) applied to Python.
  The arm is cold in CI (cwd = repo root, the fixture is there), so no gate
  acquires a network dependency. The pin is a CLAIM — "green against this wheel" —
  and `repo:tests/test_version_faithful.py` holds both it and the interpreter
  floor to the shipped values by DISCOVERY over **both** example trees, the
  docs corpus and the root `examples/` where the tour lives. It asserts the whole
  specifier (`==<product>`, extras group allowed), not the version inside a `==`:
  the first cut matched `laterite==(version)`, which silently passes anything that
  is not an exact pin, so the tour's own `>=0.5.0` — five untested minors, the
  defect that motivated the gate — produced no match and therefore no finding.
  `bump-version.sh` restamps with a substitution anchored on the pin *including*
  its optional `[extras]`; the sibling loop that stamps `.out` files rewrites
  every occurrence of the old version in a file, which is safe in generated
  output and would silently rewrite an `assert` in a source.
  Ordering forces `import laterite` to follow the fixture arm, so
  `web/docs-site/examples/**` takes the `E402` per-file-ignore the root
  `examples/**` already has — the alternative is publishing a snippet that omits
  its own import to satisfy a linter.
- **The other 22 fences — hand-written, and gated separately.** The list above is
  the *included* half: 38 of the site's 60 Python fences. The rest are LITERAL —
  fragments a page writes inline (`for group in delta["groups"]:`), which read
  correctly as fragments and would be worse as standalone examples. Every gate
  above keys off the `--8<--`, so all of them were invisible to it, as were
  `repo:README.md`, the PyPI landing page and `repo:COMPAT.md`. What that cost was
  measured on 2026-08-08: `COMPAT.md` documented
  `check_file(..., dictionary=...)` in both blocks of its error-handling section,
  and no such parameter has ever existed in either library
  (`standard_AGS4_dictionary` is the name), so the two snippets illustrating the
  divergence raised `TypeError` on both sides and demonstrated nothing. The prose
  was right; only the code was uncopyable.
  `repo:tests/test_docs_snippets.py` closes it by treating a page as ONE program
  in document order — includes replayed so a fragment inherits the `ags` / `fixed`
  its page established — EXECUTING what can run (85% of literal statements) and
  resolving names + call keywords against the wheel for the rest. It runs from a
  temp dir with the fixture copied under `examples/`, like the CLI gate and for
  the same reason: `surfaces/python.md` and `cookbook/excel.md` WRITE files.
  A `<!-- doc-snippet: skip — reason -->` marker (the shape of `doc-output: skip`,
  reason likewise mandatory) exempts a fence from execution but never from
  resolution — the `dictionary=` typo lived in a block that would carry one.
- **Node's literal fences, and the two-gate split.** The same hole existed one
  surface along: 11 hand-typed fences plus the npm landing page's 4, none
  executed. `repo:rust-packages/laterite-node/test/docs-snippets.test.ts` closes
  it — vitest, not pytest, because executing them needs `dist/` and the
  `node_modules/laterite` symlink that only the `node` job builds; a pytest
  version would skip in CI, which `test_docs_node_api.py` explicitly refused to
  accept. It welds a page into one ESM program (imports unioned per module,
  re-declarations turned into assignment — ESM forbids the rebind Python allows)
  and runs it from a temp cwd with the fixture copied in.
  **The two Node gates catch different things and neither subsumes the other:
  the executor catches CALLS, `repo:tests/test_docs_node_api.py` catches READS.**
  A bare `report.someTypo` evaluates to `undefined` and logs rather than throwing,
  so no amount of executing finds it; `buildAgs4({…})` with the wrong shape throws,
  and no amount of name-resolution finds that. Falsification proved the split and
  exposed a hole in passing — the static gate scanned `docs/node/` only, leaving
  the npm README covered by neither — so its scope now includes that page.
- **Orphan output blocks — down to one, and it is not output.** `--check-pages`
  counts `text` fences that no example backs and prints the number rather than
  absorbing it; that count was **8** and is now **1**. The seven closed were
  hand-written stdout on `learn/install.md`, `concepts/severity-tiers.md` and
  `concepts/one-engine-many-doors.md`, and one of them had drifted into the wrong
  renderer entirely (above). The survivor is `chaining/index.md`'s ASCII flow
  diagram — a `text` fence that is a *drawing*, not the output of anything, so it
  is correctly and permanently unwired. Worth stating so nobody "fixes" it: the
  right target is **zero gateable orphans**, not zero orphans.
- **The INPUT half, and why it came second.** Everything above gates the block
  showing what an example *prints*. Nothing gated the fence showing what a reader
  *runs*, so a snippet could use a name the page never binds and no gate could
  say — which is exactly what two cookbook pages did, and for long enough that
  nobody could date it. `--check-pages` now classifies every code fence as
  **include / inline / skipped / prose** on the same opt-out-with-a-reason
  contract, and prints the counts. **`inline` is the one to watch**: fences a
  reader can copy that no gate has yet executed. See [[dec-doc-code-fences]] for
  why a page is one program per surface rather than one per fence, and why `bash`
  is excluded by name rather than by silence.
- **One nightly leg asks the READER's question.** Every gate above runs against the
  working-tree cdylib, so all of them answer "is the site consistent with HEAD" —
  and the reader is not on HEAD, they are on `pip install laterite`. The
  `docs-vs-released-wheel` job in `repo:.github/workflows/nightly.yml` installs the
  **published** wheel (no pin — whatever PyPI resolves today) and runs the docs
  examples plus `repo:tests/test_docs_snippets.py` against it, printing the
  released version beside this tree's so the run states what it measured.
  Its two steps are calibrated differently and that is the whole design: an
  example that **fails to run** fails the job, because a reader following the live
  site would get a traceback; a committed `.out` that no longer byte-matches is
  reported and **not** fatal, because output drift is the ordinary consequence of
  the tree being ahead of the release. Which of the two a run is in is decided
  ONCE, by the `under-test` step asking git whether this checkout is the released
  tag, and **every** fatal step reads that one answer — the fix in #493, where the
  CLI write-mode step did not, so an unreleased *Python* change was amnestied and
  an unreleased *CLI* change was fatal on the same run. The job used to stay out
  of `notify`'s `needs` as a second guard on the same window; that guard also hid
  real failures, so the legs report and the determination does the separating
  (`repo:tests/test_nightly_wiring.py` holds both halves). GitHub-hosted and
  toolchain-free: a published wheel needs no Rust and no maturin.
- **A second nightly leg asks it about the ENVIRONMENT, not the wheel.** The leg
  above installs the released wheel *with every extra* and supplies the
  environment itself; so does the per-PR gate. Neither has ever run the PEP 723
  header the examples publish, so a header could be missing a dependency and
  nothing could fail — and two were (#514, above). `docs-example-headers`
  (`repo:tests/test_docs_example_headers.py`, opt-in behind
  `LATERITE_DOCS_HEADER_ENV`) runs each example with `uv run --exact --script`
  — the reader's command plus `--exact`, which is deliberately stricter than
  what the docstrings tell a reader to type, for the reason below.
  **It is a separate job because the amnesty
  is wrong for it**: a missing extra is not fixed by any release, so excusing it
  through the tree-ahead window — nearly every night — would leave the leg
  decorative. The classification happens in the test instead, by re-running a
  failure with the pin widened to `laterite[all]`: decided by the extras is
  fatal, anything else is a loud skip. `--exact` is load-bearing, not tidiness —
  uv caches a script environment by PATH and does not shrink it, so without the
  flag the module's own falsification passed against a deliberately broken
  header. [[dec-example-header-environment]] carries the rest.
- **…and its Node twin, `docs-vs-released-npm`.** Same question, same calibration:
  `npm install laterite` unpinned, then the optional peer **exactly as the docs and
  the runtime error tell a reader to install it**, then the examples. Building it
  is what surfaced that the published `peerDependencies` range matched no
  published version at all (see `repo:ags-wiki/tools/laterite-node.md`), so it was
  earning its keep before it had run once; the fix has since shipped. A red run
  here means the published package is behind what the site documents, and the
  remedy is a release — stated as the *rule* rather than as a running commentary
  on the current state, which is how this bullet came to carry "expect it red
  until that fix ships" for weeks after it had. Like the wheel leg it computes
  `tree_ahead` and every
  fatal step reads it, which is what lets it report to `notify` without filing an
  issue about a state already known.
- **…and the browser twin, `docs-vs-released-wasm` (#283).** `npm install
  @laterite/ags4-wasm` unpinned, then the five `examples/wasm/ex*.mjs` against it.
  The crate says `publish = false` — for *crates.io*; it **is** the npm package,
  with `wasm-pack` writing the published manifest from its version line. Same
  calibration, one difference that matters: **equal version numbers do not mean
  equal artifacts here**, because the browser package releases on its own
  `wasm-v*` tag while the crate's version line only moves at the next umbrella
  bump — so the tree can carry weeks of unreleased wasm at a version string
  identical to npm's, and the wheel leg's "same version, so any failure is a real
  defect" would be a lie. The step prints the npm **publish date** beside this
  tree's HEAD date instead. The swap itself is one seam: `WASM_PKG_DIR` tells
  `gen_doc_outputs.py:_wasm_pkg()` which package to symlink into the examples'
  `node_modules`, exactly as `LAT_BIN` does for the CLI examples — and, like it,
  the resolved path is printed rather than assumed. This bullet carried the npm
  one's twin claim — expect it red until the next `wasm-v*` release, because
  `ex03_read.mjs` pulls rows through `rows_json()` and the published artifact
  predated it — and that release has shipped. The dates the step prints are what
  answer "is it behind?" on the day it is asked; a page cannot.
- **…and the crates.io twin, `docs-vs-released-crates` (#283).** #278 wired every
  publishable crate's README example into `cargo test --workspace` via
  `#[cfg(doctest)] #[doc = include_str!("../README.md")]` — which compiles it
  *inside the workspace*, where each `laterite-*` dependency resolves through a
  `path =` entry to the source next door. The reader's path is `cargo add
  laterite-ags4-core` and then that same example, against the **released** crate
  and its **released** dependency graph, where a re-export or a feature gate that
  exists only in the tree is not there. So
  `repo:tools/check_released_crate_readmes.py` generates a scratch consumer per
  crate — crate `cargo add`ed from the registry, tree README dropped in beside it
  with the same three-line wiring — and runs `cargo test --doc`. **No `path =`
  anywhere is the whole instrument**, and it is asserted by
  `repo:tests/test_released_crate_readmes.py` rather than trusted to the
  generator; that test also holds the two derived rules (which crates are
  subjects, and which `use` roots become dependencies — `use` roots only, or the
  facade's `ags4::read(…)` would send `cargo add ags4` at the registry). One
  class here, not two: a README doctest has no committed `.out`, so there is no
  drift half — it compiles and runs, or it does not. The released-vs-tree pair is
  **printed** per crate to separate a defect from ordinary tree-ahead drift —
  printed only as far as it goes, since a version it could not read establishes no
  direction at all, and printed rather than *consumed*, which is why a tree-ahead
  red here reaches the nightly tracker where the wheel and npm legs' would not
  (`repo:.github/workflows/nightly.yml`, the `notify` header, names that trade).
  A crate that is publishable but **not yet uploaded** (`publish_crates.py`'s `DEFERRED` state,
  earmarked next for `laterite-ags4-excel`) is reported as *unasked*, never
  failed: "not released yet" must not arrive looking like "the released README is
  broken".
- **Changelog page — generated, version-stamped (#372).** `reference/changelog.md`
  is built by `web/docs-site/scripts/gen_changelog.py` (a `gen-files` script)
  from the repo-root `CHANGELOG.md` plus the shipped version read from
  `packages/laterite/pyproject.toml` — **both derived at build**, so the page
  can't drift and needs no stamping. Repo-relative links are rewritten to the
  public mirror so `--strict` passes. Before this the release notes were invisible
  on the site (root-only `CHANGELOG.md`, no nav); now merging a release republishes
  the docs (deploy-on-master) showing the new version + notes — the docs are part
  of the release without a tag gate (a doc fix still ships immediately).
- **API reference — runtime mkdocstrings against the local wheel.** The
  `reference/api.md` + `reference/modules.md` pages are generated by
  **mkdocstrings**, which **introspects the installed `laterite` wheel** at build
  time. The build wheel is the **local working-tree build** (`uv sync --group
  docs` — so the docs track HEAD, not the last release), the owner's call over a
  published-PyPI wheel. The public-surface docstrings were swept (#215) from
  Sphinx inline roles (`:func:`x``) to mkdocstrings **cross-reference links**
  (`[`x`][laterite.x]`); `show_if_no_docstring` exposes the native PyO3 members
  (e.g. `Report.count`) so their anchors resolve.
- **Group catalogue + AGS data-type glossary (#201), generated not committed.**
  `reference/groups/` is one deep-linkable page per AGS4 group plus a paginated,
  filterable master table; `reference/types.md` is the AGS data-type glossary. Both
  are built at `mkdocs build` (`mkdocs-gen-files`: `gen_groups.py` / `gen_types.py`)
  from two sources: **`laterite.registry`** (the real KEY tuples / inheritance) and
  the **single-source union `ags_dictionary.json`** read directly for **edition
  provenance** — each group/heading carries an `eds` array, so the catalogue derives
  "added in 4.x" / "removed in 4.x" (the registry serves only the latest-edition
  union and drops `eds`/`by_ed`). The shared, side-effect-free
  `repo:web/docs-site/scripts/catalogue_data.py` holds the **family taxonomy**, the
  provenance helpers, and `TYPE_GLOSSARY` (sourced from the standard dictionary's
  `TYPE` group + the `laterite-ags4-types` canonical mapping; heading tables deep-link
  each type code to its anchor). Pages are **NOT committed** (no 174-file churn per
  dict edit), so the drift guard is a pytest gate not a snapshot:
  `tests/test_groups_catalogue_faithful.py` (dev satellite, python job) asserts every group
  has a family, every declared family is non-empty, every *used* type code is
  documented, and provenance derives correctly — the catalogue-side analogue of the
  dictionary / OBSERVATIONS faithfulness gates. The paginate / filter / family-card
  / "show all" UX is vanilla JS (`repo:web/docs-site/docs/javascripts/catalogue.js`),
  CDN-free.
- **The theme layer — Material mapped onto the shared tokens (#401).** The docs
  are the third surface of the one visual direction ([[dec-landing-build-shared-tokens]]),
  and the only one with **no bundler**: MkDocs copies `docs/` verbatim, so it can
  follow neither the shared layer's relative `@import` chain nor the Fontsource
  package specifiers inside it. So the tokens are **generated into the docs tree
  and committed** by `repo:web/scripts/sync-docs-tokens.mjs` — one bundle
  (`docs/stylesheets/tokens.css`) plus the `.woff2` files its faces name — with
  `--check` wired into `ci.yml`'s `ts-lint` job, the one that already runs
  `npm ci` in `web/`. Without that gate a retuned colour ships to the app and the
  apex and silently leaves the docs on the old palette, and nothing about the
  docs looks broken enough for anyone to notice. The one transform on the way is
  the selector: the shared layer's `.dark` rule becomes Material's
  `[data-md-color-scheme="slate"]`, which is what lets the docs run the same dark
  VALUES as the other two while **Material's own palette toggle keeps working**
  untouched. `theme.font: false` removes the `fonts.gstatic.com` preconnect, and
  the fork of `partials/source.html` drops `data-md-component="source"` so the
  bundle stops calling `api.github.com` for the repository facts (latest tag,
  stars, forks) — **twice per browser tab**, not per page view: Material caches
  the result in `sessionStorage` under `__source`, and the per-page reading is
  the easy mistake to make from a request log. The site now loads **no**
  third-party origin.
  `repo:web/docs-site/docs/stylesheets/laterite.css` maps Material's ~60 `--md-*`
  variables onto the token names; that mapping is the leverage, because it
  restyles the generated catalogue pages — one per AGS4 group — that nobody
  reviews by eye.
  **Two Material conventions bite here and both are commented at the rule:**
  its defaults are declared on `:root,[data-md-color-scheme=default]` — and that
  second half matches `<body>`, so for INHERITED properties it beats a `:root`
  override by proximity, not specificity (the symptom is a white canvas and stock
  black body copy while the chrome and nav, which take colour directly, look
  correct); and each admonition type is painted at **three** selectors
  (`.md-typeset .admonition.tip`), so a two-selector override leaves stock blue
  titles and teal icons under correctly-recoloured borders.
  `repo:web/docs-site/overrides` holds the two forked partials — the masthead
  (the navigational lockup `laterite | docs · v<version>`, whose version comes
  from the `version_stamp.py` hook rather than a second lookup) and the repo
  link. **Diff a fork against its upstream partial after a Material bump**;
  `web/.prettierignore` excludes them so the formatter cannot destroy that
  diffability by reflowing the Jinja.
  > The band-keyed left nav is the reason `repo:tests/test_docs_band_containment.py`
  > exists. Each top-level section takes the next colour from the strata ramp in
  > **nav order** (pure CSS `:nth-child(7n+k)`, so a renamed section keeps its
  > place), and the active item carries its section's band as a 3px inset rule.
  > That is safe ONLY while band colour stays off prose: on a documentation site
  > for a *validator*, the warm ramp and the severity palette are the same family,
  > so a rust-tinted callout is indistinguishable from an error. The gate parses
  > every hand-written docs stylesheet and fails if a band token reaches a
  > selector outside the allowlist (nav, TOC rail, masthead hairline, catalogue
  > cap). It is a gate rather than a convention because the failure **looks like a
  > design choice**, and because the change that would cause it — a CSS-only edit —
  > is why `web/docs-site/docs/stylesheets/**` had to join the `code` filter.
  > The **table-of-contents strata rail** (#402, `repo:web/docs-site/docs/javascripts/rail.js`)
  > is the hairline dose of the apex's borehole rail — six pixels, four bands, a
  > canvas veil and a steel probe, and deliberately **no numeric readout**: the
  > apex's rail reads out a depth because on that page the depth is the joke, and
  > a documentation page has none. It must appear and disappear exactly with the
  > table of contents, which CSS cannot ask about — so its media query carries a
  > COPY of Material's `.md-sidebar--secondary` breakpoint, and
  > `repo:web/docs-site/hooks/rail_breakpoint.py` reads that number out of the
  > CSS mkdocs-material shipped and holds ours to it. The copy rotted on its
  > first outing (76.1875em, the *layout* breakpoint, against the sidebar's real
  > 60em), which left a table of contents with no strip beside it on every window
  > between 960 and 1219px — invisible in a diff and in a screenshot at one
  > width. The hook WARNS rather than raises, so `--strict` fails it on a PR
  > while the deliberately non-strict deploy stays green.
- **Two CI gates, both via `ci.yml`'s `changes`-job filter** (see
  ci-and-runners):
  - *Build half* — the `docs` job `uv sync --group docs` (mkdocs-material +
    mkdocstrings) then `mkdocs build --strict`, so `--strict` fails on a broken
    internal link, missing nav page, unresolved snippet file, **or a broken
    autodoc cross-reference**. It is a **pure Python job**: it downloads the
    cdylib `build-ext` already produced rather than compiling one, so there is no
    Rust toolchain and no sccache in it — `repo:.github/workflows/ci.yml` says so
    at the step ("No Rust toolchain / build-accel here any more"). The `docs` filter includes
    `packages/laterite/python/**` so a docstring rename re-runs it.
  - *Runtime half* — `web/docs-site/examples/**` + `examples/sample_site.ags` sit
    in the `code` filter, so editing an example or the shared fixture re-runs
    the python + CLI example gates in the `python` job (which has the compiled
    wheel + `lat`); `web/docs-site/examples/node/**` + the fixture also
    sit in the `node` filter so Node-snippet edits re-run the vitest gate. The
    wasm tree has its own per-PR leg in the `web` job — `gen_doc_outputs.py
    --check --surface wasm` — on `web/**` + `rust-packages/**`. The duckdb
    gate's per-PR leg is include-check only (runtime is nightly + on-demand,
    above).
- **Deploy.** The mkdocs build in
  `repo:.github/workflows/deploy-validator.yml` deploys the docs to the public
  Pages site (`/laterite/docs/`). The build is deliberately **non-strict** — a
  docs link nit must never block the app deploy; the strict gate is the PR-time
  `docs` job above.
- **Structure** (`mkdocs.yml` `nav`): Home · **Learn** (ordered tutorial) ·
  **Cookbook** (task-indexed recipes, grouped Reading/Validating/Querying/
  Producing/Repairing/Sharing) · **Chaining** showcase · **Concepts** ·
  **Reference** (CLI · **Python API** (mkdocstrings) · **AGS data types**
  (glossary) · **Group catalogue** (generated, 174 pages) · **Support modules** ·
  Python cheatsheet). Phase 1 = the MVP (#281); Phase 2 = the 16 per-recipe
  cookbook pages + 7 concept pages (#283); the Python API reference + #215
  docstring sweep, then the group catalogue + type glossary, landed next.

> [!warning] Material wraps tables at runtime — verify docs JS/CSS in a real browser
> Material re-parents every table into a `.md-typeset__table` wrapper **at runtime**,
> so custom docs JS must not assume the table is a direct child of its markdown
> container — `wrap.prepend(node)`, **not** `insertBefore(node, table)` (the latter
> throws `NotFoundError` and silently aborts init). That same wrapper is
> `display: inline-block`, so a table narrower than the content column left-aligns
> and leaves an **asymmetric right-gutter**; the fix is site-wide
> `.md-typeset__table { display: block }` + `> table { min-width: 100% }` (min-width,
> so genuinely wide tables still overflow-scroll). A **static-HTML DOM shim misses
> the runtime wrap** and passes while the live page is broken — verify docs JS/CSS
> against the *served* site with **headless Playwright** (measure computed geometry /
> `getBoundingClientRect`), not a hand-built DOM. `localhost` is blocked in the
> in-browser MCP tool, so Playwright against `mkdocs serve` is the path (restart
> serve after a CSS edit — its file-watch can go stale).

- **No em dash reaches a reader, and the gate has two halves** (#588,
  `em_dash_gate`). The house style is that a dash standing in for a comma, a
  colon or a bracket should be the comma, colon or bracket; the [[validator-site]]
  landing holds that with a browser test over the rendered DOM, and this site
  holds it with `repo:tools/check_docs_em_dash.py`. `--built` is the gate: it runs
  in the `docs` job on what `mkdocs build --strict` just produced, because **three
  of this site's page families have no Markdown at all** — `reference/groups/` is
  174 pages from `catalogue`, and `reference/types/` and `reference/cli/` come from
  `glossary` and its sibling, so all of their prose lives in f-strings a
  `docs/**.md` walk never opens. The first draft scanned source only, reported
  clean, and left several hundred in the built site; that is the whole reason the
  built half exists. The buildless source half stays in `repo-gates`, and the two
  are **complementary — neither subsumes the other**. The built half excludes
  `reference/api/` and `reference/modules/` by path, and both of those pages are a
  *mix*: a hand-written intro and section prose wrapped around the generated API
  reference. So the source half is the only gate those paragraphs have, while the
  built half is the only gate the three generated families have.

> [!note] Deferred (the Phase-2 tail)
> Done: the **Python** API reference (mkdocstrings), the **group catalogue** +
> **AGS data-type glossary** (generated from the registry/dictionary and
> content-gated), and the **Node / CLI / DuckDB example trees + gates** (#373,
> which tabbed the high-value cookbook recipes). **#380 finished the tail:** the
> remaining recipes (filter-select / borehole / diff / build-from-frames /
> build-from-typed-graph / list-rules) are now tabbed with newly-gated
> Node/DuckDB/CLI examples, and the transport **lock/unlock** round-trip is gated
> too (a scrypt-`log_N`-18 example — the Node docs gate's per-example timeout was
> raised to 90 s for the CI-container KDF thrash, the #369 lesson). So **all 16
> cookbook recipes are tabbed + every published snippet is executed in CI**; the
> #201 `validate-anywhere.md` synced-tabs prototype was retired (its hand-inlined
> snippets superseded by the gated `validate-a-delivery` recipe — [[reliquary]]
> #17). Still pending: **TypeDoc** for Node (static `.d.ts`, no wheel), the
> generated **rules / typed-graph** catalogues, and the **molab** `/try/` embed.

## What the gates do not reach

Every published snippet is executed, which makes it easy to read this page as saying
the site is *covered*. It is not the same claim. A one-off walkthrough (#510) read the
site the way a reader would — one agent per surface, running the documented examples
against the **released** packages rather than the working tree — and its most durable
output was not the nine defects it found but the list of places nobody had looked.
A silent skip reads as coverage, so those scopes belong here rather than in a
gitignored report that outlives nothing.

None of the below is a known defect. Each is a place nobody has looked.

- **A real browser.** The wasm examples run under Node, as `reference/wasm-api.md`
  itself does. Untested: bundler asset-URL resolution, Web Workers, `ArrayBuffer`
  transfer, and the app's own panes. The app compiles the crate from source with every
  cargo feature on, so exercising it would not test the published
  `@laterite/ags4-wasm` package either — these need separate coverage, not one
  standing in for the other.
- **Transport in the browser.** `cookbook/transport.md` says the browser reads the same
  zstd+age envelope; the released wasm package exports no transport functions at all.
  Whether the app — built from source, so with `transport` available — does is the open
  half, and it decides whether that page is wrong or merely unqualified about which
  build it means.
- **DuckDB beyond Python's bindings.** Not the CLI, not `@duckdb/node-api`, not wasm.
  `duckdb/index.md`'s "no download" claim was reasoned about rather than run under a
  blocked network, which is the only way to test it.
- **Non-macOS CLI builds**, and `--tui`.
- **Rendered signatures.** Every symbol `reference/api.md` names exists on the released
  package, but the built site's mkdocstrings *signatures* have never been diffed
  against the real ones. `strict: true` fails on a name that no longer resolves; it has
  nothing to say about one whose shape changed underneath it.
- **Everything outside the docs site** — the PyPI, npm and crates.io landing READMEs
  were out of scope by choice.
- **Three built page families, for the em dash gate specifically** (#588,
  `em_dash_gate`). `reference/api/` and `reference/modules/` are mkdocstrings
  rendering the wheel's own docstrings, and `reference/cli/` is the shipped
  `lat --readme` guide mirrored into four packages: rewriting either would be an
  API or a shipped-binary change, not a docs edit, so the gate counts them and
  prints the counts rather than reading them. The first two are a *mix*, and a
  path prefix cannot say so: their hand-written halves are gated by the source
  scan reading `docs/reference/api.md` and `docs/reference/modules.md`. Two
  narrower spots go with them —
  the short note `gen_groups.py`'s sibling `gen_cli.py` writes *above* that
  guide is ours but sits inside an excluded page, and attribute text other than
  the meta description (an `alt`, a `title`) is dropped with the tags.

**One entry has since been answered, and is recorded as corrected rather than
deleted.** #510 listed `chaining/index.md` as entirely unaudited and a plausible
carrier of release-skew. The page `--8<--`-includes its examples, and nightly's
`docs-vs-released-wheel` leg runs `test_docs_examples.py` + `test_docs_snippets.py`
against the **released wheel** — which is precisely the release-skew check that entry
worried nobody was doing. What remains unreached there is a human read of its prose,
which is a much smaller claim than the original.

## Why it matters

A born-typed, fluent API is only as good as its discoverability. The example-led +
CI-gated design is the same "the authority enforces its own doc" philosophy as the
dictionary-faithfulness and `OBSERVATIONS` gates: the docs **cannot drift** from
the shipping API because every published snippet is executed in CI, and the strict
link gate stops the prose from rotting. It is a public surface, so it deploys from
the mirror only — keeping the private preview free of the docs.

## Diagram

```mermaid
flowchart LR
  ex["examples/{python,node,cli,duckdb}/*"] -->|--8<-- snippet| page["docs/**.md"]
  ex -->|subprocess + asserts| gate["per-surface gates: test_docs_examples.py ·<br/>docs-examples.test.ts · test_docs_cli_examples.py (.out) ·<br/>test_docs_duckdb_examples.py (nightly, published ext)"]
  wheel["uv sync --group docs<br/>(local working-tree wheel)"] -->|mkdocstrings introspects| page
  reg["laterite.registry + ags_dictionary.json (eds)"] -->|gen_groups/gen_types| page
  cd["catalogue_data.py<br/>(families, provenance, TYPE_GLOSSARY)"] --> page
  cd --> cgate["tests/test_groups_catalogue_faithful.py<br/>(dev satellite, python job)"]
  page --> strict["mkdocs build --strict<br/>(docs job, builds the wheel)"]
  strict --> art["web/dist/docs"]
  vite["vite build → web/dist"] --> art
  art -->|mirror only| pages["GitHub Pages /laterite/docs/"]
```

## Where it shows up

- ci-and-runners — the `docs` job + the `code`/`docs` `changes`-filter entries.
- [[validator-site]] — shares the Pages artifact + `deploy-validator.yml`.
- [[playwright-e2e]] — the sibling "gate the real shipped artifact" pattern.

## Related
- ci-and-runners
- [[validator-site]]
- [[playwright-e2e]]
