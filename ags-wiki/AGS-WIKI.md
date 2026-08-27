# AGS Wiki — Schema & Operating Manual

> **This is the configuration file for the maintaining agent.** Read it
> first before any Ingest / Query / Lint on this vault. It is *not*
> knowledge content — it is the rules of the game. It deliberately does
> **not** modify or duplicate the codebase's own instruction files
> (those govern the code); this file governs only
> `ags-wiki/`.

## 0. Purpose & the three-layer model

A persistent, LLM-maintained knowledge base for the **AGS4 geotechnical
data format** (editions **4.0.3, 4.0.4, 4.1, 4.1.1, 4.2** — *AGS3 and
earlier are out of scope*) and this repo's AGS toolchain.

| Layer | What | Who edits |
|---|---|---|
| **Raw sources** | The 5 spec PDFs at this vault's root + `sources/external/` drops + the in-repo authorities (dictionary, OBSERVATIONS, `*.rs`, xlsx) | nobody — immutable; the agent *reads* only |
| **The wiki** | Every `*.md` page here (rules/ groups/ types/ observations/ tools/ concepts/ editions/ comparisons/) | the agent, exclusively |
| **The schema** | this file | co-evolved with the user |

Scale is bounded: **174 groups** — the `groups/` reference tier is
generated from `ags_dictionary.json` (`tools/gen_reference_groups.py`,
gated by its own `--check` in the `wiki-lint` job; the AGS4 union, D6)
plus 3 hand-authored AGS-L draft pages — alongside ~28 rules+subrules,
~18 types, ~45 observations, ~21 tools (see `index.md` for exact live
counts). The `index.md` catalog is sufficient navigation — **no
embeddings/RAG/qmd** unless the vault outgrows that (future option, not
now).

## 1. Cardinal Rule — LINK, DON'T DUPLICATE

The **repo is the source of truth**. Pages *synthesize, diagram, and
cross-reference*; they never paste source text from
`ags_dictionary.json`, `OBSERVATIONS.md`, the rule `*.rs` files, or
the bulk spec PDF (the 154-page Data Dictionary, prose sections).
Every factual claim carries a citation.

**Narrow exception — load-bearing structured primitives.** Three
classes are reproduced faithfully *with citation* because they are the
analytically load-bearing primitives of this whole wiki and a
paraphrase would defeat the gap-hunting purpose (gaps routinely hinge
on one token, e.g. Rule 19 "letters **and numbers**", `SAMP_LINK`
being `RL`):
1. the ~32 one-line **normative Rule statements** (`spec:` §4.1.1);
2. the ~17 **TYPE definitions** (`spec:` §3.3);
3. the per-group **heading table** (Heading · Status · Type ·
   Description) — rendered *mechanically from the repo's own
   `ags_dictionary.json` model authority* (not the 145-page spec
   prose), cited `repo:…ags_dictionary.json groups[code=…]`.
Everything else stays link-only — never the spec PDF's prose sections,
worked examples, suggested-unit tables, or descriptive narrative. If a page and
its cited source ever disagree → **flag it** (`status: contradicted` +
a `> [!spec-ambiguity]`/`> [!divergence]` callout), never silently
"correct" either side.

**Corollary — a MEASURED value is not a fact to copy either.** The rule above is
about what the repo **defines**; this is about what it **measures**. A number some
tool recomputes on every run — an artifact's size, the PWA precache weight, a
coverage percentage, a benchmark timing — never belongs on a page: in prose it
acquires no reader, so nothing fails when it drifts, and it drifts on the next
build. Cite the **instrument** that prints it, or the **gate** that holds it — the
gate by name, not by the number inside it.

The general form, its three carve-outs (a gated threshold, a historical series,
loose orders of magnitude) and the worked example that earned it are in
`repo:CLAUDE.md` under *Conventions*. They live there rather than here because
that rule governs the whole repo and this manual governs `ags-wiki/` only (§0) —
so this states the wiki half and defers, rather than keeping a second copy that
would drift from it, which is the very failure both rules are about.

**Citation grammar** (inline code span):
- repo file/line — `` `repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs:352` ``
- repo symbol — `` `repo:.../rules/mod.rs::run_all` ``
- OBSERVATIONS entry — `` `repo:OBSERVATIONS.md#o-33` ``
- dictionary entry — `` `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=SAMP]` ``
- spec PDF — `` `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 8` `` (the PDF itself is not
  vendored — the citation names the file, not a path; see `sources/spec-*`)
- AGS library xlsx — `` `AGSL4_2_TRI.xlsx` `` (not vendored either, and for
  the same reason the citation names the file rather than a path; see
  [[ags-library-xlsx]])

Paths are repo-root-relative, forward-slash, no backslashes.

**A `spec:` citation on a `> [!quote]` block asserts the words are the spec's,
and no gate here can check that.** The check needs the published PDFs, which are
copyright AGS, not redistributable, and deliberately absent from this tree — so
nothing in CI reads them and nothing ever will. A wrong quotation therefore looks
exactly like a right one, indefinitely, and the callout that means "these are the
spec's words" is visually identical to the ones that mean "here is where this is
implemented".

The check itself is cheap for anyone holding local copies: normalise both sides
to alphanumerics, lowercased, and ask whether the quoted text is a **substring**
of the clause. Verbatim passes; a compression cannot. Run it against every
edition, not just the newest — a page may legitimately quote an older one.

The consequence to state plainly, because a green wiki-lint does not: **quoted
spec text is only ever as good as the last manual pass.** #756 is what this note
is for — five pages had silently compressed their clause, three of them dropping
a normative "shall", and the convention had been honoured on every other page for
months while those five sat unread.

## 2. Directory & naming

```
rules/ groups/ types/ observations/ tools/ concepts/ editions/
comparisons/ sources/{repo-authorities/,external/} templates/
+ AGS-WIKI.md start-here.md index.md log.md  + 5 *.pdf (raw spec layer)
```

| Class | Filename | Example |
|---|---|---|
| rule | `rule-NN[-sub]-<kebab>.md` (NN zero-padded; each sub-rule its own page) | `rule-08-typed-values.md`, `rule-10a-key-uniqueness.md` |
| group | `<CODE>.md` (exact 4-letter AGS code) | `SAMP.md` |
| type | `<code>.md` (verbatim AGS token) | `nDP.md`, `0DP.md` |
| observation | `O-NN.md` (zero-padded) | `O-07.md`, `O-33.md` |
| tool | `<repo-artifact-name>.md` | `laterite-cli.md` |
| concept/comparison | `<kebab>.md` | `parity-model.md` |
| edition | `ags-<version>.md` | `ags-4.2.md` |
| meta | leading `_` (sorts top, Dataview-excluded) | `_template-rule.md`, `_registry.md` |

**Filename stem == the `[[wikilink]]` target == the index/log key.**

## 3. Page taxonomy → template

Every new page is created by copying `templates/_template-<class>.md`
and filling it. One class → one template → one frontmatter `type:`.

`rule · group · type · observation · tool · concept · source ·
edition · comparison`

**Campaign classes** (added Phase 0 of the Ingest campaign — see
`.bootstrap/INGEST-PLAN.md`): `insight` (gap register) · `strategy`
(Rust↔python test-probe register) · `decision` (the `design/` strand).
These are *campaign-authored*, not bootstrap stubs — they progress past
`stub` (see §13). Phase 0 also added `requirement` and `experiment`,
retired by #500: both were AGS5-strand classes, the strand is dormant and
lives in the private satellite, and neither ever had a live page here.

## 4. Frontmatter schema

YAML on **every** page. Common: `type, title, status, tags[],
related[], sources[], repo_refs{}, ags_editions[]`. Per-class extras &
controlled vocabularies are documented in each template. Vocab:

- `status` ∈ `stub | drafted | reviewed | stale | contradicted`
- `obs_tag` ∈ `VARIANCE | SPEC | BUG | NOTE`
- `phase` ∈ `V1 V2 V3 V4 V5 V6 V7 V8 | Post-V8`
- `rule_family` ∈ `line | structure | naming | dictionary | typed | groups | relational | references`
- `canonical_type` ∈ `string | integer | decimal | datetime | date | time | bool | enum`
- `tool_kind` ∈ `cli | crate | python-package | script`; `language` ∈ `rust | python | powershell`
- `source_kind` ∈ `spec-pdf | repo-authority | xlsx | external-doc | dfwg-thread`
- `ags_editions` ⊆ `[4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]`
- `gap_kind` ∈ `spec-ambiguity | spec-contradiction | cross-edition-regression | spec-vs-rust | rust-vs-python | rule-weakness`
- `insight.status` ∈ `hypothesis | probed | confirmed | ratified | refuted`; `severity` ∈ `low | med | high`
- `strategy.status` ∈ `proposed | probed | confirmed`
- `decision.status` ∈ `proposed | accepted | superseded | rejected`
- `tool.status` also allows `superseded` — a retired-package page kept as a redirect-stub tombstone (the same terminal word `decision` uses)
- `superseded_by` — **required on any `status: superseded` page** (D4): the successor page stem(s) (`[a, b]` or a bare `a`), each of which must resolve; the page must *also* carry a read-time tombstone callout (`> [!…]` naming the supersession). A superseded page missing either is invalid; these pages get a dedicated **Retired / Superseded** index section.
- `location` (`type: source`, A12) — where the artifact is, and Lint resolves it: a `repo:` citation (resolved by A1 under the §1 grammar), a repo-relative path that must exist on disk, or `"<filename> — not vendored; see the body"` for an artifact this repo doesn't hold (and then the named file must genuinely be absent — the marker is a checked claim, not a way to silence a dead path).
- `owns` (optional, D7) — a list of topic slugs this page is the single authority for (e.g. the DuckDB decision cluster: `duckdb-sql-extension` / `duckdb-host-engine` / `duckdb-read-path-perf`). A slug must not be claimed by more than one page — one owner per topic.

Dataview is **opt-in**: frontmatter is authored regardless so live
rollups work if the plugin is on; `index.md` keeps static tables so
the vault is useful without it.

## 5. Wikilinks & the no-orphan rule

Use `[[stem]]` or `[[stem|alias]]`. Every page **ends with a
`## Related`** block of typed wikilinks; `related:` frontmatter mirrors
it. Every page must be reachable from `start-here.md` within **2
hops** (Lint enforces). `cross_obs` links between observations must be
**reciprocal**.

## 6. Mermaid conventions

Every page class has a **required diagram**. At **bootstrap** only the
*mechanical* diagrams are filled (pure derivations of an authority —
structure, not synthesized insight); narrative diagrams stay
` ```mermaid `…`%% TODO (Ingest) `…``` ` placeholders.

| Class | Diagram | Bootstrap |
|---|---|---|
| `start-here` | master group hierarchy `graph TD` | generated from dictionary parents |
| `concepts/parent-child-graph` | full PROJ-rooted tree `graph TD` | generated |
| group `<CODE>` | local `erDiagram` (KEY tuple + parent inherited keys + children) | generated |
| `concepts/rule-families` | `graph LR` family→rule-numbers | generated from `rules/mod.rs` |
| `concepts/traceability-chain` | `flowchart LR` Rule→.rs→fixture→regression→O-N | generated (pattern) |
| `concepts/parity-model` | `stateDiagram-v2` Parity verdicts + reconcile→O-N | generated from `parity.rs` |
| `editions/*`, `concepts/edition-resolution` | `timeline` 4.0.3→4.2 | generated |
| `observations/*`, observation cross-ref hub | `graph LR` O-N edges | generated from OBSERVATIONS cross-refs |
| rule / type / group | `## Variations` mermaid (edition delta + Rust↔python) | **placeholder** (Ingest) |

## 7. Markdown toolkit (use these)

- **Callouts** — map to the observation taxonomy and flag everything
  notable: `> [!variance]`, `> [!spec]`, `> [!bug]`, `> [!note]`,
  `> [!divergence]` (Rust↔python-ags4), `> [!spec-ambiguity]`,
  `> [!todo]` (unfilled).
- **`## Variations`** — standard section on `rule`/`group`/`type`/
  `edition` pages: (a) cross-edition deltas 4.0.3→4.2, (b)
  Rust↔python-ags4 divergence (link `[[O-NN]]`). Frontmatter
  `varies_between_editions: true|false` + `divergences: [O-NN]` drives
  the Dataview "what changed / where we differ" rollups.
- **Transclusion / block refs** — `![[O-33#Summary]]`, `^anchor` to
  compose MOCs without restating (intra-wiki DRY).
- **MOC pages** — `start-here.md` is the hub; each class folder is a
  navigable cluster. Complements Obsidian graph view.
- Static tables in `index.md`; ` ```dataview ``` ` example blocks live
  in `start-here.md` (degrade gracefully if plugin absent).
- Future-only (do **not** build now): Obsidian Canvas, Marp decks.

## 8. `index.md` protocol

Regenerated **in full** at the end of every Ingest and every
page-creating Query. Frontmatter `counts{}`; one table per class
(page · key attrs · status · linked O-N); a **Gaps** section listing
entities present in an authority but lacking a page. Never hand-edited
between regenerations. The class list (dir/label/column, per-class
`content`/`campaign` flags) is single-sourced in
`.bootstrap/wiki-classes.json` — `reindex.py` (sections) and `lint.py`
(the `CONTENT` scaffold scope) both read it (D5). Each class dir is
scanned **recursively** (`rglob`), so a page in a subfolder — e.g.
`sources/repo-authorities/*.md` — is catalogued, not invisible (D3).
`status: superseded` pages are lifted out of their class table into a
dedicated **Retired / Superseded** section (and excluded from the class
count), each row linking to its `superseded_by` successor — a read-time
tombstone rather than a live reference (D4).

## 9. `log.md` protocol

Append-only; never edit/delete prior lines. One entry per op:

```
## [YYYY-MM-DD] <ingest|query|lint> | <title>
- op: ingest|query|lint
- source: repo:...        (ingest)   |  question: "..."  (query)  |  scope: ...  (lint)
- pages: +created  ~modified  -removed
- notes: ...
```

Greppable: `^## \[(\d{4}-\d{2}-\d{2})\] (ingest|query|lint) \| (.+)$`.

## 10. Workflow — INGEST

> The current multi-phase campaign that fills the scaffold (full
> 4.0.3→4.2 spec diff → Rust code → OBSERVATIONS → synthesis) is
> specified in **`.bootstrap/INGEST-PLAN.md`** — read it before
> running an Ingest session; resume from `index.md` status + `log.md`.

1. **Classify the source**: spec erratum/new edition · new
   `OBSERVATIONS` O-N · dogfood/parity finding (`laterite-ags4-corpus-qa`) ·
   `ags_dictionary.json` change · external doc (→ drop immutably in
   `sources/external/`, register a `sources/` stub).
2. Read the source **in full** (never skim a region you will cite).
3. **Discuss** the proposed page set with the user before writing.
4. Create/refresh pages from `templates/` — set `status: drafted`,
   add `repo_refs`, obey the Cardinal Rule (link, never paste).
5. Update **bidirectional** `related`/`## Related` on every touched
   page *and its neighbours*; refresh the relevant mermaid &
   `## Variations`.
6. Regenerate `index.md` in full; append one `log.md` entry.

## 11. Workflow — QUERY

Answer from the wiki first (follow `[[links]]`, cite `repo_refs`). If
the wiki is insufficient, either answer with an explicit gap caveat or
run an Ingest of the underlying authority. **Durable syntheses are
filed back** to `comparisons/` (set `origin_query`) so explorations
compound. Always append a `query` `log.md` line (pages: none if
nothing created).


## 12. Gap capture & propose-O-N (campaign sub-protocol)

When any Ingest phase finds a gap (spec ambiguity/contradiction, a
cross-edition regression, spec↔Rust, Rust↔python, or a 4.2
rule-weakness):

1. Create an `insights/<slug>.md` from `_template-insight.md`. Set
   `gap_kind`, `severity`, `editions_affected`, `rules`,
   `discovered_phase`, ≥1 `spec:`/`repo:` citation.
2. **Grounding rigor**: a Rust↔python or spec↔Rust gap is
   `status: hypothesis` until **empirically probed** — a crafted
   fixture (NEVER `laterite-ags4-validator/tests/fixtures/`, which is
   the validator's own contract) run through *both* `lat` and the
   python-ags4 oracle, output recorded as `evidence` → then
   `confirmed`. Spec-only gaps are graded by citation strength,
   `> [!spec-ambiguity]`. **The probe harness is not in this repo**:
   `.bootstrap/probes/` and `tools/py_ags4_check_json.py` are dev-satellite
   material, so without it, reach the oracle the way this repo can —
   `./tools/run_python_ags4_tests.sh` (python-ags4's own suite against
   `laterite.compat`) or `laterite.compat` directly — and say in the
   `evidence` which route produced the output.
3. If it warrants an OBSERVATIONS entry, set
   `proposes_observation: true` and fill the drafted `### O-NN …`
   block. The agent then **writes it into `observations.json`** (repo
   root) — that file is the canonical authority — and **regenerates**
   with `uv run --no-sync python tools/gen_observations.py`. Edit it
   *deliberately*: match the 5-field house style (observed/where ·
   spec · assessment · upstream-reportable · our decision), use the
   next free `O-N`, keep it clean-room (cite, never paste). Add the
   matching `ags-wiki/observations/O-NN.md` page from
   `templates/_template-observation.md` — it links and cross-references
   the record but never copies its fields. Flip the insight to
   `status: ratified` and surface the new/changed O-N in the response
   so the maintainer sees the authority change.

   > [!warning] **Never hand-edit `OBSERVATIONS.md`.**
   > It is a *generated view* of `observations.json`, as is the wiki's
   > coverage-map list. `gen_observations.py --check` gates both on
   > `ci.yml` and `nightly.yml`, and `--check-wiki` holds the JSON and
   > the `O-NN.md` pages in agreement — so a hand-edit is a red PR, not
   > a shortcut. (Earlier campaign text said "the agent never writes
   > `OBSERVATIONS.md`; the user ratifies" — that was a self-imposed
   > guardrail wrongly attributed to the user. Deliberate, visible
   > authority edits are fine; they just go through the JSON.)
4. Wire `feeds_strategy` so the gap flows into the test strategy.
   (Its twin `feeds_ags5_req` fed the AGS5 requirement register, retired
   with that class in #500, and was removed with the register's last
   reader — see [[reliquary]].)

## 13. Status lifecycle

`stub` (template only, no prose — bootstrap state) → `drafted`
(content written by Ingest) → `reviewed` (verified) ; Lint may set
`stale` (a cited source moved) or `contradicted` (conflicts another
page/source); the next Ingest clears them.


## 14. Decisions log

- AGS3/earlier **out of scope** (only 4.0.3–4.2). O-30 keeps its page
  (it documents the validator *refusing* AGS3 — an AGS4-tooling
  insight) but there is no AGS3 *edition* page.
- Spec PDFs are held once, never duplicated — and **not redistributed in this
  public repo** (AGS's own copyright); `sources/spec-*` says where to get them.
  `reports/AGS 4_1.pdf` & `reports/AGS 4_2.pdf` were deleted; the
  `AGSL4_2_*.xlsx` stay in `reports/` and are cited there.
- Sub-rules get their own pages; `nDP/nSF/nSCI` are one parametric
  page each.
- New decisions appended here as the schema co-evolves.
