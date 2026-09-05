# CLAUDE.md

This file guides Claude Code (and any agentic contributor) working in this repository. It is the AI-assistant companion to `CONTRIBUTING.md`.

> **This repo is the development home.** `niko86/laterite` holds the shipped
> product **and** the knowledge base (`ags-wiki/`), and releases publish from
> here. The **shipped** product is the **AGS4** toolchain (validator + core +
> bindings + wasm + the `lat` CLI). The experimental **AGS5** strand (`.ags5db`
> DuckDB store + `.agsx` tar/zstd format) is a **dormant concept**, not shipped
> and not in this tree — represent it as an idea, never as a shipped feature.

## Knowledge base — consult the wiki first

The **why** — design decisions, format concepts, the crate map, the O-1..O-N
divergence catalogue, editions, and per-crate tool pages — lives in **`ags-wiki/`**
in this repo. **Reach for it before** answering an AGS4 question, changing
validator/parity behaviour, or touching the architecture (crate layout, the
wheel/dep/build split, the PyO3 or wasm boundary) — **and before asserting how any
part of this repo behaves**, in an issue, a PR body, a review comment or a triage
note.

That last trigger is the one most easily missed, because it fires when you are
**not editing anything**, so neither of the others does. A wrong claim published to
the tracker outlives the session that made it and gets acted on by someone else.
And much of the wiki — `concepts/`, `design/`, `tools/`, `strategies/` — is about
how the **repo** works (CI, the docs gates, e2e, testing strategy) rather than how
AGS4 does, so "not an AGS4 question" is not a reason to skip it. Worked example: an
issue was filed claiming a docs gate read a marker nothing consumed;
`ags-wiki/concepts/docs-site.md` already described the two-gate split that made it false.

**To find what covers the files you are about to change — or the files a claim you
are about to publish is about** — ask; don't guess from filenames, and don't assume
nothing covers them:

```bash
uv run --no-project python ags-wiki/.bootstrap/librarian.py --paths <files…>
```

Under a second, no build. It inverts the `repo:` citations already on the pages
and prints them ranked, with their titles, marking a hit that comes only from a
page citing a parent *directory* as `(directory only — may not describe this
file)`. A stem lookup cannot answer this: the page covering
`rust-packages/laterite-ags4-reference/src/dict.rs` is called
`edition-resolution`. It is a lookup, not a gate — nothing checks that you ran
it.

For orientation rather than lookup: `ags-wiki/start-here.md`, and
`ags-wiki/concepts/crate-map.md` is the keystone crate map.

Treat a page as a pointer, not gospel: verify a load-bearing claim against the
repo authority it cites (`observations.json`, `ags_dictionary.json`, the validator
rule modules) before relying on it — code can move under a page.

**The OBSERVATIONS canon is a code SSOT.** The O-N divergence catalogue's source
of truth is `observations.json` (repo root); `OBSERVATIONS.md`, the wiki's
coverage-map lists, and `web/docs-site/docs/reference/divergences.md` are its
rendered views. A record reaches that last one by carrying a `user_facing`
block (a reader-facing `summary`) alongside a `relation` block (`python` +
`spec` + `converged`, the first two over closed sets the generator enforces) —
membership is a decision recorded on the record, never derived from `kind`, and
so is the SECTION it renders under, which is derived from `relation` and must
never be authored beside it — and a record marked `status` (with `resolved_by`)
may not also be `user_facing`. To add or change an O-N: edit
`observations.json` (aim for the house style — observed / spec / assessment /
upstream-reportable / our decision — next free O-N, clean-room, accurate), then
**regenerate** with `uv run --no-sync python tools/gen_observations.py` (never
hand-edit a rendered file). Set `upstream: true` for anything worth taking to the
AGS-DFWG — the `## Upstream-reportable` table is rendered from that flag.

Every O-N also needs a wiki page — one zero-padded `O-NN.md` per record under
`ags-wiki/observations/`, copied from
`ags-wiki/templates/_template-observation.md` — that **links and cross-references
but never copies the fields**, with `obs_tag` and `upstream_reportable` matching
the record. `tools/gen_observations.py --check-wiki` holds the two in agreement
and is a CI gate; `--lint` reports house-style departures without rewriting them.

> Private, dev-only notes (CI/runner operations, the dormant AGS5 strand, the QA
> tool design pages, session-workflow recipes) are **not** in this public repo —
> they live in the private satellite checkout. If a `CLAUDE.local.md` is present
> (gitignored), it points to them.

## Commands

Everything goes through `uv` — this is a uv workspace, not a plain venv project.

```bash
uv sync                                          # install laterite + dev deps
uv run pytest packages/laterite/tests -q         # the shipped wheel's own tests
cd packages/laterite && uv run --no-sync maturin develop --release --uv  # rebuild the PyO3 wheel
uv run pytest packages/laterite/tests/test_certificate.py -v         # one file
uv run pytest packages/laterite/tests/test_certificate.py::test_xxx -v   # one test by node id

uv run ruff check .                              # lint (the only linter; CI runs this exact command)
uv run ruff format --check .                     # format gate (CI enforces it)
uv run ty check                                  # Astral `ty` type-faithfulness gate over the shipped package

uv run lat validate delivery.ags --json          # the shipped AGS4 validator CLI (laterite-cli)

cargo run -p laterite-ags4-forge -- check <file-or-dir>   # dual-validate against python-ags4:
                                                 # each side's rules + per-rule counts + the parity
                                                 # verdict. A directory sweeps recursively. This is
                                                 # how you compare the two engines on real files.

./tools/run_python_ags4_tests.sh                 # python-ags4's own suite vs laterite.compat
                                                 # (parity oracle; needs ../ags-python-library cloned from GitLab)
```

**Run the gates before pushing, in this order.** `tools/check_changelog.py --base
origin/main` **first** — it is surface-independent, so it is the one a
"what did my change touch?" reading misses. Then `git add -A`, because the
tracked-file scanners (`check_doc_refs.py`, `check_issue_refs.py`, the generator
`--check` gates) are blind to a new file that is still unstaged and will report
green over it. Then the touched surfaces' full sets — lint AND format AND types AND
tests, not whichever one seems relevant. Derive that list from the workflows rather
than from the diff: a default-feature clippy cannot see a broken `#[cfg]`, and node
carries its own prettier.

The changelog gate **refuses to report success on an empty diff** ("a gate that sees
no diff has checked nothing"), so run it against a commit, not a dirty tree — a pass
before you commit is not a pass.

The python-ags4 parity runner shims `python_ags4` to `laterite.compat`. The
`parity` CI gate (`.github/workflows/parity.yml`) enforces the failing SET **by
identity** (`parity-known-failures.json`, `tools/check_parity.py`) — a required
merge check.

**Three programs answer to `lat`**: the shipped Rust binary (`laterite-cli`), the
wheel's console script, and the Node launcher. They are the same tool **by contract,
not by construction** — one guide, mirrored into each package and gate-held
byte-identical — so a gate that resolves `lat` from `PATH` tests whichever one the
environment happened to put there. Name the one you mean: `LAT_BIN` is how
`tools/gen_doc_outputs.py` is pointed at a specific binary, and it prints what it
resolved rather than assuming.

The dev workspace floor is Python ≥ 3.12 (`requires-python` in the root
`pyproject.toml`), matching the shipped wheel. There is **no `.python-version`**
— this line used to claim the dev interpreter was pinned to 3.14 by one, and no
such file has ever been tracked. The **shipped** `laterite` wheel is
**abi3-py312** → installable on **≥ 3.12** (green on 3.12/3.13/3.14).

**Dep-shape split:** `pip install laterite` installs **polars + duckdb** only —
no pandas, no pyarrow. The python-ags4 drop-in surface (`laterite.compat`) lives
behind the `[compat]` extra (`pip install laterite[compat]` adds **`pandas<3`
only — pyarrow-free**). pyarrow is an OPTIONAL accelerator (`[compat,pyarrow]` /
`[pyarrow]` / `[all]`), auto-detected at runtime. **DuckDB is load-bearing in the
base** — it is the pyarrow-free dataframe bridge; dropping it would reintroduce
pyarrow for the pandas path. (Verified against `packages/laterite/pyproject.toml`.)

## Architecture

This section is the *how*; the **why** is in the wiki (see above), starting at
`ags-wiki/concepts/crate-map.md`.

### Registry-driven model generation

The core architectural commitment: **AGS group definitions live in one JSON file**,
and everything else is generated.

`rust-packages/laterite-ags4-reference/data/ags_dictionary.json` is the single
source of truth — **174 AGS groups** (the union across editions 4.0.3–4.2); each
entry has a 4-letter code, a parent code (or `null`), and an ordered tuple of
headings (status KEY/REQUIRED/OTHER, AGS type, unit, description).

Generators that consume the JSON:

1. **Rust `build.rs`** (`rust-packages/laterite-py/build.rs`) — emits one
   `#[pyclass]` per group into `$OUT_DIR/typed_groups.rs`. The codes live in the
   **`laterite.groups`** submodule (`from laterite.groups import PROJ, LOCA, …`),
   not the top-level namespace.
2. **Python `.pyi` generator** (`tools/generate_pyi.py`) — emits the IDE/mypy
   stub. A CI drift gate guards against dictionary-edit drift.
3. **Validator dictionaries** (`rust-packages/laterite-ags4-validator/build.rs`
   via `laterite-ags4-reference`) — projects each edition into `phf` static
   tables; the wasm build consumes the same data.

**Adding a new AGS group** is a one-file edit: edit the dictionary JSON, rebuild
(`maturin develop`) to refresh the `#[pyclass]` codegen, then
`uv run python tools/generate_pyi.py`. The drift tests fail loud if you forget.

### The shipped AGS4 pipeline

Rust workspace crates (mapped in `ags-wiki/concepts/crate-map.md`); the
load-bearing chain:

- `laterite-ags4-types` — shared typed-Arrow column builder + the AGS type system
  (`ags_types`), a wasm-safe leaf.
- `laterite-ags4-parse` — the shared parse leaf: one tolerant tokenizer
  (`split_ags_line`/`field_span`) + one source-true byte/line/char walk
  (`parse_bytes`/`parse_str`). Deps `encoding_rs` + `memchr` only.
- `laterite-ags4-reference` — the reference-data leaf: the multi-edition
  dictionary (`ags_dictionary.json` + its `phf` projection) + the rules-catalogue
  accessors, mechanically derived from bundled JSON.
- `laterite-transport` — the shared zstd + age passphrase file envelope
  (`pack`/`unpack`/`lock`/`unlock`); behind core's `transport` feature (not
  wasm-clean).
- `laterite-ags4-core` — DuckDB-free pure-string core: the dictionary registry,
  the AGS4 read codec (`ags4_codec`), and the `.ags.idx` cert + byte-offset index.
- `laterite-ags4-emit` — byte-faithful AGS4 writer/emitter.
- `laterite-ags4-validator` — the clean-room numbered-rules engine; consumes its
  dictionary editions + rules catalogue from `laterite-ags4-reference`.

Surfaces: `laterite-py` (PyO3 → the `laterite` wheel), `laterite-node` (napi-rs),
`laterite-ags4-wasm` (browser). CLI: **`lat`** (`laterite-cli`). Supporting
crates include the `laterite-ags4-{parity,forge,corpus-qa,perf}` QA tools,
`laterite-ags4-xcheck` (cross-surface output-value gate), and `laterite-ags4-excel`
(AGS4↔XLSX, extracted so its `calamine`/`rust_xlsxwriter` deps don't ride into
every consumer).

Shipped Python import surface (`packages/laterite/python/laterite/`):
- `from laterite.groups import PROJ, LOCA, SAMP, …` — the 174 typed-graph classes.
- `from laterite import read, validate, build_ags4, BuildResult, Report, Ags4File, AgsQuery` — the read/validate/emit surface (`.read()` accepts path/text/bytes; `.certify()` mints an `.ags.idx`; a fresh+matching cert lets `.validate()` skip the rule engine).
- `from laterite.ags4 import read_typed` · `from laterite.dynamic import get_or_register` · `from laterite.transport import pack, unpack, lock, unlock` · `from laterite.registry import GROUPS, GroupDescriptor, Heading, child_groups` · `from laterite.ags_types import canonical_type, parse_value, CanonicalType` · `from laterite import compat as AGS4`.

Distribution & language strategy: `lat` is the shipped Rust binary
(`tools/build-rust.sh`); the Python `laterite` package is the primary library
surface + parity oracle. The intended Rust↔Python boundary is **Rust drives
Python** (PyO3 / embedded), never the reverse — see
`ags-wiki/design/dec-rust-drives-python.md`.

## Conventions

Comments lead with **why**, not what — see
`rust-packages/laterite-ags4-core/src/index.rs` and
`rust-packages/laterite-ags4-validator/src/parse.rs` for the established tone.
Don't narrate obvious code.

**Never write a MEASURED value into prose** — a comment, a doc, or a wiki page. A
number some tool recomputes on every run (the PWA precache weight, an artifact's
size, a coverage percentage, a benchmark timing) belongs where something *reads*
it: a gate, a threshold, an assertion. In prose it acquires no reader, so nothing
fails when it drifts — and it drifts on the very next build. Name the
**instrument**, not the reading: "the figure `vite-plugin-pwa` prints on every
build", never the figure.

Worked example (#345): the precache weight was stated in three places and gated
in none. Each was corrected in turn and wrong again within days — including
inside the comment that explained the mechanism ("a stale number in a gate fails
the build, a stale number here fails nobody").

Three carve-outs, and only three:

- **A gated threshold** — cite the gate **by name, never the number inside it**.
  `tools/release/check-wasm-tier1.mjs` fails CI when its ceilings are crossed, so
  they cannot go stale; a copy of one in prose propagates it again and rots
  identically when the ceiling moves. (This is the carve-out most easily misread
  as "numbers with a gate are fine". They are not.)
- **A historical series** — 3.3 MB → 4.8 MB → 6.6 MB describes builds that
  already happened, so it cannot drift.
- **Loose orders of magnitude** — "a multi-MB engine", "tens of MB". Third-party
  artifacts this repo neither builds nor measures (DuckDB's ~36/41 MB) count here.

**Surgical, targeted changes.** Change only what the task needs; don't refactor,
"improve", or reformat adjacent code beyond scope — smaller diffs succeed more and
regress less. Spotted relics (dead code, stale comments, redundant tests) get
**inventoried first** (an issue, or the active design page), then removed
**deliberately, one at a time**, each in the phase/PR that retires the feature
they belong to — never an indiscriminate sweep. The living relic register is
`ags-wiki/design/reliquary.md`.

**A gate that drops input says what it dropped.** Every gate here carries a
scope — a class of input it silently does not look at — and nothing audits those
scopes, so a gate reports green while being blind to exactly the thing that later
breaks. The worked example is `tools/check_doc_refs.py`, whose precision
heuristic skips backticked tokens containing no `/`: `compat.py` was cited in the
docs and never checked, and the blind spot surfaced months later by someone
tripping over it rather than by the gate that was meant to catch it. So report
what was filtered out **on every run, pass or fail** — a count is enough, and
`tools/check_package_contents.py` and `tools/check_released_crate_readmes.py`
already do it. A filter nobody can see is a blind spot with a green tick on it.
(#295 found three of these in one day; #460 — `check_doc_refs.py` itself, the
last gate owing its report — was discharged, and that gate now prints its
skipped-token count with a `--skipped` flag to list them.)

**A bare `#N` means an issue or PR in THIS repo.** That is what a reader
assumes and what GitHub autolinks, so a number that means somewhere else carries
its repo — `laterite-dev#512` for the dev satellite, `microsoft/mimalloc#1327`
upstream. Getting it wrong is not a dead link: a borrowed number resolves to
nothing until this repo's numbering climbs past it, and then starts resolving to
a real, plausible, unrelated page with nothing failing anywhere (#458).
`tools/check_issue_refs.py` holds the numbers already known to be foreign; it
cannot judge a new one, and says so on every run.

`output/` is gitignored working space. `experiments/` holds dictionary
scaffolders — not production code.

### Working in this environment (tool-call hygiene)

- **Small batches.** Parallelize only genuinely independent, read-only calls, and
  keep it to a handful. Never put many mutating calls in one message.
- **No mixed-dependency batches.** If step B reads what step A wrote (write →
  reindex → lint → commit → push), make them separate, sequential,
  individually-checked calls — one failure in a batch cancels its siblings.
- **Absolute paths always.** Don't rely on cwd.
- **Keep each Bash call plain.** In a worktree-isolated session the harness refuses
  any command it cannot verify stays inside the worktree — compound pipelines, `for`
  loops, heredocs feeding a second command, output redirects. One command per call
  costs a round trip; a refused one costs two and returns nothing.
- **Verify against disk/git, then act.** When results look off, re-read the file
  or `git status` *first*, in its own step. Git state is the source of truth, not
  a possibly-stale tool echo.
- **Disk is tight here.** `rust-packages/target/debug` balloons (DuckDB + PyO3).
  Reuse the workspace target dir; reclaim with `rm -rf target/*/incremental`
  before heavy builds.
- **Branch hygiene.** Reconcile in one read-only pass at session start and after
  PR merges: `git fetch --all --prune`, fast-forward `main`, delete merged local
  branches; surface stale remote branches for the owner rather than deleting.

## Agent skills

Configuration the installed engineering skills read. These files answer *where*
things live; they do not restate the rules above.

### Issue tracker

GitHub issues on `niko86/laterite`, via the `gh` CLI. External PRs are **not** a
request surface. See `docs/agents/issue-tracker.md`, which also carries the PR
rules a skill would otherwise get wrong (every change goes through a PR, the
maintainer merges, a stacked PR runs zero CI).

### Triage labels

The five canonical roles, each label string equal to its name. Only `wontfix`
exists on the repo today; the other four have to be created. See
`docs/agents/triage-labels.md`.

### Domain docs

Single-context, but **not** the usual `CONTEXT.md` + `docs/adr/` layout — this
repo's domain layer is the wiki, so `ags-wiki/start-here.md` stands in for
`CONTEXT.md` and `ags-wiki/design/dec-*.md` for the ADRs. See
`docs/agents/domain.md`.
