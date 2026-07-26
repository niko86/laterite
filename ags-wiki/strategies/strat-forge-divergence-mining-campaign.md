---
type: strategy
title: "Combinatorial divergence-mining campaign — staged forge mine matrix over time"
status: proposed
tags: [strategy, register, campaign]
targets: [rule-10a-key-uniqueness, rule-10c-parent-child, rule-08-typed-values, rule-05-quoting, rule-16-abbr-defined]
divergence_hypothesis: "n/a — a staged combinatorial campaign to DISCOVER new Rust↔python-ags4 divergences, not one hypothesis"
probe_files: [ags-wiki/.bootstrap/probes/parity-matrix.json]
expected_rust: ""
expected_python: ""
evidence: "forge mine report.json per run (combo · seed · signature · divergence_prone · verdict · python_rules)"
feeds_ags5_req: []
related: [laterite-ags4-forge, evolutionary-dogfooding, strat-parity-matrix, strat-forge-rule10a-relational, parity-model]
sources: []
---
# Combinatorial divergence-mining campaign

> [!note] A **planned, staged** campaign (not yet run). `forge mine`
> already implements the funnel: synthesize every rule-combination
> across a placement-seed sweep → subtract the python-ags4-covered
> shapes → spend the python oracle ONLY on the novel *divergence-prone*
> signatures. This page is the matrix that drives it **over time** and
> the harvest loop that feeds the per-PR divergence-lock gate
> (`packages/laterite/tests/test_parity_divergences.py`, #190).

## Sequencing
**Runs AFTER the forge injector improvements** (#169 laterite-ags4-compliance crate;
#172 synthetic DEPTH). The campaign is partly runnable today over the 9
single-injectable faults, but its *coverage* is bounded by the injector
menu — so the higher-value, wider campaign waits on those improvements to
inject more rule classes (below).

## The matrix (axes)
| axis | values | meaning |
|---|---|---|
| **k** (combination size) | 2 → 3 → 4 → … | how many faults co-occur in ONE file |
| **injectors** | the 9: `rule10a rule10c rule8 rule5 rule19 rule13 rule14 rule16 rule17` | which rule-breaks |
| **scaffold** | `loca-samp` → `wide` | which GROUPs carry them (LOCA/SAMP/GEOL/ABBR → ~50 groups) |
| **seeds** | 4 → 8 | placement variety — teases distinct signatures from one combo |

Combination counts (per scaffold, before the seed multiplier): k=2 → 36,
k=3 → 84, k=4 → 126 (`C(9,k)`). The python-oracle cost is bounded
separately by `--max-oracle` (only the divergence-prone gaps are spent on).

## The funnel (forge mine, per stage)
0. **`forge seed vendor`** — vendor python-ags4's test corpus as the
   *covered* set, so mine subtracts what their tests already cover (without
   it, every signature reads as a gap).
1. **Rust-only profile** (cheap, no python) → each gap tagged
   `divergence_prone`.
2. **Oracle pass** (`--max-oracle` capped) → dual-validate only the
   divergence-prone gaps against the real python-ags4 (the sibling clone /
   pinned 1.2.0).
3. **Harvest** → each *confirmed* divergence becomes a **new probe + an
   O-N + an entry in the divergence-lock gate** (#190). Recognised
   divergences (already an O-N) are NOT re-churned — the persistent
   confidence ledger (`forge confidence`) + the vendor corpus track
   coverage so successive runs EXPLORE new territory.

## Staged plan (cover over time)
- **Stage A** — k=2, `loca-samp`, seeds=4. 36 combos, ~25 divergence-prone
  → oracle them. The pairwise interactions.
- **Stage B** — k=3, `loca-samp`. 84 combos, oracle-capped to the
  divergence-prone subset.
- **Stage C** — k=2–3 on `wide` (~50 groups). Same faults, more group
  context (where edition/relational edges like [[O-42]] hide).
- **Stage D+** — higher k / new injectors as they land (below).

## The "improvements still" (parallel track — widens the matrix)
forge can inject 9 rules today; the rest are `not_single_injectable`
(`forge catalog`). Adding injectors is forge work (#169) and each widens
this campaign:
- **candidate injectors** (forge already flags them): `rule10b`
  (empty REQUIRED), `rule15` (undefined UNIT).
- **byte-level emitter**: Rules 1 / 2a / 3 / 6 (need >255 code points,
  LF-only lines, bad descriptors, embedded CR) — also unlocks the
  field-count class (Rule 4) behind #191's early-failure axis.
- heading-rename (9/18/19a/19b), reorder (2b), record-link (11a/b/c),
  FILE (20) — domain setup, later.

## Where it lives / how to run
- `repo:rust-packages/laterite-ags4-forge` — `laterite-ags4-forge mine --min-k K --max-k K
  --scaffold loca-samp|wide --seeds N --max-oracle M` (`--no-oracle` for
  the cheap Rust-only profile first).
- Output: `report.json` per run (`combo · seed · signature ·
  divergence_prone · verdict · python_rules`).
- Harvest target: the divergence-lock gate (#190) + a probe per confirmed
  divergence in `ags-wiki/.bootstrap/probes/`.

## Related
[[laterite-ags4-forge]] · [[evolutionary-dogfooding]] · [[strat-parity-matrix]] · [[strat-forge-rule10a-relational]] · [[parity-model]]
