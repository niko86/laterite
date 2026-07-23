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
gated by `tests/test_reference_groups_faithful.py`; the AGS4 union, D6)
plus 3 hand-authored AGS-L draft pages — alongside ~28 rules+subrules,
~18 types, ~45 observations, ~21 tools (see `index.md` for exact live
counts). The `index.md` catalog is sufficient navigation — **no
embeddings/RAG/qmd** unless the vault outgrows that (future option, not
now).

## 1. Cardinal Rule — LINK, DON'T DUPLICATE

The **repo is the source of truth**. Pages *synthesize, diagram, and
cross-reference*; they never paste source text from
`ags5_dictionary.json`, `OBSERVATIONS.md`, the rule `*.rs` files, or
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
   `ags5_dictionary.json` model authority* (not the 145-page spec
   prose), cited `repo:…ags5_dictionary.json groups[code=…]`.
Everything else stays link-only — never the spec PDF's prose sections,
worked examples, suggested-unit tables, or descriptive narrative. If a page and
its cited source ever disagree → **flag it** (`status: contradicted` +
a `> [!spec-ambiguity]`/`> [!divergence]` callout), never silently
"correct" either side.

**Citation grammar** (inline code span):
- repo file/line — `` `repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs:352` ``
- repo symbol — `` `repo:.../rules/mod.rs::run_all` ``
- OBSERVATIONS entry — `` `repo:OBSERVATIONS.md#o-33` ``
- dictionary entry — `` `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=SAMP]` ``
- spec PDF — `` `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 8` `` (PDF lives at this vault root)
- AGS library xlsx — `` `repo:reports/AGSL4_2_TRI.xlsx` ``

Paths are repo-root-relative, forward-slash, no backslashes.

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
| tool | `<repo-artifact-name>.md` | `laterite-ags4-check.md` |
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
(Rust↔python test-probe register) · `decision` / `experiment` /
`requirement` (the `design/` strand). These are *campaign-
authored*, not bootstrap stubs — they progress past `stub` (see §13).

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
- `requirement.status` ∈ `proposed | accepted | prototyped | validated | rejected`; `priority` ∈ `must | should | could`
- `experiment.outcome` ∈ `worked | partial | failed`; `decision.status` ∈ `proposed | accepted | superseded | rejected`
- `tool.status` also allows `superseded` — a retired-package page kept as a redirect-stub tombstone (the same terminal word `decision` uses)
- `superseded_by` — **required on any `status: superseded` page** (D4): the successor page stem(s) (`[a, b]` or a bare `a`), each of which must resolve; the page must *also* carry a read-time tombstone callout (`> [!…]` naming the supersession). A superseded page missing either is invalid; these pages get a dedicated **Retired / Superseded** index section.
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
   `ags5_dictionary.json` change · external doc (→ drop immutably in
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
   fixture under `.bootstrap/probes/` (NEVER
   `laterite-ags4-validator/tests/fixtures/`) run through *both*
   `lat` and `tools/py_ags4_check_json.py`, output recorded as
   `evidence` → then `confirmed`. Spec-only gaps are graded by
   citation strength, `> [!spec-ambiguity]`.
3. If it warrants an OBSERVATIONS entry, set
   `proposes_observation: true` and fill the drafted `### O-NN …`
   block. The agent then **writes it into
   `OBSERVATIONS.md`** directly —
   that file is the canonical authority, so edit it *deliberately*:
   match the 5-field house style (observed/where · spec · assessment
   · upstream-reportable · our decision), use the next free `O-N`,
   keep it clean-room (cite, never paste). Flip the insight to
   `status: ratified` and surface the new/changed O-N in the response
   so the maintainer sees the authority change. (Earlier campaign
   text said "the agent never writes `OBSERVATIONS.md`; the user
   ratifies" — that was a self-imposed guardrail wrongly attributed to
   the user, not a maintainer instruction. Corrected: deliberate,
   visible edits are fine; gratuitous churn of the authority is not.)
4. Wire `feeds_strategy` / `feeds_ags5_req` so the gap flows into the
   test strategy and the AGS5 requirement register.

## 13. Status lifecycle

`stub` (template only, no prose — bootstrap state) → `drafted`
(content written by Ingest) → `reviewed` (verified) ; Lint may set
`stale` (a cited source moved) or `contradicted` (conflicts another
page/source); the next Ingest clears them.


## 14. Decisions log

- AGS3/earlier **out of scope** (only 4.0.3–4.2). O-30 keeps its page
  (it documents the validator *refusing* AGS3 — an AGS4-tooling
  insight) but there is no AGS3 *edition* page.
- Spec PDFs are *moved* to this vault root (not duplicated);
  `reports/AGS 4_1.pdf` & `reports/AGS 4_2.pdf` were deleted; the
  `AGSL4_2_*.xlsx` stay in `reports/` and are cited there.
- Sub-rules get their own pages; `nDP/nSF/nSCI` are one parametric
  page each.
- New decisions appended here as the schema co-evolves.
