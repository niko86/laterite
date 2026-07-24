---
type: concept
title: "perf campaign: the strategy, the stopping rule and the ledger"
status: drafted
tags: [concept, performance, process, register]
volatile: [timings, status]
volatile_asof: 2026-07-24
ags_editions: []
repo_refs:
  benches: "repo:rust-packages/laterite-ags4-validator/benches/validate.rs"
  fixtures: "repo:tools/gen-bench-fixtures.sh"
  readme_bench: "repo:tools/bench-vs-python-ags4.py"
related: [core-perf-baseline, abi3-perf, testing-strategy, crate-map, reliquary]
sources: []
---
# perf campaign: the strategy, the stopping rule and the ledger

## Definition

The **living work-list** for performance: how we choose what to profile next,
when to stop, what has been measured, what is claimed, what is
priced-and-declined, and what is still an unexamined guess.
[[core-perf-baseline]] records the *findings*; this page records the *campaign* —
so the next session can pick up the list rather than re-derive it, and so
"we profiled things" never again means "we profiled whatever was in front of us".

## Why it matters

Three separate times, this repo recorded a mechanism that was **wrong** while its
measurement was right:

- `index_ags4_bytes` "costs as much as a full parse — unexplained". It *was* a
  full parse.
- The `AutoFix` premium "is entirely validate-and-fix". It was mostly a duplicate
  parse of the same bytes.
- `read_ags4_bytes`' +67% is "the price of byte-fidelity". It was a per-row
  `HashMap` and 8.4M cloned heading names.

Each was plausible, written in good faith, and never checked against the code. A
stopwatch tells you *where*; only reading tells you *why*. **An unverified
attribution is a guess with a number attached** — this ledger marks them as such.

## The rules of engagement

1. **One candidate at a time.** Measure, change, re-measure, land. A batch of
   changes with one benchmark run cannot attribute its own result.
2. **Paired comparison is the number.** Absolute figures drift ~20% run-to-run on
   a large fixture (page cache). Quote criterion's before/after, not milliseconds.
3. **`--quick` is for triage, never for a claim.** It has reported a +10%
   regression that proper sampling showed to be a −6% improvement. Inconsistent
   signs across rungs mean noise.
4. **Read the code before believing the profile.** See above.
5. **A claim needs a committed bench.** If nothing in the repo reproduces it, it
   is folklore. This applies to wiki numbers and README numbers equally.
6. **Cover what you touch.** A perf change that moves a branch into a hot path
   without a test is a latent bug with better timing. If the area has a coverage
   gap, close it *in the same change* — see the coverage column below.

## Order of attack

**Rust core first**, until it is as fast as is reasonably practicable — the
surfaces (wheel, node, wasm, CLI) all sit on it, so a percent there is a percent
everywhere, and optimising a surface over a slow core measures the wrong thing.
Only then move outward.

The layering to work down:

```
parse leaf  →  rules engine / read codec / index  →  emit  →  surfaces
```

## The strategy

The campaign is not "optimise what is in front of us". It is a queue, worked top
down, with an explicit condition for putting the tools away.

**Ranking function.** Every candidate is scored `(prize × confidence) / cost`.

- **prize** — the *ceiling*, expressed against a measured stage in
  [[core-perf-baseline]]. A candidate inside `rule-family/groups` cannot be worth
  more than 8.5 ms, whatever its allocation count looks like. Quoting the
  enclosing stage's measured time is how a static allocation census stops
  masquerading as a timing.
- **confidence** — has the mechanism been read in the code (`file:line`), and is
  the enclosing stage on the baseline table? A verified mechanism inside an
  *unmeasured* stage is medium confidence at best: we know the work is redundant,
  not what removing it buys.
- **cost** — `trivial` (a few lines, one file, no semantics decision) ·
  `contained` (one crate, no public type changes, mechanical call-site updates) ·
  `invasive` (changes a public type, a signature crossing a surface boundary, or
  a behaviour four callers depend on).

Working rules that fall out of it:

1. **Big-cheap first, invasive last, and never both unknown at once.** An
   invasive change must be the top of the queue *and* have its prize already
   measured. We do not spend an invasive change to find out whether it was worth
   it.
2. **Free riders, not sweeps.** A `trivial` fix in a file already open for a
   ranked candidate rides along in the same PR — recorded as "taken, below
   measurement resolution", never claimed with a number. It does not earn its own
   session, and it does not license a sweep of the crate.
3. **One candidate, one paired run, one commit.** A tranche is one session and
   one PR, but each candidate inside it lands as its own commit with its own
   criterion before/after.
4. **A candidate whose stage is not on the baseline table is not rankable.** It
   goes to a bench-writing tranche first.
5. **Declining is a recorded result.** A candidate ruled out by the stopping rule
   goes into the ledger *with its ceiling*, so the next session does not
   rediscover it and re-derive the same arithmetic.

## The stopping rule

The purpose of a threshold is to make "we are done here" a checkable statement
rather than a mood. Three thresholds, each anchored to a number already in
[[core-perf-baseline]].

### The candidate floor: 5% of its enclosing stage

**Abandon any candidate whose measured or ceiling-bounded prize is below 5% of
the stage it sits in**, on a paired criterion comparison at the 25 MB rung.

Why 5, and not 2 or 10:

- **Below it, the claim cannot be reproduced.** Absolute figures drift ~20%
  run-to-run on a 25 MB fixture, and this page records a concrete instance:
  *identical* `index_ags4_bytes` code measured 56.9 ms, then 47.0 ms — a 17%
  swing with no code change. A 3% win exists only inside one machine's criterion
  baseline directory, which makes rule of engagement 5 unsatisfiable in practice.
- **Above it, nothing has ever been excluded.** Every item in the Claimed table
  is ≥16.7% (`parse_bytes` −16.7% is the smallest). The campaign has **no**
  landed item in the 5–17% band, so a 5% floor has never yet cost us a win.
- **It does real work immediately.** 5% of `check_parsed` (147.6 ms) is 7.4 ms.
  That rules out, by measurement rather than opinion, six of the eight rule
  families: `groups` 8.5 + `typed_values` 5.1 + `references` 0.46 + `structure`
  0.43 + `dictionary` 0.11 + `naming` 0.01 = **14.6 ms, 9.9% of the whole rules
  engine, in aggregate**. No individual candidate inside any of them can clear
  the floor. Those bands are closed.

### The tranche floor: 10% of the headline stage

**When a whole tranche lands and the stage it targeted has not moved 10%, that
band is finished** — move down the layering, and when the layering runs out, move
to the surfaces.

10% is half the observed run-to-run drift band, so a tranche returning less
cannot be demonstrated on a user's machine without a same-session paired run — it
will never appear in the README tables and never survive a machine change.

### The invasive gate: 20%, and top of the queue

**An `invasive` candidate must be measured at ≥20% of its stage before it is
opened.** The ledger already contains the revealed preference: the positional row
model was priced at **13% of the typed read** and deliberately declined on churn
grounds. 13% at invasive cost is demonstrably below this repo's bar, so 20% gives
a clear margin over the thing already refused rather than re-litigating the same
number under a new name.

### The structural stop for the validator band — already nearly reached

A sharper signal is available for the rules engine specifically:

> **Stop the validator band when `check_parsed` ≤ `parse_bytes` at the same
> rung.** When the rules cost less than the walk that feeds them, further rules
> work is optimising the smaller half of `check_file`.

At baseline that ratio was 343.2 / 171.3 = **2.0×**. It is now 147.6 / 142.8 =
**1.03×**. The validator band has essentially arrived. It gets **one** more
tranche (T1) and is then closed regardless of what static allocation counts
remain in it.

> [!note] An arithmetic flag for the next paired run, not a finding. At baseline
> `check_file` ≈ `parse_bytes` + `check_parsed` to within ~8 ms. Post-fix the
> same sum is 142.8 + 147.6 = 290.4 against `check_file`'s 328 ms — a residual of
> ~38 ms rather than ~8 ms. The ~20% drift warning covers a gap that size, so
> re-derive it on one machine in one session before attributing it to the I/O or
> dictionary-resolution leg. Attributing it now would be the exact failure this
> page exists to prevent.

### The exemption: correctness is never subject to the stopping rule

A candidate is a **correctness** candidate when the current code returns a wrong
answer, silently drops data, or has no test exercising a path a user can reach.
Those are **not queued, not ranked, and not gated by any threshold**. They land
at the point of discovery, as their own PR, however small the perf prize is.

The corollary matters too: **a measurement gap is not subject to the stopping
rule either**, because you cannot decline what you have never timed. It is
subject to a different test — a bench that costs a session to write must
plausibly cover a stage that is more than 10% of a user-facing operation.

## The ledger

Status vocabulary: **open** (unmeasured), **priced** (measured, not yet acted
on), **claimed** (landed), **declined** (measured and deliberately not taken —
with the condition for revisiting).

### Claimed

| candidate | mechanism | result | coverage |
|---|---|---|---|
| rules 10a/10c | loop-invariant column indices re-derived per row; cells cloned to hash | relational −55% | existing rule tests |
| rule 3 | whole line tokenised to read field 0 | line_format −66% | `display_spans.rs` proptest added |
| `index_ags4_bytes` | ran a full parse, kept ~123 records | −66% | `locate_only.rs` added (equivalence + rejection parity) |
| DATA cell double-allocation | `fields[1..].to_vec()` cloned every tokenizer `String` | `parse_bytes` −17% | existing walk tests |
| `AutoFix` duplicate parse | `validate()` parsed, dropped the parse, then re-parsed the same bytes | autofix −49% | existing emit tests |
| `tokenize_spans` retirement | third implementation of the line grammar; `AgsSpan.text` shipped derivable data across the wasm boundary | 511.5 → 147.3 ns/line | TS tiling + contiguity tests added — **caught a real conversion bug** |
| `from_shared` row projection | per-row `HashMap` + heading name cloned per cell + values cloned from a parse about to be dropped | `read_ags4_bytes` −27%, projection −74% | existing `from_shared_trim.rs` pins semantics |
| **T1** relational parent-tuple memoise | parent KEY-tuple `HashSet` rebuilt per child; memoised by parent code | `rule-family/relational` **−13.9%** | two-children-of-one-parent cache-reuse test added |
| **T1** line_format byte scans | three per-line `chars()` walks (rule_1 max_cp, rule_6 CR/LF, check_quoting) → byte scans | `rule-family/line_format` **−48.5%**, `check_parsed` **−11.4%** combined | multi-byte-before-CR `char_span` test added |

> [!note] **The validator band is closed** (T1, 2026-07-24). The structural stop
> fired: `check_parsed` / `parse_bytes` = 120.1 / 142.8 = **0.84×**, so the rules
> engine now costs *less* than the parse that feeds it. Further rules work
> optimises the smaller half; the queue's remaining validator rows below are
> retired unmeasured, recorded here so they are not rediscovered.

### Priced, declined

| candidate | prize | why not | revisit when |
|---|---|---|---|
| positional row model (`Vec<Vec<String>>` + heading index) | ~25 ms, 13% of the typed read | breaks `r["LOCA_ID"]` at every call site — `lat read`, `laterite-excel`, node, `read_groups_raw` | a caller reads these rows in a hot loop, **or** `AgsGroup` is being reshaped anyway. Not on the 13% alone. |

### The queue — ranked, big-cheap first

Every row below has been read in the code and then adversarially re-read by a
second pass. Prize cells quote the **measured ceiling** from
[[core-perf-baseline]] wherever one exists; static allocation counts are marked
as such, because they are counts, not timings.

| # | candidate | band | frequency | prize (ceiling) | cost | bench | coverage gap to close with it |
|---|---|---|---|---|---|---|---|
| ~~1~~ | ~~parent KEY-tuple set rebuilt per child~~ | validator | per-row | **LANDED T1 — relational −13.9%** | contained | yes | cache-reuse test added |
| ~~2~~ | ~~`build_column` casts per cell via `parse_value` (courier `String`, `canonical_type` re-resolved per cell)~~ | laterite-types | per-cell | **LANDED T3 — typed_read_file −73.7% (75.9→19.9 ms)** via bulk Arrow cast | contained | yes | `typed_build_parity.rs` pins `build_column` ≡ `parse_value` |
| ~~3~~ | ~~`line_format`'s three per-line `chars()` walks~~ | validator | per-line | **LANDED T1 — line_format −48.5%** | contained | yes | `char_span` test added |
| 4 | `raw_lines` pushes one owned `String` + full copy per line on the **default** validating profile (`parse/lib.rs:721`, `text.into_owned()`) | parse leaf | per-line | **medium** — inside `parse_bytes`, 142.8 ms, the largest stage of `check_file` (328 ms) | contained | yes | `RawLine.text` is `pub` at ~4 production + ~5 test sites; the lossy-replacement branch must keep its *decoded* text |
| 5 | `Sidecar::assemble` runs a second full walk inside `mint`, over bytes `mint` has already parsed, plus a third full-buffer hash pass | core + trust | per-file | **medium** — `index_ags4_bytes` is 56.9 ms; `mint`'s total is **unmeasured** (no `benches/` in `laterite-ags4-trust`) | contained | partial | no test mints a file whose declared encoding is not UTF-8 |
| 6 | node's `table_ipc` has no `with_keys=false` escape, so the keychain pass runs on the default `table(code)` call and the keys are then stripped | surfaces (node) | per-row | **large** — Python's twin documents this pass as "the dominant read cost"; **unmeasured on node** | contained | **no** — node has no bench of any kind | no test asserts `.sql()`/`.at()` after a prior plain `table()` on the same group |
| 7 | `parse_compat_arrow` builds a `RecordBatch` for every group even when `only_groups` narrows | surfaces (compat) | per-group | **medium** — unmeasured; `bench-vs-python-ags4.py` always calls the full read, so it is structurally blind to this | contained | no | the strict-check metadata must keep running for *all* groups |
| 8 | the GIL is never released anywhere in `laterite-py` (zero `allow_threads`/`detach`) | surfaces (wheel) | not hot — concurrency only | **unknown** — invisible to all five criterion benches by construction and to `test_perf_read.py` (single-threaded) | contained | **no** — needs a new *kind* of bench | no test anywhere exercises concurrent access to the wheel |
| 9 | `EmitGroup` owns `Vec<Vec<String>>`, so two callers deep-clone an already-owned matrix (`emit.rs:188`, node `lib.rs:243`) | emit + node | per-cell | **small** — `emit_ags4/report` is 22.4 ms and the writer is 2.9 ms of it | **invasive** — changes a public field type | partial | node and excel have no bench crate |

Below the floor, measured out — recorded so they are not rediscovered:

| candidate | ceiling |
|---|---|
| `rule_16`'s `BTreeSet<&str>` insert per PA cell | capped by `groups` at 8.5 ms (5.8% of `check_parsed`); iteration order is output-visible |
| *free rider inside #1:* `fields_with_status` allocates two `String`s per heading visit | a fixed per-group cost beside per-row work |
| *free rider:* `in_pandas_range` re-parses two constant date strings per DT cell | `typed_values` is 5.1 ms (3.5%); the fixture is 0.09% DT cells, so the bench is blind to it |
| *free rider:* `references::check` builds a fresh `HashSet<String>` per borrowed heading | the whole `references` family is 0.46 ms |

> [!note] These free riders were **not** taken in T1 — the two ranked candidates
> alone cleared the tranche floor and closed the band, and a sweep of trivial
> untimed edits is exactly what the "free riders, not sweeps" rule forbids once
> there is no longer a ranked candidate opening the file. They retire with the
> band; reopen only if a future change is in these functions for another reason.

### Correctness and measurement — outside the ranking, not subject to the floor

| id | item | status | why it is not a perf row |
|---|---|---|---|
| **C1** | `from_shared` collapsed duplicate headings (`ags4_codec.rs`) | **LANDED** (#88) | wrong data, not merely lost data — see below |
| **C2** | `arrow_in.rs` had zero tests and compiled under no CI test job | **LANDED** (#87) | untested code shipping inside the wheel and the node binding |
| **M1** | `laterite-types/arrow` + `laterite-ags4-emit/arrow` (651 lines) were compiled out of the coverage build by resolver-2 feature unification once their only four enablers are excluded | **LANDED** (#87) | the 88% gate was not measuring the Arrow boundary at all |
| **M2** | the bench fixture is **clean by construction**, so `findings::add`, rule 10b's per-bad-row `format!`/`join`, rule 11c's O(child × target) scan and the entire FYI tier are never executed by any bench | open — T5 | unknown by construction: validating a file *with errors* is an unmeasured workload |

**C1, as measured rather than as queued.** The row above originally read
"silently collapses duplicate headings". That understated it. Rows are keyed by
heading name, and every consumer walks `headings` *positionally* then indexes the
row *by name* (`laterite-excel:144`, node `lib.rs:117`, wasm `lib.rs:1198`,
`read_groups_raw`) — so the survivor was returned for **both** positions. Built
against the pre-fix commit, a `LOCA` with headings `LOCA_ID, LOCA_GL, LOCA_ID`
and values `FIRST, 1.00, SECOND` read back as:

```
['SECOND', '1.00', 'SECOND']
```

`FIRST` gone **and** `SECOND` duplicated into its column, with no error. A column
that looks fully populated and is wrong is a worse failure than a missing one,
because nothing downstream can detect it. Now fatal by default, with an opt-in
`__2` suffix recovery on every read surface.

> [!warning] **C2 was overstated when queued, and this is the correction.** The
> row read "a silent `Value::Null` on a shipped public entry point". Probing
> arrow 59 across union, run-end, dictionary, struct, map, interval, duration,
> binary, fixed-size-binary and time64, `ArrayFormatter::try_new` does not fail
> for **any** of them — the `Err` arm is defensive, not live. The real defect was
> only ever that none of the file compiled in CI. A test now pins the fallback so
> it cannot start swallowing real cells without going red first.
>
> That is the **fourth** time this campaign has recorded a mechanism that was
> wrong while its framing was right. The framing earned the fix; the mechanism
> did not survive being read. Both halves belong in the ledger.

> [!note] **M1 cost less than budgeted.** The decision accepted that the headline
> coverage number would drop when 651 lines entered the denominator. Measured
> over the identical exclude set it came in at **89.24%** against the previously
> documented 89.6% — 0.36 points — because `arrow_cols.rs` (88%) and `ipc.rs`
> (100%) were already well covered by their own tests and merely invisible to the
> measurement. The floor stayed at 88; no re-baseline was needed.

## Tranches

Each is one session and one PR. Each candidate inside a tranche lands as its own
commit with its own paired criterion run. Bench-writing tranches come **before**
the optimisations that need them.

### T1 — Close the validator band — ✅ DONE (2026-07-24)

**Candidates:** #1, #3 (the free riders were left untaken — see the note above).
**Bench first:** already committed (`validate/rule-family`, `validate/check_parsed`).

**Landed:** relational **−13.9%**, line_format **−48.5%**, `check_parsed`
**−11.4%** combined — clearing the 10% tranche floor. The structural stop fired
at **0.84×** (`check_parsed` / `parse_bytes`). Band closed. Parity held by
identity. Coverage: the cache-reuse and `char_span` tests below both landed.

> [!warning] #1 is **not** the landed 10a/10c fix. What landed was the
> *column-index hoist* (`cols()`, whose doc comment names exactly what it
> removed). Untouched by that hoist are the per-child-group rebuild of
> `parent_tuples` and the double `tuple_at` per row — a different mechanism in
> the same function. Read `relational.rs:438-450` before touching it.

**Coverage closed:** a multi-byte-character-before-the-CR test pinning
`char_span` (the CR's char offset, which a byte offset would have got wrong), and
a two-children-of-one-parent test proving the memoised set is applied to the
right child. Both landed.

### T2 — Put the typed read on the baseline axis (bench only) — ✅ DONE (2026-07-24)

**Landed:** `types/typed_read_file/large` = **75.9 ms** (312 MiB/s) — the typed
build over the whole fixture's real cells, now on the baseline table. The
mixed-group bench prices the typing directly: typed **19.4 ms** vs raw-string
compat **0.88 ms** (~22×), so ~95% of the typed build is casting and the String
arm (60.3% of headings) is its bulk. All five `build_column` arms are now benched
(`0DP`/`YN` added), plus a `null-half` rung and the three sibling builders
(`compat`, `with-ids`, `ipc`). Coverage: two value-correctness tests now assert
`build_column`'s decoded array *contents* per arm and its typed-null handling.
The typed-read stage is measured, so **candidate #2 is now rankable** and moves
to T3 with a real ceiling (a fraction of 75.9 ms) rather than a heading census.

The largest hole in the campaign: the baseline table prices `parse_bytes` →
`read_ags4_bytes` → `check_parsed` → `emit`, all in pure Rust strings. The
**typed** read — what every surface actually calls and what the README's read
tables exercise — appears nowhere on it. `read_ags4_bytes`' unattributed residual
cannot be reasoned about while the layer above it is unmeasured.

- Add fixture-fed rungs to `types/build_record_batch` at 1/10/25 MB, so its cost
  is comparable with `parse_bytes` on one axis.
- Add the two missing families: `0DP` (the only `Integer` code) and `YN` (the
  only `Bool` code) — two of `build_column`'s five arms are unbenched.
- Add a **null/empty-cell rung**: the closure returns the same non-empty `&str`
  for every row, so the `append_null` branch a sparse delivery takes constantly
  is never timed.
- Bench `build_record_batch_compat`, `build_record_batch_with_ids` and `ipc.rs`.

> [!note] **M1 no longer belongs to this tranche** — it landed early in #87,
> alongside C2, because turning the feature edge back on without C2's tests would
> have moved the gate before anything held it. The coverage denominator is
> already honest; T2 is now purely a measurement tranche.

**Exit:** a typed-read row exists on the baseline table, and #2 is rankable
against a measured stage rather than a heading census.

### T3 — The typed read's per-cell casting — ✅ DONE (2026-07-25)

**Landed:** `types/typed_read_file/large` **75.9 ms → 19.9 ms (−73.7%)**, 312 MiB/s
→ 1.16 GiB/s — a **~3.8×** speedup on the exact `build_record_batch` every typed
surface pays. Well past the 5% candidate floor and the 10% tranche floor.

**What changed (the owner's redesign, not the incremental plan).** The original
candidate #2 was "bypass `parse_value` in `build_column`'s string arm". The owner
proposed going further: **build every column as `Utf8`, then cast the numeric
columns in bulk through Arrow's `compute::cast` kernels** instead of casting
per cell. A measured spike (branch `spike/t3-arrow-cast`) proved it out, so it
**replaced** the incremental edit.

- **String / Enum / unknown (~60% of headings):** built `Utf8` directly (trim +
  empty→null), skipping the per-cell `canonical_type` dispatch entirely.
- **Integer (`0DP`):** `Utf8 → Float64 → Int64`. The double cast reproduces
  `parse_ags_integer`'s `int(float(s))` exactly — truncation toward zero and the
  #611 i64 range guard fall out for free.
- **Decimal (`nDP`/`nSF`/`nSCI`):** `Utf8 → Float64` + a bulk finite-null pass
  (Arrow's string→float admits inf/NaN; `parse_ags_decimal` nulls non-finite).
- **Bool (`YN`) / Datetime (`DT`):** unchanged custom arms — Arrow's generic cast
  can't reproduce the Y/N/YES token set or the six AGS date formats. On the
  fixture these are ~0.1% of cells, so leaving them per-cell costs nothing.

**Parity is the gate, and it holds.** `build_column` is byte-parity with the
per-cell `parse_value` build — Arrow-representation identical (`ArrayData` logical
equality, the object the C-data interface / IPC hand to polars / duckdb / arrow-js)
over **663 columns / 2,557,209 cells** of the fixture plus the edge seams, pinned
permanently in `laterite-types/tests/typed_build_parity.rs`. A live wheel diff
confirmed it end-to-end: every group's polars DataFrame (schema + dtypes + values
+ nulls) unchanged, on both the frame and `keys=True` paths.

**Surfaces inherit it free:** all typed surfaces funnel through `build_column`
(py→polars via `build_record_batch_synth`, node→arrow-js and wasm→duckdb via IPC),
so no API/feature change; `arrow-cast` is already non-optional under the `arrow`
feature. The compat/pandas builder (`build_record_batch_compat`) is a separate
all-`Utf8` path and is untouched, so python-ags4 parity is unaffected.

**Follow-up left on the table:** the Decimal finite-null pass is an `iter/collect`
scan; an Arrow `is_finite`-mask kernel could shave it further, but the win already
clears every floor, so it is not pursued now.

### T4 — The parse leaf and the redundant certify walk

**Candidates:** #4 (`raw_lines` span rewrite), then #5 (`assemble_from_parsed`).
**Bench first, in this PR, before the #5 change:** a `laterite-ags4-trust` bench
covering `mint`/`certify` end to end. The operation that pays for the walk twice
is not measured at all, so "2× parse on a 25 MB file" is architecturally correct
and currently unquantified. Write the bench, land it, take the number, *then*
change the code.
**Coverage closed:** the lossy-replacement branch for #4; a non-UTF-8 mint test
for #5 — reuse probably *fixes* that, but a semantics change must not ride in
silently as a side effect of a perf refactor.
**Exit:** `parse_bytes` moves ≥5% and the new trust bench ≥10%. If `parse_bytes`
does not move, the parse leaf is closed.

### T5 — A dirty fixture and the FYI tier (bench only)

**M2.** Every validator bench runs against a file the forge *asserts* is clean,
and all three bench functions use `CheckOptions::default()`
(`include_warnings: false, include_fyi: false`). The engine's entire
error-reporting half has never been timed: `findings::add` never executes once;
rule 10b's dirty path builds a `HashMap`, a `Vec<String>` with a `format!` per
column and a `join("|")` **per bad row**; rule 11c never runs and hides the worst
asymptotic in the engine; and `rule_16_fyi_nonstandard_abbr` linearly scans and
`split_once`s the whole 3,471-entry abbreviation table per ABBR row — on the
`compat` / `lat validate --show-fyi` path.

**Work:** a dirty rung via `forge gen --scaffold wide --combine <injectors>` (the
CLI already supports this), plus a rung with both tier gates on. Note the ceiling
honestly: `gen` is unscaled, so this tops out at a handful of findings. A
size-scaled densely-dirty fixture needs a fault-density mode in `forge
scale`/`calibrate` that does not exist — scope that separately.
**Exit:** the rungs are committed and produce numbers. **No optimisation lands in
this tranche.** Whatever they show re-enters the queue at the same 5% floor.

### T6 — The surfaces (only once the core stopping rule has fired)

**Candidates:** #6, #7, #8. **Benches first: all three.** Node has no harness of
any kind; compat is exercised only by `bench-vs-python-ags4.py`, which always
calls the full read and so cannot see #7; #8 needs a *new kind* of bench — a
Python-level multi-threaded throughput test comparing wall-clock at N=1 against
N=core-count. A held-GIL implementation shows flat N-times-serial time; a
released one shows near-linear speedup. Single-threaded wall-clock, which is all
`test_perf_read.py` measures, is identical either way.

**Coverage closed:** for #6, a test asserting `.sql()`/`.at()` correctness *after*
a prior plain `table()` on the same group — Python's read path is deliberately
structured to avoid exactly this trap, and node's single-cache-per-code
architecture has no such split, so a naive `with_keys` bolt-on breaks the
relational path silently.
**Exit:** the tranche floor applies per surface, not to the group. #9 is
deliberately **not** here: it is invasive and its stage is 22.4 ms, so it cannot
clear the 20% gate. It reopens only if node gets a bench and node's emit turns
out to dominate there.

### Refuted — do not chase

> [!warning] **Ledger correction.** The old Open row reading
> "`laterite-types::arrow_cols` … never benched in isolation" was **wrong**.
> `rust-packages/laterite-types/benches/arrow_cols.rs` benches
> `build_record_batch` **per type family and mixed**. What is genuinely unbenched
> is narrower: no file rung; the `Integer`/`Bool` arms; the null/empty branch;
> and `build_record_batch_compat`, `build_record_batch_with_ids` and `ipc.rs`.
> T2 closes exactly those four.

- **"`from_shared` makes a second heap copy of every DATA cell."** Already landed
  in `d3d3867` — it takes `ParsedFile` **by value** and moves every
  heading/unit/type/value, with `Arc<str>` row keys allocated once per group. The
  cited lines are the *pre-commit* code, visible as the `-` side of that diff.
- **"~793 ns per finding construction."** `rg -n "793"` across the repo returns
  nothing. The figure has no source in code, bench output or wiki. **Invented.**
  The *mechanism* (the finding path is unbenched) is real and is M2; the number
  is not.
- **`laterite-py`'s two emit sites as `EmitGroup` double-copies.** Miscited —
  neither touches `EmitGroup`; both hand a matrix to `write_ags4_matrix`, which
  already takes it **by reference**. The wheel's emit surfaces never had this bug.
- **`laterite-excel` as "copies every cell twice".** One clone, not two, and
  untouched by the #9 fix; removing it needs a different row representation.
- **`encoding_rs`' decode as the source of the per-line allocation.**
  `decode_without_bom_handling` returns `Cow::Borrowed` with zero allocation for
  valid UTF-8 — the common case. The malloc is 100% the deliberate `.into_owned()`.
- **`desc.into()`/`group.to_string()` as unconditional allocations in
  `findings::add`.** Several call sites already pass an owned `String`, and
  `"".to_string()` allocates nothing. The one that genuinely fires every call is
  `rule.to_string()`, because `BTreeMap::entry` needs an owned key.
- **`field_span` as the cheap way to read field 0.** Its contract is "the field
  at `field_index + 1`", so `field_span(line, 0)` returns the field *after* the
  descriptor, and it returns **char** offsets. `scan::first_field` is the answer.

## Decisions taken

Recorded here so the next session does not re-open them. Each was a genuine fork
with consequences both ways; none is a default that fell out of the code.

**Sequencing — correctness and measurement debt first, then T1.** C2 and M1 land
before the validator tranche. The reasoning is the exemption above: untested code
that returns a silent wrong answer is not competing with a perf candidate for the
same slot, and both are small. The accepted consequence is that the headline
coverage number **drops** when the Arrow boundary enters the denominator — an
honest lower number beats a flattering wrong one, and unlike the QA-crate
exclusions this one was never a decision in the first place.

**Item 4b — UNIT-first, then fall back.** `parse_datetime` consults the format
the `UNIT` row names; if the cell does not match, it falls back to the existing
table. Decided on **correctness, not speed** — `typed_values` is 5.1 ms (3.5% of
`check_parsed`), below the candidate floor, and the fixture is 0.09% `DT` cells,
so the perf case is being argued from a number nobody has. The correctness prize
is real and does not need strictness to collect it: consulting `UNIT` first
resolves the ambiguous slash-separated date by what the file says about itself
rather than by table order (`%Y/%m/%d` is currently tried before `%d/%m/%Y`).
Strict UNIT was rejected because it would null cells that parse today wherever a
real delivery's `UNIT` is wrong, absent or free text — a read-path behaviour
change with parity consequences against python-ags4.

**C1 — duplicate headings fail the read by default, with an opt-in recovery
flag.** The defect is worse than a lost column: consumers iterate `headings`
positionally and index the row map by name, so the surviving duplicate is
returned *for both positions*. The first column's data is gone **and** the
second's is duplicated into its place — the column still looks populated, with
wrong values. That is why "keep first and warn" was rejected: it leaves the
positional defect live by default on four surfaces that never run the rule
engine.

- **Default: fail the read**, naming the flag in the error. Files that succeed
  today will start failing; they were producing wrong data, not right data.
- **Opt in** to disambiguate rather than collide: the second and subsequent
  occurrences are suffixed `__2`, `__3`, … in **both** `headings` and the row
  key, which restores correct positional reads and loses nothing.
- **On every read surface** — `lat read`, `laterite-excel`, node,
  `read_groups_raw` and the Python read. Not because each needs it equally, but
  because a read option present on some surfaces and absent on others is exactly
  the drift `laterite-ags4-xcheck` and the drop-in surface gate exist to catch;
  partial wiring fails CI rather than quietly diverging.

> [!note] The output is deliberately **not valid AGS4** — a suffixed heading is
> not a spec heading. That is the accepted trade: the flag exists to recover data
> from a broken file, not to round-trip it. The exposure is narrow because
> `AgsGroup` is a read-side projection and the emit path builds from frames and
> `ParsedFile`, so a suffixed read cannot corrupt validate-and-fix. The one real
> case is a suffixed XLSX converted back to AGS4, which carries the suffixes —
> and only for someone who opted in.

**Node gets a bench harness.** Building it is a session and some CI cost, but the
alternative positions are both worse: node carries the clearest per-row waste in
the workspace (#6) with no way to price or defend a fix, and declaring the
surface out of scope would leave a shipped binding permanently unmeasured. This
is the T6 prerequisite, so #6 stays queued until the harness exists.

## Coverage discipline

Perf work moves branches into and out of hot paths, which is precisely when an
untested edge becomes a silent wrong answer. Two live examples:

- the `locate_only` profile skips the descriptor model but **must not** skip the
  strict-structure guard — pinned only because the equivalence test asks;
- the wasm byte→code-point conversion was queried out of order and collapsed
  every inner value to empty. The `debug_assert` guarding it is compiled out of
  release wasm; the **TS-side test** is what caught it.

So: when a candidate touches an area with a coverage gap, close the gap in the
same change, and record it in the ledger's coverage column. Nightly coverage
excludes the QA crates (see `.github/workflows/nightly.yml`) — a denominator
choice, not a regression, and worth re-checking before reading any coverage
delta as real.

## Where the benches live

Beside the crate they measure. `tools/gen-bench-fixtures.sh` synthesises the
criterion rungs (1/10/25 MB, forge `wide` scaffold, seed 0 — byte-identical for a
given size+seed, and carrying no real delivery data).
`tools/bench-vs-python-ags4.py` reproduces the README's comparison tables, with
the fixtures SHA-pinned so generator drift fails loudly instead of quietly moving
the numbers.

> [!note] An absent fixture SKIPS rather than fails, so a clean checkout still
> works — but a skipped bench measures nothing, which is exactly how the
> pre-2026-07 bench sat silently dead on every machine but one.

## Related

[[core-perf-baseline]] — the findings this campaign produced.
[[abi3-perf]] — the binding's cost, measured separately.

[[core-perf-baseline]] · [[abi3-perf]] · [[testing-strategy]] · [[crate-map]] · [[reliquary]]
