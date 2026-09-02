---
type: concept
title: "perf campaign: the strategy, the stopping rule and the ledger"
status: drafted
tags: [concept, performance, process, register]
volatile: [timings, status]
volatile_asof: 2026-09-02
ags_editions: []
repo_refs:
  benches: "repo:rust-packages/laterite-ags4-validator/benches/validate.rs"
  fixtures: "repo:tools/gen-bench-fixtures.sh"
  readme_bench: "repo:tools/bench-vs-python-ags4.py"
  results: "repo:tools/perf-results/python-lane.json"
related: [core-perf-baseline, abi3-perf, testing-strategy, coverage-campaign, crate-map, reliquary]
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
   gap, close it *in the same change* — see the coverage column below, and the
   coverage sibling of this ledger, [[coverage-campaign]].
7. **A/B/A, on a quiet machine.** An end-to-end claim (anything measured
   outside criterion's own pairing — `/usr/bin/time -l`, the Python-lane
   harness, a wheel-level probe) runs baseline, candidate, baseline again in
   one sitting. The two A legs price the machine's drift *during that
   session*; a delta inside their spread is not a result, because the
   instrument cannot resolve it. Quiet machine means nothing heavy sharing
   the box — no builds, no indexers — and the harness records the starting
   load average so a noisy run indicts itself. (This discipline ran the
   #788–#790 emit ladder from the issue bodies; #821 moved it here.)
8. **Peak RSS is the cross-library instrument; dhat/tracemalloc are
   diagnosis.** Any number comparing two libraries is the **peak RSS of a
   fresh subprocess** running one end-to-end operation through each
   library's public API — the only instrument both sides pay identically.
   `dhat` (Rust) and `tracemalloc` (Python) attribute allocations *inside*
   our own code once a gap is found. These are different claims — what the
   process cost the machine, versus who allocated what — and they **never
   share a table**. The precedent is
   `laterite-ags4-emit/examples/heap_profile.rs`, whose header draws exactly
   this line; the harness is `tools/bench-vs-python-ags4.py`, which declares
   the results schema.
9. **The memory headline is ×-of-output.** Peak RSS quoted as a multiple of
   the operation's file size — size-independent, so rungs and workloads stay
   comparable. Absolute MB stays in the committed results file
   (`tools/perf-results/python-lane.json`), where re-running the harness
   updates it; a MB figure in prose goes stale the next time anything moves.
10. **The memory floors are the time floors, denominated in peak RSS.**
    Candidate ≥ 5% of the enclosing operation's peak; a tranche that has not
    moved its headline operation 10% is finished; an invasive change needs a
    measured 20%. Same thresholds, same reasoning — below the floor a claim
    cannot be told from run-to-run noise, and the invasive gate must clear
    the number the campaign has already declined once.
11. **Rungs have a memory cap, and a refusal is a recorded result.** Memory
    columns stop at the 265 MB rung (epic #820 decision 7): a run that
    pushes the machine into swap measures the pager, not the library. The
    524 MB rung is time-only. A rung a side cannot run without swapping —
    or at all — is recorded in the results file as a refusal with its
    reason, never skipped: a skip is a blind spot, a refusal is a verdict.
12. **Baseline parity is a floor, not the finish line** (epic #820 decision 5
    as amended 2026-08-31 — an owner call). A cell at ratio ≤ 1.0 has cleared
    the floor; the axis stays open while any candidate inside it clears the
    campaign's own floors (rule 10, denominated in the operation's peak RSS),
    and only a recorded diminishing-returns verdict — argued in absolute
    terms, against what the operation actually holds — closes it. The
    baseline says nothing about what the engine *could* hold; the
    attribution instruments do.
13. **The wasm lane's memory instrument is linear-memory high-water — its
    own labelled claim, never a peak-RSS column** (decided inside #824, as
    the ticket required). Process peak RSS does not transfer to a browser
    context, so the wasm lane reports `WebAssembly.Memory.buffer.byteLength`
    after one end-to-end operation in a **fresh instantiation** — wasm32
    linear memory only ever grows, so the size at exit *is* the peak, and
    the number is deterministic (byte-identical run to run, verified) and
    engine-independent: the growth a browser's wasm heap would show, not
    the harness's V8. The rejected candidate — the JS heap's own
    measurement APIs — is engine-dependent, GC-timing non-deterministic,
    and in the harness's node host does not even count wasm linear memory
    (an external `ArrayBuffer`), so the "broader" instrument would measure
    *less* of the engine, less reproducibly. The claim's scope is the
    **engine's hold only**: JS-side allocations (the input bytes, the
    returned IPC/text) are out of claim and belong to the host app. Like
    RSS vs dhat (rule 8), the claims never share a table or a column —
    results files label every wasm cell by instrument
    (`wasm-linear-memory`, key `peak_linear_memory_bytes`), and the matrix
    renderer names the instrument on every rendered row. One consequence,
    recorded so nobody "fixes" it: this instrument needs no swap watch —
    host paging cannot move a linear-memory byte count — so the wasm lane's
    refusal vocabulary is `beyond-mem-cap`/`failed` only, while the rung
    cap itself stays (it is ladder policy, rule 11, not a swap guard).

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
| **M5** the shipped pandas hop's polars intermediate (#834) | each group's Arrow table copied into polars purely to rename positional columns before DuckDB registration; now the native table's own capsule registers and the SQL projection renames | shipped-hop read peak **−0.9% (100 MB) / −0.6% (265 MB)**, read time **−3–5%** (fresh-child peak-RSS / warm A-B-A with pyarrow import-blocked — diagnosis-family instruments, never the lane's table); the ~+2.5× shipped-hop premium proved to live elsewhere — the memory queue's M5 row carries the correction | no-polars-intermediate, hostile-identifier, strict-zip and cross-hop-equality tests added; parity oracle by identity on both hops |

> [!note] **The validator band is closed** (T1, 2026-07-24). The structural stop
> fired: `check_parsed` / `parse_bytes` = 120.1 / 142.8 = **0.84×**, so the rules
> engine now costs *less* than the parse that feeds it. Further rules work
> optimises the smaller half; the queue's remaining validator rows below are
> retired unmeasured, recorded here so they are not rediscovered.

### Priced, declined

**This table is the home of record.** Each row below had a GitHub issue holding
its decision; the issues were closed on 2026-08-03 so the board stops carrying
work nobody intends to do, and the reasoning moved here where the thresholds it
was judged against also live. A row leaves this table only by its own revisit
condition being met and a fresh measurement, never by someone re-filing it.

| candidate | prize | why not | revisit when |
|---|---|---|---|
| positional row model (`Vec<Vec<String>>` + heading index) | ~25 ms, 13% of the typed read | breaks `r["LOCA_ID"]` at every call site — `lat read`, `laterite-ags4-excel`, node, `read_groups_raw` | a caller reads these rows in a hot loop, **or** `AgsGroup` is being reshaped anyway. Not on the 13% alone. |
| `raw_lines` pushes one owned `String` per line under `validating()` (`parse/lib.rs:35` `pub text: String`, allocated at `:721`) — queue #4, was issue #112 | ~9.9 ms = **6.9% of `parse_bytes`**, 1.9% of `check_file` — and that was the ceiling *for the raw-line push alone* | **invasive**: needs `ParsedFile<'a>` or whole-file-decode + span, changing the public `RawLine.text` type across `line_format`/`structure`/`fixes`/PyO3. Fails the 20% gate by ~3× | **COLLECTED 2026-08-31 as M4's rider (#838)**: the memory-axis re-pricing (rule 12) landed the span rewrite, and the time win came with it far past this row's ceiling — paired criterion `parse_bytes` −43…−47% (the per-cell `String`s were most of the parse *time* too, not just the hold). The decline was correct on the evidence it had; the "revisit when" condition fired exactly as written |
| `EmitGroup` owns `Vec<Vec<String>>`, so `emit.rs:354` deep-clones an already-owned matrix to hand the writer a *view* — queue #9, was issue #113 | **measured 2026-08-03, paired**: `emit_ags4/report` 18.155 → 16.525 ms with the clone removed = **−1.63 ms, −9.0%** (autofix −8.8%, +synth −7.6%) | **invasive**, and more so than when first declined: `laterite-ags4-emit` now **publishes to crates.io** (0.9.0), so changing `EmitGroup.rows`'s type is a breaking change to a published API — an engine MINOR under the pre-1.0 convention. 9% against a 20% gate | ~~the original condition was "node gets a bench harness **and** node's emit dominates there"~~ **CLOSED 2026-09-01 (#823), both halves resolved — and the candidate itself no longer exists.** The #790 arc's #792 made the write view borrow (`EmitGroup.rows: &[Vec<String>]`), deleting the deep-clone this row priced — the breaking change it weighed happened anyway, for the streaming rework's own reasons, inside the emit 0.12.0 cycle. And the node write bench now exists (`laterite-node/bench/perf-matrix.mjs`, the #823 lane): the node emit door measures at rust-parity throughput across the whole ladder, nothing dramatic. Nothing is left to revisit |
| keychain S3 — memoise the parent `_id` across a group's rows — was issue #111 | ~5–15% of *id-minting*, which post-S1/S2 is no longer the dominant stage of the keyed read | end-to-end ceiling falls **below the tranche floor**. Contained, but there is nothing left to win here | id-minting becomes the dominant stage of the keyed read again |
| keychain S4 — fuse UUID→string into the Arrow builder, skipping the per-row 36-char `String` (`keychain.rs:181`) — was issue #111 | **measured 2026-08-03**: with both `to_string()` calls removed outright, `group_row_ids/SAMP-10k` 1.327 → 1.045 ms = **−21.6% of that stage**. That is an over-stated bound (it drops the hyphen formatting too, which S4 keeps); scaled to the keyed read it lands in the estimated **4–8%** band | straddles the 5% candidate floor **from below** once the over-statement is discounted, and it touches the `_id`/`_parent_id` byte-identity contract guarded by the cross-surface golden. Delicate work for a sub-floor prize | the keyed path is being revisited for another reason and this can ride along under the existing golden-pin |
| keychain S5 — emit `_id`/`_parent_id` as a 16-byte DuckDB `UUID` instead of 36-char Utf8 — was issue #111 | not a perf candidate at all | it is a **contract change**: the column *type* moves, touching the golden, `test_content_keys.py`, `p3-content-keys.test.ts`, `arrow_cols`, every surface reader and the extension's `read_ags`. It sat inside a perf issue where the stopping rule cannot judge it | a DuckDB-heavy consumer asks for it. Then it gets its own design page and a deliberate decision, not a slip-in — the contract it would change is described in [[laterite-ags4-reference]] |

### The queue — ranked, big-cheap first

Every row below has been read in the code and then adversarially re-read by a
second pass. Prize cells quote the **measured ceiling** from
[[core-perf-baseline]] wherever one exists; static allocation counts are marked
as such, because they are counts, not timings.

> [!note] **Re-baselined 2026-08-31 (#821, absorbing #807).** Every stage on
> the baseline table was re-measured in one session because three landed
> changes (#790/#804 emit streaming, #803 cell enum, #777/#800 effective
> dictionary) postdated the numbers the queue's arithmetic rests on. Verdict:
> the read/validate stages moved 6–22% **with the machine** (one direction,
> ns-scale benches flat — [[core-perf-baseline]] has the reasoning), the emit
> orchestrator moved **down** 16–25% against that tide (#790's arc landing),
> and the stage *shape* is unchanged. No queue row's ranking changes; no new
> time candidate cleared the 5% floor; the validator band stays closed at
> 0.80×. The time queue below therefore stands as the record of a finished
> layer — the campaign's open front is the **memory lane** (see below).

| # | candidate | band | frequency | prize (ceiling) | cost | bench | coverage gap to close with it |
|---|---|---|---|---|---|---|---|
| ~~1~~ | ~~parent KEY-tuple set rebuilt per child~~ | validator | per-row | **LANDED T1 — relational −13.9%** | contained | yes | cache-reuse test added |
| ~~2~~ | ~~`build_column` casts per cell via `parse_value` (courier `String`, `canonical_type` re-resolved per cell)~~ | laterite-ags4-types | per-cell | **LANDED T3 — typed_read_file −78% (75.9→16.7 ms)** via direct per-cell parse (no arrow-cast) | contained | yes | `typed_build_parity.rs` pins `build_column` ≡ `parse_value` |
| ~~3~~ | ~~`line_format`'s three per-line `chars()` walks~~ | validator | per-line | **LANDED T1 — line_format −48.5%** | contained | yes | `char_span` test added |
| ~~4~~ | ~~`raw_lines` pushes one owned `String` per line under `validating()` (`parse/lib.rs:721`)~~ | parse leaf | per-line | **DECLINED T4 — measured ~9.9 ms** (validating 144.2 vs lean 134.3 @ 25 MB): only ~6.9% of `parse_bytes`, ~1.9% of `check_file`, and that is the *ceiling* (a span rewrite keeps the `Vec` push) | **invasive** (ledger said contained — WRONG: removing the alloc needs `ParsedFile<'a>` / whole-file-decode + span, changing the `pub RawLine.text` API across `line_format`/`structure`/`fixes`/PyO3) | yes | fails the 20% invasive gate at ~5% realized |
| ~~5~~ | ~~`Sidecar::assemble` walks the file a second time inside `mint` to rebuild the byte index~~ | core + trust | per-file | **LANDED T4 — mint −13.3% (324→280 ms @ 25 MB)**: reuse the validating parse's source-true offsets instead of re-walking (`assemble_from_parsed`) | contained | yes (new `trust/mint` bench) | non-UTF-8 mint pinned (core fallback + trust end-to-end) |
| ~~6~~ | ~~node's `table_ipc` has no `with_keys=false` escape, so the keychain pass runs on the default `table(code)` call and the keys are then stripped~~ | surfaces (node) | per-row | **LANDED T6 — default `read + table(all)` 692 → 152 ms (−78%)**: the keychain is ~96% of the native build (isolated: keyed `tableIpc(all)` 509 ms vs keyless 18 ms), so `withKeys=false` skips it on the keys-less default; only the explicit keyed variant still pays it | contained | yes (`node/bench/read.bench.ts`) | `.sql()`/`.at()` after a prior plain `table()` pinned (`p3-content-keys.test.ts`) |
| ~~7~~ | ~~`parse_compat_arrow` builds a `RecordBatch` for every group even when `only_groups` narrows~~ | surfaces (compat) | per-group | **LANDED (#99) — narrowed `AGS4_to_dataframe` 144.4 → 131.2 ms (−9.1%)**; the native `parse_compat_arrow` drops 121.7 → 111.0 ms when one group of 123 is asked for. `only_groups=None` builds every group exactly as before, and ~25 MB of peak RSS goes with the tables that are no longer materialised | contained (~20 lines: an `only_groups` parameter on the pyfunction, threaded from `AGS4_to_dataframe`) | yes (paired native + end-to-end) | `test_compat_only_groups.py` — the strict raises (dup GROUP / ragged / dup heading) still fire on groups the caller filtered out |
| ~~8~~ | ~~the GIL is never released anywhere in `laterite-py` (zero `allow_threads`/`detach`)~~ | surfaces (wheel) | not hot — concurrency only | **LANDED T6 — concurrent throughput: validate 0.99 → 5.08×, read 0.96 → 3.53× @ 10 cores** (`Python::detach` around the pure-Rust compute in `run_check`/`parse_arrow`/`parse_compat_arrow`); single-call latency unchanged | contained | yes (`tools/bench-gil-throughput.py`, T6) | `test_gil_released.py` proves a concurrent thread advances *during* the native call |
| ~~9~~ | ~~`EmitGroup` owns `Vec<Vec<String>>`, so a caller deep-clones an already-owned matrix~~ | emit + node | per-cell | **DECLINED 2026-08-03 — measured −1.63 ms, −9.0% of `emit_ags4/report`** against a 20% invasive gate; **the candidate was then deleted by #792** (the write view borrows) — see *Priced, declined* above for the close (#823) | **invasive** — changes a public field type on a crate that now publishes to crates.io | yes (`benches/emit.rs`; node benches write too since #823's matrix lane) | — |
| ~~10~~ | ~~the process uses the system allocator; `parse_bytes` is allocation-bound (~5M blocks / 25 MB, dhat-confirmed)~~ | parse leaf → all surfaces | per-cell alloc | **LANDED — mimalloc `#[global_allocator]` on all 3 native artifacts: wheel end-to-end read −22%, validate −14%; lat/node the same read win** for +163 KB (.so) / +116 KB (lat) | contained (dep + 3 lines/artifact; C `libmimalloc-sys` on the abi3 matrix, the accepted dep-shape cost) | yes (dhat + wheel e2e) | wheel 681 + node 289 green; Arrow release-callback handoff proven safe |
| ~~11~~ | ~~the *keyed* keychain (`group_row_ids`) rebuilds a per-row all-columns `HashMap<String,String>` and re-hashes with a fresh `Sha256` per row — paid on every `.sql()`/`.at()`/`keys=True`/`to_duckdb` read (#6 skipped it on the keys-less default but left the keyed path untouched)~~ | reference leaf (keychain) → surfaces | per-row | **LANDED (Steps 1+2, #106/#108).** S1 `perf/keychain-positional-keys` — kill the per-row map, read KEY cells positionally: end-to-end node 25 MB **keyed** read **521 → 277 ms (−47%)**, keychain overhead ~386 → ~144 ms (−63%); isolated 1002 → 201 ns/row. S2 `perf/keychain-streaming-hash` — borrow + stream KEY cells into one reused `Sha256` (`finalize_reset`) behind a `ByteSink` trait: isolated `group_row_ids` **353 → 132 ns/row (2.68×)**. S2 end-to-end is flat — post-S1, id-minting sits one stage *behind* the Arrow key-column build + IPC, which now dominate the keyed read | contained (byte-identical; public signatures unchanged) | yes (`benches/keychain.rs` criterion + node `read.bench.ts`) | `content_id_pins_the_cross_surface_golden` + injectivity + node `p3-content-keys.test.ts` pin byte-identity |

> [!note] **Keychain fast-follows (#11 continued) — priced, not taken.** After
> S1+S2 the keyed read is bounded by the **Arrow key-column build + IPC**, not
> id-minting, which reprioritises what remains of the plan:
> - **S3 — memoise the parent `_id`** across a group's rows (siblings share a
>   parent key-chain): ~5–15% of *id-minting*, concentrated on deep child groups
>   (SAMP/SPEC/GEOL). But id-minting is no longer the dominant stage, so its
>   *end-to-end* ceiling now falls below the tranche floor. Contained; recorded.
> - **S4 — fuse UUID→string into the Arrow builder** (`laterite-ags4-types::arrow_cols`,
>   a reused `[u8;36]`): this targets the *new* bottleneck, so it is the more
>   promising end-to-end of the two (~15–30 ms / ~4–8% ceiling). Contained; the
>   ranked fast-follow if the keyed path is reopened after 0.8.0.
> - **S5 — emit `_id`/`_parent_id` as a 16-byte DuckDB `UUID`, not 36-char Utf8**
>   (**FLAG** — changes the column *type*, not identity: proven round-trip-equal
>   and join-compatible with existing VARCHAR `.duckdb` files). Its own design
>   page and a deliberate owner decision, not a 0.8.0 slip-in.

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

> [!note] **Two of these were re-verified against the current tree in the
> 2026-08-31 re-baseline (#821), because #807 flagged that the code under them
> had moved.** Both stay retired:
>
> - `fields_with_status` migrated from the validator's `relational.rs` to
>   `laterite-ags4-reference::effective_dict` in #800 and is now private
>   behind `key_fields`/`required_fields`. The allocation is still there
>   (`to_ascii_uppercase()` plus a `to_string()` per matching heading). #807
>   worried it now sat on five rule families' path; read against the live
>   tree, its only consumers are the **relational** family's six call sites,
>   and every one is per-*group*, not per-row. The `dictionary` family — the
>   whole Rule 7/9 surface that also moved onto the shared module — measures
>   ~0.11 ms, and `relational` (99.3 ms) moved with the session's drift, not
>   ahead of it. A per-group allocation inside a stage whose 5% floor is
>   ~5 ms stays below measurement resolution.
> - `references::check` still builds a `HashSet<String>` per borrowed heading
>   after the #800 rewire (the `merged` closure), and the whole `references`
>   family still measures **0.42 ms** — the ceiling is the family.

### Correctness and measurement — outside the ranking, not subject to the floor

| id | item | status | why it is not a perf row |
|---|---|---|---|
| **C1** | `from_shared` collapsed duplicate headings (`ags4_codec.rs`) | **LANDED** (#88) | wrong data, not merely lost data — see below |
| **C2** | `arrow_in.rs` had zero tests and compiled under no CI test job | **LANDED** (#87) | untested code shipping inside the wheel and the node binding |
| **M1** | `laterite-ags4-types/arrow` + `laterite-ags4-emit/arrow` (651 lines) were compiled out of the coverage build by resolver-2 feature unification once their only four enablers are excluded | **LANDED** (#87) | the 88% gate was not measuring the Arrow boundary at all |
| **M2** | the bench fixture is **clean by construction**, so `findings::add`, rule 10b's per-bad-row `format!`/`join`, rule 11c's O(child × target) scan and the entire FYI tier are never executed by any bench | **LANDED T5 + T5-followup** — `validate/error-path` benches both gates on over SIZE-SCALED dirty twins (`forge scale --inject --density`); emitting 314k findings is **−6% vs clean**, 10b's line ~310 ns/finding but structurally capped | **closed, ceiling measured: DECLINED** — below the 5% floor at any realistic fault density; 11c stays unreachable (the `wide` scaffold has zero RL columns) |

**C1, as measured rather than as queued.** The row above originally read
"silently collapses duplicate headings". That understated it. Rows are keyed by
heading name, and every consumer walks `headings` *positionally* then indexes the
row *by name* (`laterite-ags4-excel:144`, node `lib.rs:117`, wasm `lib.rs:1198`,
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

### T3 — The typed read's per-cell overhead — ✅ DONE (2026-07-25)

**Landed:** `types/typed_read_file/large` **75.9 ms → 16.7 ms (−78%)**, 312 MiB/s
→ 1.39 GiB/s — a **~4.5×** speedup on the exact `build_record_batch` every typed
surface pays. Well past the 5% candidate floor and the 10% tranche floor.

**What changed.** `build_column` matched `canonical_type` once per column, then
called `parse_value` per cell — which re-resolved `canonical_type`
(`trim().to_uppercase()` + table lookups) *again* for every cell and boxed the
result through a `serde_json::Value`. The fix parses each borrowed cell straight
into its typed Arrow builder, dropping both the redundant resolution and the
boxing:

- **String / Enum / unknown (~60% of headings):** build `Utf8` directly (trim +
  empty→null) — no dispatch, no `Value`. This is the bulk of the win.
- **Integer (`0DP`):** `parse_ags_integer` (== `int(float(s))`, range-guarded)
  straight into an `Int64Builder`.
- **Decimal (`nDP`/`nSF`/`nSCI`):** `parse_ags_decimal` (finite `f64` only)
  straight into a `Float64Builder`.
- **Bool (`YN`) / Datetime (`DT`):** unchanged custom arms.

**Not via Arrow's cast kernel.** The owner first proposed building `Utf8` then
bulk-casting the numerics through `arrow::compute::cast`. A spike (branch
`spike/t3-arrow-cast`) proved that works and is parity-clean, but a **paired
bench showed the direct parse is ~16% faster still** (the cast builds an
intermediate `Utf8` column + a second pass, and needs a finite-null reconcile
because Arrow's string→float admits inf/NaN) **and** referencing `cast` links the
arrow-cast kernels into the wasm bundle, a **+3.5 MB (~50%)** bloat that broke the
PWA precache limit (`e2e`). The direct parse needs neither — same speedup axis,
zero wasm cost — so it replaced the cast approach.

**Parity is the gate, and it holds.** `build_column` is byte-parity with the
per-cell `parse_value` build — Arrow-representation identical (`ArrayData` logical
equality, the object the C-data interface / IPC hand to polars / duckdb / arrow-js)
over **663 columns / 2,557,209 cells** of the fixture plus the edge seams, pinned
permanently in `laterite-ags4-types/tests/typed_build_parity.rs`. A live wheel diff
confirmed it end-to-end: every group's polars DataFrame (schema + dtypes + values
+ nulls) unchanged, on both the frame and `keys=True` paths.

**Surfaces inherit it free:** all typed surfaces funnel through `build_column`
(py→polars via `build_record_batch_synth`, node→arrow-js and wasm→duckdb via IPC),
so no API/feature change. The compat/pandas builder (`build_record_batch_compat`)
is a separate all-`Utf8` path and is untouched, so python-ags4 parity is
unaffected.

### T4 — The parse leaf and the redundant certify walk

**Candidates:** #4 (`raw_lines` span rewrite), then #5 (`assemble_from_parsed`).

**#4 — DECLINED (2026-07-25), measured.** A throwaway bench priced the raw_lines
build directly: `parse_bytes_opts(validating)` **144.2 ms** vs `(lean)` **134.3 ms**
@ 25 MB → the per-line `into_owned()` + push is **~9.9 ms**, only ~6.9% of
`parse_bytes` and ~1.9% of `check_file`. That is the *ceiling* (full removal); a
span rewrite keeps the `Vec` push, so the realized win is ~5%. And the ledger's
"contained" was wrong: `raw_lines` is read by `line_format`/`structure`/`fixes`
via the `pub RawLine.text` field, so removing the allocation needs a
lifetime-bound `ParsedFile<'a>` (or a whole-file-decode + offset scheme) rippling
across those rules, the PyO3 boundary, and tests — **invasive**. ~5% realized
against a 20% invasive gate → not worth it. `parse_bytes` does not move ≥5% by any
non-invasive means, so **by this tranche's own exit clause the parse leaf is
closed.**

**#5 — DONE (2026-07-25).** Bench-first established the first `mint` baseline
(`trust/mint`, the trust crate had none): **mint 324 ms**, of which the redundant
index walk (`index_walk`) is **45.3 ms = 14%**. `mint` parsed the file to validate
it, then `Sidecar::assemble` walked it *again* via `index_ags4_bytes` to rebuild
byte offsets the first parse already had. New `Sidecar::assemble_from_parsed`
reuses that parse's `group_records` (guarded by `byte_offsets_source_true`, which
is `true` for the clean UTF-8 files that certify); `mint` hands its validating
parse straight in. **mint 324 → 280 ms, −13.3%** (p < 0.05), clearing the ≥10%
exit. Semantics preserved: a non-UTF-8 parse is not source-true, so it falls back
to the lean/`Reject` re-walk — the byte-identical rejection — pinned by
`assemble_from_parsed_falls_back_when_not_source_true` (core) and
`the_mint_still_rejects_a_non_utf8_file` (trust, closing the flagged gap). The
`sha256` freshness pass stays (it is not redundant). This closes T4.

**T4-followup — the deeper allocation profile, and the allocator candidate
(2026-07-25).** The criterion benches TIME the read stages; they cannot say
whether a stage is slow because it *allocates* or because it is
compute/bandwidth-bound. A `dhat` heap profiler (`laterite-ags4-types/examples/dhat_read.rs`,
dev-only, `arrow`-gated) attributes the allocations *inside* each stage over the
25 MB `large` fixture, one stage per run:

| stage | total | blocks | live peak | reads as |
|---|---|---|---|---|
| `build_record_batch` (all groups) | 61.3 MB | **9,967** | 782 KB / 27 blocks | **allocation-optimal** — ~10k allocs for a 25 MB build. The 16.7 ms (post-T3) is a compute/bandwidth **wall**, not an allocation problem. T3 is genuinely done. |
| `parse_bytes` | 353.7 MB | **4,982,948** | 160.7 MB / 3.4M blocks | **allocation-bound** — ~5M allocations, ≈1 owned `String` per cell, 160 MB live. A ~500× allocation gap to the build. |

So the read path's allocations live almost entirely in the parse leaf — exactly
the `String`-per-cell that #4 named and that a non-invasive edit could not remove.
That points at the allocator itself, and a global-allocator swap prices it without
touching a line of parse logic:

**mimalloc probe — parse `−21.5%` (139.2 → 108.5 ms @ 25 MB, p < 0.05).** A
throwaway `#[global_allocator]` swap on the parse bench (reverted) turned a ~5M-alloc
workload from 170 → 219 MiB/s. It is the single biggest cheap win the campaign has
surfaced: `parse_bytes` is on **every** read across all four surfaces, so ~21% here
flows everywhere, and dhat explains *why* it lands — an allocation-bound leaf is the
canonical case a per-thread-heap allocator accelerates.

**It clears every floor (5/10/20%) by a wide margin, so the gate was never perf —
it was dep-shape, an owner call.** For the `lat` binary the swap is trivial and
safe (it owns `main`). For the **shipped wheel** it was a real decision: a
C-compiled `libmimalloc-sys` rides the abi3 cross-platform build matrix
(musl/macOS/Windows), against a base dep-shape deliberately kept to polars+duckdb;
and a *library* setting the host process's global allocator is a heavier commitment
than a binary doing so.

**ADOPTED — the wheel measured, then taken on all three native surfaces.** Before
deciding, the swap was measured end-to-end on the wheel (public `laterite.read`/
`laterite.validate`, 25 MB, median of 9, reproduced), not just the isolated parse
bench:

| operation | baseline | + mimalloc | change |
|---|---|---|---|
| validate (parse + rules) | 260.4 ms | 224.0 ms | **−14.0%** |
| read (parse + load engine) | 147.0 ms | 112.7 ms | **−23.3%** |
| read + all groups (full typed) | 174.1 ms | 135.2 ms | **−22.3%** |

The isolated ~21% parse win lands as **~22% faster reads and 14% faster validate**
end-to-end (reads are parse-dominated; validate spends more in the rules engine,
which the allocator doesn't touch). Size cost on the `.so`: **+162.6 KB (+0.77%)**
of 20.1 MB; on `lat`: **+116 KB (+1.4%)**. Build cost: ~40 s one-time C compile per
platform. The flagged risk — a library swapping the allocator corrupting the
Arrow→polars/pyarrow handoff — **did not materialise**: the full read (all 123
groups through the Arrow/DuckDB/polars bridge) and validate ran clean, because the
engine leaves Rust via Arrow release callbacks, so Rust frees what Rust allocated.
`#[global_allocator]` is set in each final artifact — `laterite-cli` (`lat`),
`laterite-py` (wheel), `laterite-node` (addon); a shared leaf crate cannot set one.
**wasm is excluded** (different toolchain; already native-speed, keeps dlmalloc).
Wheel suite 681 passed, node 289 passed, lat validates clean. See
[[core-perf-baseline]].

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

**DONE (2026-07-25) — numbers, no optimisation.** `gen-bench-fixtures.sh` now
also emits `dirty.ags` (`forge gen --combine rule10a,rule10c,rule8,rule5,rule19,rule13`,
seed 0 → ~64 KB, ~10 rules firing), and `validate/error-path` benches the
error-reporting half two ways with both tier gates on:

| bench | time | thrpt | reads as |
|---|---|---|---|
| `error-path/large/gated` (25 MB CLEAN, warnings+FYI on) | **274.6 ms** | 86.5 MiB/s | ~+2% over gates-off `check_file/large` (269 ms) — the FYI/warning tier walk (incl. rule 16's abbr scan) is **below the 5% floor at scale** |
| `error-path/dirty/gated` (64 KB dirty) | **2.245 ms** | 27 MiB/s | the emission path (`findings::add`, rule 10b/11c dirty branches) runs at **27 vs 86 MiB/s clean — ~3.2× slower per byte** |

So the error path is proportionally expensive *per byte* but the FYI tier walk is
cheap at scale, and on a realistically-sparse dirty file the absolute cost is
bounded by finding count — no error-path candidate clears the 5% floor at today's
achievable fault density. **What would change that is the missing `forge scale`
fault-density mode** (a size-scaled densely-dirty rung): only then can rule 11c's
O(child × target) asymptotic and the 10b per-bad-row `format!`/`join` be priced at
25 MB. Until it exists, this tranche is closed with the numbers above.
**Exit met:** the rungs are committed and produce numbers; nothing landed.

**T5-followup — the fault-density mode, and the verdict (2026-07-25).** The
missing piece landed: `forge scale --inject <token> --density <p>` (the new
`rule10b`/`empty-required` injector + a deterministic `apply_dense`) builds
size-scaled densely-dirty twins of `large`. `gen-bench-fixtures.sh` now emits two
clean-*isolated* 25 MB dirty rungs (`dirty-r16` = 314,689 Rule-16 findings only;
`dirty-r10c` = 4,369 Rule-10c findings only), and `validate.rs` prices the
emission path three ways:

| bench | clean | dirty | delta | reads as |
|---|---|---|---|---|
| `error-path/*/gated` (whole `check_file`) | 269.8 ms | `dirty-r16` (314k findings) **253.4 ms** | **−6%** | emitting 314k findings is *faster* than validating the clean twin — the finding-build is fully absorbed (undefined abbrevs fail the FYI table scan fast, where valid codes pay the full lookup) |
| `relational-emit/*` (`relational::check`) | 89.1 ms | `dirty-r10c` (4,369 findings) **108.1 ms** | +19 ms (+21%) | ~4.3 µs/finding for the relational family — but only at all-SAMP-orphaned density, ~7% of whole validate |
| `rule10b-emit/<n>` (10b's `format!`/`join`, isolated) | — | 10k → 3.00 ms, 200k → 64.4 ms | **~310 ns/finding, linear** | the named line, cascade-free (synthetic all-empty-REQUIRED ABBR rows; unique keys → no 10a, root group → no 10c) |

**Verdict: the error-emission path is NOT a candidate — DECLINED.** At the
whole-engine level, emitting findings is not even a visible cost (dirty ≤ clean).
Rule 10b's specific line is ~310 ns each but its REQUIRED-non-KEY fields are
*structural* (`TRAN_AGS`, the `ABBR/UNIT/TYPE` `*_DESC` definitions), so a real
file caps it at a few hundred findings → tens of µs, far below the 5% floor of a
270 ms validate. The relational family shows a genuine ~4 µs/finding cost, but
only at pathological density and still ~7% of validate. Nothing clears the bar at
any realistic fault density — **the tranche is now closed with the ceiling
measured, not argued.** (Rule 11c's O(child × target) stays unpriced: the `wide`
scaffold carries zero Record-Link columns, and 11c cannot fire on a file with no
RL data, so no corruption of this fixture reaches it — pricing it needs an
RL-bearing scaffold, a separate forge capability.)

### T6 — The surfaces (only once the core stopping rule has fired)

**Candidates:** #6, #7, #8. **Benches first: all three.**

**Node harness — DONE (2026-07-25).** Node had no benchmark of any kind, so its
read cost was invisible. `laterite-node/bench/read.bench.ts` (vitest `bench()`,
no new dep, `npm run bench`) mirrors the Rust `typed_read_file` / Python read
bench on the 25 MB rung. First numbers:

| bench | mean |
|---|---|
| `read` (parse only) | 133 ms |
| `read + table(all groups)` (default, keys stripped) | 692 ms |
| `read + table(all, keys)` | 693 ms |

Default ≈ keyed (692 vs 693 ms) made **#6 rankable**: the content-addressed
keychain is computed on the default keys-less `table(code)` and then thrown away —
the strip is free, the keychain is not.

**Wasm harness — DONE (2026-07-25).** The browser engine was the last surface with
no perf floor. `web/bench/wasm-read.bench.ts` (vitest `bench()`, its own lane,
`npm run bench:wasm`) drives the SAME browser cdylib the app loads — the glue
instantiated straight from the built `.wasm` bytes — over the 25 MB `large`
fixture, so its numbers sit on the shared axis. The JS→wasm boundary copy of the
input and the Arrow-IPC return trip stay *in* the measured path, because a browser
pays them for real:

| bench | mean | native/node reference | reads as |
|---|---|---|---|
| `read` (parse only) | **118.2 ms** | native `parse_bytes` ~119 ms | wasm parses at **native speed** — the boundary copy is negligible against 25 MB of work |
| `read + arrow_ipc(all groups)` (keyless) | **142.1 ms** | node `table(all)` 152 ms | the browser explorer's actual read cost, on par with node |
| `read + arrow_ipc(all, keys)` | **995.7 ms** | keychain-dominated | the content-key chain again (cf. #6): ~850 ms over keyless, the same pass node pays |
| `validate` (both gates off) | **273.4 ms** | native `check_file` 269.8 ms | within **1.3%** of native |

**No wasm-specific candidate exists.** Parse is within 1% of native and validate
within 1.3%; the boundary copy the surface was suspected to pay is lost in the
noise at 25 MB. The only large cost is the keyed keychain, which is #6's cost, not
a wasm one — and the keys-less default already skips it. The surface now has a
floor; nothing to land.

**Wasm allocator — investigated, dlmalloc kept (2026-07-25).** The wasm read path
is allocation-bound for the *same* reason native is — it runs the identical
`parse_bytes` leaf (~5M allocations / 25 MB, dhat above) — so the native mimalloc
win was a natural thing to chase here too. But mimalloc (C, needs a native/WASI
toolchain) **cannot target `wasm32-unknown-unknown`**, and the two serious
pure-Rust alternatives both *lose* to the default **dlmalloc** on the read bench:
**talc +5.7%** and **rlsf +25%** on parse (measured, both `#[global_allocator]`
probes reverted). dlmalloc — the Rust wasm default — is already well-tuned for this
~5M-small-alloc workload (it is also why wasm parses *faster* than native under the
system malloc: 118 ms vs 139 ms). The size-optimised allocators (`wee_alloc`,
`lol_alloc`) are slower by design and were not benched. **Decision: keep dlmalloc;
there is no wasm allocator win.**

**#6 — DONE (2026-07-25).** An isolation probe (parse once, then loop `tableIpc`
over every group both ways) priced the keychain directly: keyed `tableIpc(all)`
**509 ms** vs keyless **18 ms** — the keychain is **~96%** of the native build, not
a minor pass. The native `table_ipc` gained a `with_keys` arg (defaults ON — the
relational contract); the DEFAULT `table(code)` now calls it with `false`, so a
keys-less read skips the keychain wholesale instead of building-then-stripping.
Result: **default `read + table(all)` 692 → 152 ms (−78%)** — now only ~22 ms over
parse. The keyed variant is unchanged *by #6* (649 ms, within drift): the keychain
is paid only when a caller asks for `_id`/`_parent_id`. **The keyed keychain itself
was then made cheaper as its own candidate — see queue #11 (Steps 1+2, #106/#108).**

**#7 — MEASURED, DEFERRED (2026-07-25, issue #99).** `bench-vs-python-ags4.py`
always does a full read, so a targeted probe was needed: on the 25 MB rung the
narrowed `AGS4_to_dataframe(only_groups=[1])` still costs 93% of the full read
(158 vs 169 ms) because `parse_compat_arrow` builds + crosses all 123 compat
tables regardless. But that build is only **14.1 ms** (`parse_compat_arrow` 157.8
vs `parse_arrow` 143.7 — the parse is shared and dominates), so the ceiling is
**~14 ms ≈ 9% of a 1-group narrowed read, 0 on a full read**. It clears the 5%
candidate floor, but it is the smallest win on the board and narrowed-reads-only.
A prototype (`parse_equiv_probe`) also proved the compat and native `read()`
parses are byte-identical, so the "reimplement compat on the lazy handle" refactor
is viable — but it lands the *same* 14 ms and touches the python-ags4 parity
oracle, so its only payoff is one internal read path.

**#7 — LANDED (2026-08-03).** The small pushdown, as recommended. `only_groups`
became a parameter on `parse_compat_arrow` itself, so the Arrow tables for groups
the caller is about to discard are never built or crossed. Re-measured on the same
25 MB rung the day it landed: narrowed `AGS4_to_dataframe` **144.4 → 131.2 ms
(−9.1%)**, and the native call **121.7 → 111.0 ms** when one group of 123 is asked
for; `only_groups=None` builds every group exactly as before. Note the *ratio* in
the paragraph above had already moved on its own — a narrowed read was 93% of a
full one when first measured and 69% by August, drift in the Python-side
materialisation rather than in the prize, which stayed ~11 ms where it was.

The refactor half (reimplement compat on the lazy handle) was **not** taken and is
not queued: same win, more risk, and its only payoff is architectural. It belongs
to whoever retires `parse_compat_arrow`, not to this campaign.

What the pushdown must never narrow is the *raises*. python-ags4 rejects a
duplicate GROUP, a ragged DATA row and a duplicate heading under
`rename_duplicate_headers=False` by reading the whole file, and narrowing those
alongside the tables would turn a rejected file into an accepted one — silent data
loss dressed as a speed-up. Every group therefore still crosses its headings, line
anchors and `ragged` list; only the table is conditional.
`test_compat_only_groups.py` pins each raise firing on an offence in a group the
caller filtered out, which is the property a timing can never show.

**#8 — LANDED (2026-07-25).** The single-threaded benches are blind to this by
construction — the win is *concurrency*, not latency. A new throughput harness
(`tools/bench-gil-throughput.py`) runs the same total work across 1 vs N threads
(`ThreadPoolExecutor`). Before, N threads ≈ 1 thread (GIL held: validate 0.99×,
read 0.96×). The three CPU-bound read/validate entry points — `run_check`,
`parse_arrow`, `parse_compat_arrow` — now release the GIL for their pure-Rust
compute (`Python::detach`, cloning the one Python-bound input, the cert, out
first), so on the 25 MB rung across 10 cores concurrent **validate 5.08×** and
**read 3.53×** throughput. Single-call latency is unchanged. The regression guard
`test_gil_released.py` (every CI) proves a second Python thread advances *during*
the native call — closing the "no test exercises concurrent access" gap. The same
`detach` pattern extends to `mint`/`table_for`/emit/diff/merge/excel if a
concurrent workload ever wants them.

**Coverage closed:** #6's risk was node's single-cache-per-code architecture — a
naive `with_keys` bolt-on would hand the keys-less table to the relational layer
and break joins silently. The fix is a **two-cache split** (`#tables` keyed,
`#framesKeyless` for the default), so a plain `table()` never poisons the keyed
cache `sql()`/`at()` need. Pinned by tests asserting `.sql()`/`.at()` resolve
*after* a prior plain `table()` on the same group (`p3-content-keys.test.ts`).
**Exit:** the tranche floor applies per surface, not to the group. #9 is
deliberately **not** here: it is invasive and its stage is 22.4 ms, so it cannot
clear the 20% gate. It reopens only if node gets a bench and node's emit turns
out to dominate there. (Resolved 2026-09-01: the node bench exists — the #823
matrix lane — and #792 had already deleted the clone; the *Priced, declined*
row carries the close.)

### Refuted — do not chase

> [!warning] **Ledger correction.** The old Open row reading
> "`laterite-ags4-types::arrow_cols` … never benched in isolation" was **wrong**.
> `rust-packages/laterite-ags4-types/benches/arrow_cols.rs` benches
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
- **`laterite-ags4-excel` as "copies every cell twice".** One clone, not two, and
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

## The memory lane (#820)

Opened 2026-08-31 by #821 — the first cross-library memory measurement this
repo has made (the earlier showcase table was laterite-only). Everything in
this section is the **peak-RSS instrument** (rule of engagement 8) — except
where a subsection names another claim family itself: the attribution and
diagnosis subsections below are the dhat/diagnosis instruments (rule 8's
other half), and the wasm lane's subsection is rule 13's linear-memory
instrument, each labelled at its own tables; absolute
MB and the per-rung series live in `tools/perf-results/python-lane.json`, and
the tables quote the **×-of-output headline** (rule 9) at the **265 MB rung**
— the top memory rung (rule 11), where the interpreters' import floors have
washed out. python-ags4 is the **baseline measure** for the Python lane,
nothing more.

### The memory baseline (peak RSS ÷ file size, 265 MB rung)

| axis | baseline | ours | our door | verdict (epic decision 5, as amended — rule 12) |
|---|---|---|---|---|
| validate | 9.7× | **4.1×** (was 4.6× pre-M6, 7.6× pre-M4) | `laterite.validate` | **M4 claimed 2026-08-31 (#838)**, **M6 landed 2026-09-01 (#850, owner waiver — see the queue row)** — the structure flattening took another 11.4% off the 265 MB peak; now ~0.4× the baseline. Open only if a fresh attribution finds a mechanism clearing rule 10's floors against the smaller hold |
| read → typed | 8.9× | **4.6×** (was 5.2× pre-M6, 8.2× pre-M4) | `laterite.read` | **M4 claimed 2026-08-31 (#838)**, **M6 landed 2026-09-01 (#850)** — another 11.5% off at 265 MB; ~0.5× the baseline. Same posture as validate |
| read → strings | 8.0× | **9.2×** (was 9.8× pre-M6, 12.8× pre-M4) | compat `AGS4_to_dataframe` | M4's landing moved this cell, and M6's structure flattening rode along (−5.9% at 265 MB — the frames build over a slimmer retained parse); the residual over baseline is the frames — which ARE the product — plus the shipped-hop DuckDB-bridge premium the M5 row records as unattributed. **M1 diagnosed & declined on this instrument (#831**; the release is real but invisible to darwin peak RSS — diagnosis below), then **landed via the #833 cross-OS fork** (GO on Linux and Windows — the queue row below has the cells): this darwin cell will not move, and that is the instrument, not the fix |
| write | 8.9× | **10.7×** (was 11.2× pre-M6, 14.2× pre-M4) | compat `dataframe_to_AGS4` | rode M4's input half and M6's flattening (−5.0% at 265 MB); the rest is M3, which **inherits M1's #831 verdict** |
| write | 8.9× | **10.9×** (was 13.9× pre-M2, 14.5× pre-M6, 18.8× pre-M4) | `build_ags4(...).save()` | **queue M2 CLAIMED 2026-09-01 (#855)** — the streamed per-group join + the writer-built verdict ([[dec-emit-streamed-verdict]]) removed the second parse hold (the door's validating parse-back) and the whole-file formatted slab together: −21.3/−21.9% of the write peak at the 100/265 MB gate rungs (over rule 10's 20% floor), −18.0% at 25 MB (the import floor's share of that denominator, recorded not skipped). The remaining residual is the caller's OWN holds — the retained handle and the frames leg, the #848 variant table's caller-side rows — which ship as a docs recipe, not a fix row |

- **Every cell up to the 265 MB rung measured** — no swap growth, no deaths —
  so the table above is fully populated. The 524 MB rung is time-only by
  rule 11, and the results file records that as eleven `beyond-mem-cap`
  **refusal cells** rather than silence: the rung was run through the
  harness and refused, not skipped.
- The write cells include materialising the input through the same library's
  own read door (you cannot write what you do not hold) — attribute a write
  number by subtracting the same door's read cell, which is how M3's verdict
  below was reached.
- The laterite **import floor** is about double python-ags4's (the native
  module plus engine imports) — per-side floors are recorded in the results
  file's `import_baselines`, and they are why the small rungs' factors
  overstate everything: rank from the big rungs.
- The compat pandas cells measure **whichever hop the harness venv admits**
  — the pyarrow-accelerated one when pyarrow is importable (this table's
  committed cells), the shipped pyarrow-free DuckDB hop otherwise — and the
  two still differ by whole ×-of-output after #834 removed the intermediate
  copy: the premium lives in the DuckDB bridge leg itself (the memory
  queue's M5 row carries the corrected attribution). Since #833's round the
  harness names the hop in the results file's `versions.pyarrow` + `notes`,
  so the next reader cannot mistake the accelerated cells for the shipped
  default.

### The memory queue — ranked, big-cheap first

Same ranking function, same floors (rules 9–10), denominated in the
operation's peak RSS. Fix tickets are minted from this queue **one at a
time** (epic decision 8) and never pre-written.

| # | candidate | axis | prize (ceiling) | mechanism, as read | cost |
|---|---|---|---|---|---|
| M1 | compat read holds two whole-file representations at peak | read_strings | was priced at **~4.8×-of-output** at the 265 MB rung; **DECLINED 2026-08-31 (#831)** — the measured ceiling of the contained fix on this instrument is **~0** (held vs released diagnosis children agree within 0.4% across the whole memory ladder, 5–265 MB; the released+forced-purge variant matches at 25/100 MB) | the seam-read mechanism was RIGHT and the presumed fix still does nothing here: releasing per group genuinely frees each table (heap reuse proven — a second parse lands in the freed pages; mimalloc's own stats report the freed heap purged after one forced collect), but darwin hands pages back only on `munmap` — `MADV_FREE` and `MADV_FREE_REUSABLE` both leave `ps`-RSS **and** `ru_maxrss` unmoved, raw-syscall probe — so the sum-peak is physically real on the measuring machine. Where the OS does reclaim (Linux `MADV_DONTNEED`; darwin under memory pressure) the release pays, but the lane's instrument cannot see that. Full diagnosis below | was `contained`; the #833 fork ran 2026-09-02 and came back **GO on both OSes** — the third-family probe (`tools/perf_probe_m1.py`, committed results `tools/perf-results/m1-cross-os-probe-{linux,windows}.json`) measured the plain release cutting **−17.9/−27.9%** (Linux) and **−20.6/−29.0%** (Windows) off the held read peak at the 25/265 MB rungs, no purge hook needed on either — darwin's invisibility really was the measuring machine, not the fix. **LANDED under #833**: `AGS4_to_dataframe` pops each group's table as its frame materialises, exactly the loop the probe's `released` cell measured. The allocator-config door (#294/#448 territory) stays open |
| M2 | the `build_ags4` workflow peak stacks held input + the per-cell emit hold + materialised output | write | was **~10.6×-of-output** above its own read peak at the 265 MB rung pre-M4; post-M4 re-price + spike 2026-09-01 (#848): the **pair** of fixes below clears rule 10's invasive floor at the 100/265 MB gate rungs at ceiling — with the whole margin over the floor being what a real validation mechanism may cost, since the spike's ceiling leg deleted the verdict outright | **Re-attributed (2026-09-01, #848 — the 2026-08-31 reading below is superseded on the lane rungs)**: the `OwnedGroup` slab is *freed before the parse-back* (the #790 drop discipline held — the per-cell-bound reading survives only as the dense-TREL worst case), and the peak instead carries the **parse hold twice**: the caller's retained `ParsedFile` + the door's validating re-parse, standing on co-peak shoulders that back each other up (tail-skip alone moves ~nothing, streaming alone lands under the floor — only the pair clears; shares are not contributions). Fix shape settled by the #848 grill: streamed per-group join on BOTH doors + a **writer-built verdict** on the post-layout structure ([[dec-emit-streamed-verdict]]), plus the `out=` to-disk rider (caller steady-state, explicitly not floor arithmetic) | **CLAIMED 2026-09-01 (#855)** — the re-price ran on the post-layout lane (record on #855: B-both −23.9/−28.8/−29.9% at ceiling, margin over the floor 8.8/9.9 points at the gate rungs = the budget a real verdict could spend), the ticket was minted, and the land A/B/A on the committed lane instrument measured the real mechanism at **−18.0/−21.3/−21.9%** of the write peak at 25/100/265 MB (13.92/10.94/10.87×-of-output; A-legs spread ≤ 0.04% and reproducing the lane cells; output sha-identical on every leg at every rung) — **over rule 10's 20% floor at both gate rungs**, the 25 MB cell recorded as the structural shortfall the gate predicted (the import floor's share of that denominator). The real writer-built verdict spent 7.5/8.0 of the ceiling's 8.8/9.9-point budget. Time rider: the emit pipeline −28…−30% on the criterion ladder (all three modes), `write_ags4` itself within noise. The caller-side levers (drop the handle, capsule-bearing tables — the #848 variant table) ship as a docs recipe on the write door's docs page, not a fix row |
| M3 | compat write rides M1 | write | the write itself adds only **~1.5×-of-output** over the compat read's peak (the #805/#818 streaming door doing its job); the rest of the 14.2× **is** M1's hold | **inherits M1's verdicts** — declined with it on darwin (#831), and falls with it where the release can be seen now that the #833 fix landed; still not a separate candidate | — |
| M4 | the parse leaf holds one owned `String` per cell, under **every read-shaped operation on every surface** | validate + read_typed (and every write door's input half) | dhat at the 25 MB rung (re-run 2026-08-31, byte-identical to T4's numbers — the mechanism has not moved since July): **~6.5× the input requested-live at the parse peak**, ~1 block per cell, against a whole-operation peak RSS of ~8.2×. A span rewrite's ceiling is **roughly half the peak of every read-shaped operation**, clearing the 20% invasive gate several times over | `RawLine.text` / `DataRow.values` become spans over one decoded buffer (`ParsedFile<'a>` or offset pairs). This is the SAME rewrite the time campaign priced at ~9.9 ms and declined (time queue #4, "Priced, declined") — rule 12 re-prices it: the decline was denominated in ms, this row in peak RSS, and neither verdict carries to the other's axis. Its "revisit when" condition is met by the axis change itself | **invasive** — the public `RawLine.text` type crosses `line_format`/`structure`/`fixes`/PyO3; design page DONE: [[dec-parse-cell-representation]] (2026-08-31 — the shape, the scope, and the spike-gated mint conditions); **minted as #838** (2026-08-31, taking the queue slot #834 vacated) |
| M5 | the shipped pyarrow-free pandas hop paid a whole-file polars intermediate | read_strings (the default `[compat]` install only) | was priced at **~+3.0×-of-output** over the pyarrow hop at the 100 MB rung (#831 diagnosis children); **CLAIMED 2026-08-31 (#834) — and the price was misattributed**: removing the copy moved the shipped hop's read peak **−0.9% / −0.6%** at the 100/265 MB rungs (−1.9%/−4.2% at 25/5 MB; write ~0; A-legs bracket ±0.1%, the pyarrow control unmoved) and its read time −3–5%, while the premium itself — **~+2.4–2.6×-of-output at every rung** — survives the fix | the copy existed and is gone: the native table's own `__arrow_c_stream__` registers into DuckDB and the rename rides the SQL projection. But it was never sum-like at peak — each group's freed copy is reused by the next (the same heap-reuse behaviour #831's probe 2 measured), so the intermediate contributed ~one group's worth to peak, not the file's. The surviving premium is the **DuckDB bridge leg itself** (Arrow scan → engine vectors → NumPy `.df()`), sum-like on this instrument and **unattributed** — pricing it is a fresh diagnosis with its own code-reading, not a rider on this row | landed as an avoidance + simplification (#834; tests pin the no-intermediate path, hostile identifiers, the strict-zip raise and cross-hop frame equality). The owner's evidence trigger **FIRED**: post-fix the shipped hop still trails the pyarrow hop by **~+18–19% of the operation's peak** at the top rungs — recorded on #834 for the owner's dep-shape decision, nothing acted on |
| M6 | the retained parse **structure** — per-row span-vec heap blocks plus source-byte fields the validator never reads | validate + read_typed (and every `ParsedFile` holder: every write door's input half, and M2's constructed verdict had it landed) | was priced by the #848 attribution as most of the retained hold beyond the buffer, sitting *at* rule 10's gate; the spike's A/B/A on the lane instrument measured the full flattening at **−6.9/−6.2% (25 MB) and −13.1/−10.7% (100 MB)** of the validate/read_typed peaks — **under the 20% floor on all four gate cells** (A-spread ≤ 0.035%; A-legs reproduce the committed cells within 0.25%), and the row stood declined on that verdict. **LANDED BY OWNER DECISION 2026-09-01 (#850)**: the measured ~10–13% plus the time rider — `parse_bytes` −9…−15%, `check_parsed` unchanged — was judged worth the remaining effort with the spike already built and contract-green. A dated waiver of the floor for this row, argued in absolute terms on #850; NOT a precedent that ~10% clears an invasive gate | the shape was built exactly as designed (per-group arena + slim 12-byte row index, `RawLine` 24 → 16 B, per-row/per-line offsets profile-gated — the inventory held: nothing outside the leaf's own tests reads them) and the structure is real — but the requested-bytes pricing OVERstated its RSS contribution, the inverse error to the undercount the design reasoned from: the span content itself (8 B/cell) survives in the arena by construction, and the old per-group rows-Vecs' doubling headroom was largely never-touched pages RSS never paid (requested 123.2 MB vs ~68 MB content at the 100 MB rung, #848's own table). What flattening removes — the row/line slimming plus allocator block overhead — is ~13% of the operation peak at 100 MB, ~7% at 25 MB where the import floor pads the denominator. dhat prices requests; RSS pays touched pages — in both directions | **landed** (owner decision, floor waived — the record and the waiver are both on #850): [[dec-parse-structure-layout]] back to `accepted` with the gate outcome noted; the one-arc migration is the #850 land PR, before/after refreshed in `tools/perf-results/python-lane.json` at the landed tree. M2's "re-price on the post-layout lane" precondition is live again — see its row |
| M7 | the node write door stands the file up beside itself twice more than the engine needs: `buildAgs4` accumulates every group's IPC `Buffer` before the one native call (the `ipcGroups` slab in `rust-packages/laterite-node/ts/index.ts`), and the native side then decodes the whole vec into a second full typed materialisation before the streaming emit runs (`emit_ags4_from_ipc` → `group_from_ipc` in `rust-packages/laterite-node/src/lib.rs`, all groups up front) | write (node lane, #823) | door increment over the same lane's typed hold: **5.40×-of-output at 265 MB (10.25 − 4.85) and 5.06× at 100 MB, vs the rust door's 1.67×/1.62×** on the same run — the ~3.4–3.7× delta is ≈ **33–36% of the write stage's peak**, over the invasive gate *at ceiling* | mechanism as read, not yet priced: the post-M2 engine already streams per group, so a per-group handoff shape exists on the native side — but #848's co-peak lesson applies in full (shares are not contributions; the slabs may back each other up), so nothing here is a prize until variant children run alone *and* paired on the node lane | **unpriced** — a diagnosis round before any mint; reshapes the napi handoff, so invasive |
| M8 | the codec read door stands the file up three times over: the span `ParsedFile`, then every group's rows re-materialised as `HashMap<Arc<str>, String>` — one owned `String` per cell plus per-row map overhead (`read_ags4_with`'s conversion, `rust-packages/laterite-ags4-core/src/ags4_codec.rs`, `AgsGroup.rows`) — then the dumped group's projection into `Vec<Vec<String>>` + rendered text (`rust-packages/laterite-cli/src/commands/read.rs`) | read (CLI lane, #825) | the door holds **10.92×-of-input at the 265 MB rung** (11.20× at 100 MB) against the engine's ~3.3× span-parse hold on the same tree — a ~7.6×-of-input increment, ≈ **69% of the operation's peak**, over the invasive gate *at ceiling* | mechanism as read, not yet priced: the map slab, the projection copy and the render buffer overlap in construction, and #848's co-peak lesson applies in full (shares are not contributions — the `ParsedFile` may already be gone at t-gmax), so variant children must run alone *and* paired before anything here is a prize | **unpriced** — a diagnosis round before any mint; `AgsGroup.rows` is core's public read projection with consumers beyond the CLI (excel, the surfaces' recover-flag paths), so invasive |

> [!note] The time queue and this one never share a table, and neither do the
> instruments behind them (rule 8). When an M-row is opened, the diagnosis
> step is `dhat`/`tracemalloc` on our own side — those numbers go on the fix
> ticket, not in the baseline table above.

### The 2026-08-31 attribution pass (dhat — requested bytes, never RSS)

Run when rule 12 landed, to price the absolute candidates the parity column
had been hiding. Everything below is the **diagnosis instrument** (dhat,
requested bytes live at t-gmax) and shares no table with the peak-RSS
figures above. Both instruments re-ran their July workloads and reproduced
them byte-for-byte — the fixtures are pinned and the mechanisms had not
moved — so these are confirmations with today's date, not new drift.

| instrument, workload | live at peak | reads as |
|---|---|---|
| `dhat_read.rs`, parse stage, 25 MB rung | ~6.5× the input, ~1 block per cell (matches T4-followup exactly) | the read/validate hold is the parse leaf's `String`-per-cell — **M4's prize** |
| `dhat_read.rs`, typed-build stage, 25 MB rung | KB-scale, 27 blocks | the Arrow build holds ~nothing — the typed output itself (~1×) is the only retained slice, and it IS the product |
| `heap_profile.rs`, the #790 TREL workload, autofix | **8.4× its output, 66.5 bytes/cell** (the #790 ladder's endpoint, reproduced) | the Arrow-door emit slice is per-cell-bound: every group's formatted `OwnedGroup` plus the parse-back live together — **was M2's attributed mechanism**; superseded on the lane's wide rungs by #848's probe (the slab is freed before t-gmax there — the M2 row above carries the correction), and surviving as the dense-cell worst case the TREL shape represents |

What the pass changed: M2's "needs a dhat attribution first" is met, and its
mechanism cell was **corrected** — the "emit adds ~nothing over its input"
claim belonged to the compat stream door (M3's evidence), not the Arrow
door. M4 entered the queue: the same span rewrite the time campaign declined
at ~9.9 ms is worth roughly half of every read-shaped operation's peak on
the memory axis. Working order after #831: **M4 → M2** (M1 declined by the
diagnosis below, which also added M5 to the queue), one minted ticket at a
time.

### The 2026-08-31 M1 diagnosis (#831): the release is real, darwin never takes the pages back

The ticket's own first step — "a probe confirming each group's Arrow buffers
actually return on release" — failed, and the failure re-prices the whole
memory queue's strategy on this machine. Everything here is the **diagnosis
instrument family** (never the lane's cross-library table): `ps`-RSS
step-probes, mimalloc's own exit stats, one raw-`madvise` syscall probe, and
per-op `ru_maxrss` children running loop-equivalent code. The pair ran the
**whole memory ladder** (5/25/100/265 MB): the held-loop children reproduce
the lane's committed compat cells within 0.3% at the top three rungs (~2%
at 5 MB, where the import floor dominates) — the instrument-stability check
the A/B/A protocol wants — and the released-loop children sit on top of
them within 0.4% at every rung. The harness itself was not re-run: nothing
landed, so the committed cells still describe the shipped code, and there
was no B to pair.

The finding has three layers, each verified separately:

1. **The release works.** Dropping each group's `table` frees the Arrow
   buffers to the wheel's mimalloc: after a full release, a second parse
   lands almost entirely in the freed pages instead of growing the process
   (heap-reuse probe), and a forced `mi_collect(true)` reports the whole
   freed hold purged in mimalloc's own stats. The seam-read mechanism in
   M1's row was correct; [[arrow-c-ffi-allocator-ownership]]'s
   producer-frees contract held exactly.
2. **The purge is invisible.** mimalloc's purge decommits with
   `MADV_FREE_REUSABLE` on darwin — and on this machine (Darwin 25.4) a raw
   syscall probe shows both `MADV_FREE` and `MADV_FREE_REUSABLE` succeed
   while leaving `ps`-RSS and `ru_maxrss` exactly where they were; a second
   large mapping then takes both to the sum. Absent system memory pressure,
   the kernel keeps madvised pages resident and gives new allocations fresh
   ones. **Only `munmap` moves the instrument**, and mimalloc munmaps only
   whole-free non-arena segments: env-forcing `MIMALLOC_ARENA_RESERVE=0`
   recovers roughly a quarter of the hold at the 25 MB rung (fragmentation
   keeps segments partial) and is in any case an allocator-config decision
   in #294/#448's territory, not a compat-loop edit.
3. **So the sum-peak is physical, not an accounting artefact.** The frames
   are allocated by numpy/CPython's allocator and cannot reuse mimalloc's
   retained pages, so held/released/released-plus-forced-purge peaks are
   indistinguishable. A 12-line native hook exposing `mi_collect(true)` was
   built, probed and reverted in the same session.

What still bounds the axis: a parse-only child peaks at **~8.4×-of-file at
the 265 MB rung** (~9.2× at 100 MB) before any frame exists — the
ParsedFile `String`-per-cell hold (M4's row, ~6.5× the input by dhat)
co-resident with the Arrow build — so even a perfectly-collected release
would leave compat read near **baseline parity (~8.6× against the 8.0×
baseline and today's 12.8×)** at the headline rung: the gap to baseline
IS the double-hold, and this OS will not give it back. The operative rule this diagnosis
leaves behind: **on this instrument, only allocations that are never made
can be claimed** — avoidance (M4, M5) over release. Where the OS does
reclaim — Linux purges with `MADV_DONTNEED`, darwin under real memory
pressure prefers reusable pages — the per-group release genuinely pays;
both ends of that chain are verified but the composition is unmeasured
here. The fork was put to the owner on #831 and resolved the same day:
**measure where users run before landing** (most are hosted Linux or
Windows) — that measurement is #833, and the probe instrument it runs is
`tools/perf_probe_m1.py` via the dispatch-only `perf-probe` workflow.

### The 2026-09-01 M2 re-price (#848): the write peak carries the parse hold twice, on co-peak shoulders

The owner-directed check-don't-estimate round, probe-first like #831 and
spike-gated like #838. Everything in it is the **diagnosis instrument
family** (fresh-child `ru_maxrss` variant children + dhat at t-gmax, never
the lane's table); the full record — variant ladders, at-peak
attributions, the spike's A/B/A with byte-identity across every leg —
lives on #848. What it settled:

- **The attribution.** Post-M4, on the lane rungs, the formatted
  `OwnedGroup` slab is *freed before the validating parse-back runs* (the
  #790 drop discipline held), and the peak instead holds **two
  `ParsedFile`s** — the caller's retained parse and the emit door's
  re-parse of its own output — beside the typed input and the output
  bytes. The AutoFix tail contributes ~nothing on this workload (the
  fixture yields no safe fixes, and the tail reuses the freed slab).
- **The co-peak lesson** (the #834 lesson, sharpened): a slice's at-peak
  *share* is not its contribution. The spike removed each shoulder alone
  and then both: the tail-skip alone moved the peak almost nothing, the
  streamed slab alone landed under the invasive floor, and **only the
  pair cleared rule 10 at the 100/265 MB gate rungs**. Price shapes by
  variant children, alone *and* paired.
- **The split verdict** (grilled with the owner, 2026-09-01): the
  mechanism divides into the layout row — **M6, minted as #850**
  ([[dec-parse-structure-layout]]) — and M2's streamed door + writer-built
  verdict ([[dec-emit-streamed-verdict]]), sequenced M6-first with M2's
  ticket minted only after a re-price on the post-layout lane. Gate rungs
  for the write row are 100+265 MB; the 25 MB cell is recorded as a
  structural shortfall (the import floor's share of that rung's
  denominator), never skipped silently. (M6's spike subsequently missed
  its own gate; the owner landed the layout anyway at the measured prize —
  the M6 queue row carries the numbers and the waiver.)
- **Caller-side levers stay out of the queue**: dropping the read handle
  once the frames exist, and feeding the door capsule-bearing tables
  instead of materialised frames, are measured savings on #848's variant
  table — but they are the caller's allocations. They ship as a docs
  recipe on the write door's page, with this note as the ledger's record.

### The node lane (#823, 2026-09-01)

The first surface lane on the cross-surface matrix (epic decision 2: our own
ladder, **no external baseline**).
`rust-packages/laterite-node/bench/perf-matrix.mjs` (`npm run bench:matrix`)
drives the npm package's **public API** — napi marshalling and arrow-js
decode included, because that is what a Node caller pays — over the same
pinned ladder, and writes the same schema-2 per-surface file as
[[laterite-ags4-perf]] for `tools/perf-matrix.py` to merge. The rust column
below is the same day's run of the rust bin at the same tree (post-M6/M2),
so the pairs are attributable; both instruments are the lane's own (wall
medians over 10 iters, fresh-child peak RSS).

| op | 100 MB (node / rust) | 265 MB (node / rust) |
|---|---|---|
| validate | 134 / 131 MB/s · 4.60× / 3.99× | 141 / 132 MB/s · 4.07× / 3.68× |
| parse-to-typed | 277 / 305 MB/s · 5.37× / 3.47× | 286 / 310 MB/s · 4.85× / 3.34× |
| write | 107 / 97 MB/s · 10.43× / 5.09× | 106 / 99 MB/s · 10.25× / 5.01× |

What the lane says:

- **Time is at parity with the engine.** Validate and write sit inside the
  drift band on both sides of rust (same engine underneath — a node "win"
  here is run-to-run spread, not a finding); parse-to-typed pays ~8–10% for
  the IPC hop + arrow-js decode. No time candidate clears any floor.
- **Memory is where the boundary lives.** Validate's premium is ~the node
  runtime floor (the 5 MB rung prices the floor: ~98 MB total vs rust's
  ~29 MB — rank from the big rungs, as ever). The typed read holds
  ~1.5×-of-input more than rust's — the floor plus the retained IPC bytes
  (the napi boundary has no capsule analog; the `Buffer`s ARE the boundary).
  The write door's increment over the same lane's typed hold is
  **5.40×-of-output vs the rust door's 1.67×** — that delta is the **M7**
  queue row, mechanism read in the code, unpriced. (The doors are not quite
  twins — node's dictionary-fills UNIT/TYPE where the rust leg carries the
  source's — but that difference is per-heading metadata, nowhere near
  ×-of-file scale.)
- **Every cell measured to the cap** — the 265 MB rung included, no refusals,
  no deaths; the ladder's default stops there (rule 11), so the file carries
  no 524 MB row at all rather than a silent gap.
- **The swap watch fired once, honestly, at the wrong target.** The lane's
  first full run refused all three 265 MB cells as `swapped` — and fresh
  re-probes of the same children measured cleanly, within 0.1% of the later
  committed cells. The pager's load was the *harness parent* (a V8 process
  that has just run the timed loops keeps gigabytes resident; nulled refs
  return no pages), so the harness now runs the whole memory pass FIRST,
  while the parent holds nothing beyond the manifest. The rust bin can
  interleave because its parent genuinely frees; a GC'd host cannot — the
  wasm/CLI lanes (#824/#825) should inherit the mem-first ordering where
  their parent is GC'd too. (Both did — the wasm lane in the next section,
  the CLI lane in the one after; the CLI parent holds no rung data either
  way, so there the ordering is kept for comparability, not survival.)

### The wasm lane (#824, 2026-09-02)

The second surface lane on the matrix (epic decision 2: our own ladder, **no
external baseline**). `web/bench/perf-matrix.mjs` (`npm run bench:matrix`
from `web/`) drives the browser cdylib's **public API** — the `wasm-full`
build the app loads, instantiated from its `.wasm` bytes, the JS→wasm input
copy inside the measured path because a browser pays it — over the same
pinned ladder, writing the same schema-2 per-surface file for
`tools/perf-matrix.py` to merge (`wasm-read.bench.ts` stays the regression
guard; this is the comparison lane). It inherits the node lane's mem-first
ordering, and its memory column is **rule 13's instrument** — linear-memory
high-water in a fresh instantiation, verified byte-identical run to run —
which is why the memory table below carries no rust column: the claims
differ, so they do not share one.

**Time** (wall medians over 10 iters; the rust column is the same session's
run of the rust bin at the same tree, so the pairs are attributable — with
two named skews. Each lane measures its surface's *default* call shape, and
the wasm `validate` default has the warnings tier on where the rust bin's
`CheckOptions::default()` has it off, a tier T5 priced at ~2% at scale. And
the timed pass drives **one shared instance across the ladder** — the same
one-process shape the rust/node lanes' timed loops run in, kept for
comparability — but wasm linear memory never shrinks, so earlier rungs'
allocator state carries forward in a way the sibling heaps' can partially
give back; the memory table is immune, every cell a fresh child, and a rung
that outgrows the shared instance is a recorded skip in the artifact, never
a lost run):

| op | 100 MB (wasm / rust) | 265 MB (wasm / rust) |
|---|---|---|
| validate | 112 / 129 MB/s | 117 / 132 MB/s |
| parse-to-typed | 296 / 303 MB/s | 302 / 311 MB/s |
| write | 72 / 96 MB/s | 75 / 98 MB/s |

**Memory** — the lane's own labelled claim (linear-memory high-water,
×-of-output; engine-side only, JS holds out of claim):

| op | 5 MB | 25 MB | 100 MB | 265 MB |
|---|---|---|---|---|
| validate | 4.97× | 4.41× | 4.26× | 4.62× |
| parse-to-typed | 4.36× | 3.98× | 3.85× | 4.22× |
| write | 7.99× | 4.98× | 5.16× | 6.17× |

What the lane says:

- **parse-to-typed runs at native parity across the whole ladder** (within
  ~2–3% at the top rungs) — the boundary copy in and the per-group Arrow
  IPC back are absorbed at scale, the July 25 MB finding now held on every
  rung. No candidate.
- **validate trails rust by ~11–13%** at the big rungs — but the pair is
  the one skewed cell (the warnings-tier default above, plus
  bytes-in-memory vs the rust bin's path door), so the residual is smaller
  than it reads and **unattributed**. Nothing here clears a floor until a
  diagnosis round says where it lives.
- **write is the lane's one visible time gap: ~25% behind rust** at every
  rung. Plausible mechanisms are all boundary- or allocator-shaped (the
  whole input's IPC crossing at the call, the document string crossing
  back, dlmalloc where the native artifacts run mimalloc — the wasm
  allocator probe kept dlmalloc for the *read* path), but none has been
  read against this measurement — **unattributed, recorded, no queue row
  minted** (the #848 rule: variant children before any mint).
- **Every cell measured to the cap** — twelve cells, no refusals, no
  deaths; the top cell (265 MB write) tops out well inside wasm32's 4 GB
  address ceiling, so the ladder fits the surface.
- **The write door's increment over the same lane's typed hold** (same
  instrument, so the subtraction is legitimate) is **~1.3×-of-output at
  100 MB and ~1.9× at 265 MB** — the same ballpark as the rust door's
  peak-RSS increment and nothing like the node door's M7 stack, which is
  consistent with the doors' shapes (the wasm door takes per-group IPC
  directly; node's slabs the whole file's IPC and re-decodes) but is a
  cross-instrument reading and stays an observation, not an attribution.
- **x-of-output rises from the 100 MB rung to the 265 MB rung on every op**
  (e.g. validate 4.26× → 4.62×) where the rust lane's RSS falls. A fresh
  instance underlies every cell, so this is real per-op growth behaviour
  on this instrument — linear memory never shrinks, and dlmalloc's growth
  requests scale with the workload — recorded as the instrument's own
  signature, unattributed.

### The CLI lane (#825, 2026-09-02)

The last surface lane on the matrix (epic decision 2: our own ladder, **no
external baseline**). `tools/perf-cli.py` drives an explicitly named release
`lat` binary — `--lat-bin` / `$LAT_BIN` / this checkout's build, **never
`PATH`** (three programs answer to `lat`), the resolved path and version
recorded in the artifact — over the same pinned ladder, writing the same
schema-2 per-surface file for `tools/perf-matrix.py` to merge. Every cell is
one **fresh subprocess, spawn included** (that is what a CLI caller pays),
which makes this the campaign's simplest surface for the peak-RSS
instrument: the measured child IS `lat`, reaped per-child via `os.wait4`.
The shared memory contract (cap, refusal vocabulary, swap watch) is
**imported from the python lane's harness rather than copied** — #865's
drift class — and the memory pass still runs first, kept for comparability
rather than survival (this parent never holds rung data).

`validate` is the shared axis (the same rule engine end-to-end, the same
path door as the rust bin's `check_file`), so it joins that block with two
named skews: spawn + report rendering ride inside the time, and the CLI's
surface default keeps the warnings tier ON where the rust bin's has it off.
The read and write doors are the CLI's own and carry their **own op names**
plus a per-row `door` string — `read` (`lat read <rung> GEOL --csv --out`,
the bulk group picked per rung by streaming the file, GEOL on every rung of
this scaffold) renders strings, not typed Arrow, and the write door **as
exposed** is `merge` (`lat merge <rung> <rung> --out`, self-merge): the one
verb that drives the emit engine, since `fix` on a clean file returns the
source verbatim. Folding either into `parse-to-typed`/`write` rows would
pair claims the doors do not make.

**Time** (wall medians over 10 iters; the rust column is the same session's
run of the rust bin at the same tree):

| op | 25 MB (cli / rust) | 100 MB (cli / rust) | 265 MB (cli / rust) |
|---|---|---|---|
| validate | 122 / 129 MB/s | 126 / 128 MB/s | 134 / 133 MB/s |

| CLI door | 25 MB | 100 MB | 265 MB |
|---|---|---|---|
| read (strings → CSV on disk) | 183 MB/s · 11.68× | 194 MB/s · 11.20× | 201 MB/s · 10.92× |
| merge (the write door as exposed) | 27 MB/s · 16.62× | 25 MB/s · 15.50× | 24 MB/s · 16.03× |

(The ×-figures are peak RSS ×-of-output; merge's emitted size came back
byte-count-equal to the input at every rung — the self-merge's KEY dedup
held, no row doubling — so its ×-of-output is ×-of-input too.)

What the lane says:

- **validate is at engine parity from the 100 MB rung** — the spawn +
  report-render premium is ~10% at 5 MB and inside the drift band above it
  (the 265 MB cell came in ahead of the same-session rust run; the two
  named skews are below the instrument's resolution at scale). Peak RSS
  carries a ~0.2×-of-input premium over the rust bin's cells (3.83× vs
  3.66× at 265 MB) — the process + report floor, washing out exactly as the
  small-rung factors predict. No candidate.
- **The read door holds ~11×-of-input across the ladder** (13.77× at 5 MB
  → 10.92× at 265 MB) against the engine's ~3.3× span-parse hold on the
  same tree — and the mechanism is read, not guessed: `read_ags4_with`
  re-materialises every group's rows as `HashMap<Arc<str>, String>` — one
  owned `String` per cell plus per-row map overhead
  (`laterite-ags4-core/src/ags4_codec.rs`, `AgsGroup.rows`) — over the
  span `ParsedFile` it just built, and the CLI then projects the dumped
  group into a third copy (`Vec<Vec<String>>` + the rendered text,
  `laterite-cli/src/commands/read.rs`). **Minted as M8 in the memory
  queue, unpriced** — the #848 co-peak rule applies before any prize is
  claimed. Throughput *rises* with the rung (158 → 201 MB/s) as the fixed
  spawn cost amortises.
- **The merge door is the ladder's biggest hold on any lane: ~16×-of-output,
  ~24–27 MB/s, flat across rungs** (4.4 GB peak at 265 MB — the swap watch
  stayed quiet; the rung cap admits by *input* bytes, so a door-side
  multiplier like this is exactly what the recorded cell is for). The
  readable floor is two whole-file parse holds (~3.3× each — merge parses
  both arguments); the remaining ~9×-of-output (the union's owned
  group/cell structures plus the emit leg) is **unattributed, recorded, no
  queue row** — variant children before any mint (#848), and a self-merge
  is the door's degenerate case, not its economics.
- **Every cell measured to the cap** — twelve cells, both instruments, no
  refusals, no deaths, `skipped` empty.

## Decisions taken

Recorded here so the next session does not re-open them. Each was a genuine fork
with consequences both ways; none is a default that fell out of the code.

**The M1 fork — measure where users run, land on evidence (owner,
2026-08-31, grilled).** #831's diagnosis left a fix that is real but
invisible to the ledger machine's instrument. Rather than landing it blind
or burying it, the owner chose to measure it on the OSes users actually run
— hosted Linux and Windows — before deciding (#833): a trimmed go/no-go
probe against the campaign's own 5% floor, on the self-hosted Linux pool
plus GitHub-hosted `windows-2022` (`ags5-win11` stays release-only; in
reserve only if hosted Windows proves unworkable). Cross-OS numbers are a
**third labelled claim family** that never shares a table with the darwin
lane. Paired with it, a measurement-honesty rule: the lane's results file
now names WHICH pandas hop its compat cells measured, because the dev
venv's pyarrow silently put the accelerated hop in the table while the
shipped `[compat]` default takes the DuckDB hop. And a scoped non-decision:
the pyarrow-free `[compat]` default is NOT being re-opened — M5 (#834)
gathers the evidence, and only a recorded finding that the shipped hop
still trails substantially puts the dep shape back on the owner's desk.
(That finding landed 2026-08-31: the trigger fired — the fix left the
shipped hop ~+18–19% of the operation's peak behind the pyarrow hop at
the top rungs; the record is on #834 and the M5 queue row.)

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
- **On every read surface** — `lat read`, `laterite-ags4-excel`, node,
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
given size+seed, and carrying no real delivery data), plus the two clean-isolated
dirty twins (`dirty-r16`, `dirty-r10c`) the T5 emission benches consume.
`tools/bench-vs-python-ags4.py` reproduces the README's comparison tables, with
the fixtures SHA-pinned so generator drift fails loudly instead of quietly moving
the numbers.

Per-surface read harnesses mirror the Rust rungs on the same 25 MB fixture, each
in its own lane (not part of the unit suite): `laterite-node/bench/read.bench.ts`
(`npm run bench`) and `web/bench/wasm-read.bench.ts` (`npm run bench:wasm`, driving
the built browser cdylib from its `.wasm` bytes). Since #823 the node surface also
carries the cross-surface **matrix lane**, `laterite-node/bench/perf-matrix.mjs`
(`npm run bench:matrix`): the campaign's three ops over the whole forge ladder,
time plus the fresh-child peak-RSS instrument, emitting the same uniform
per-surface schema as `laterite-ags4-perf` for `tools/perf-matrix.py` to merge
(the read bench stays the regression guard; the matrix lane is the comparison).
The wasm surface's matrix lane is `web/bench/perf-matrix.mjs` (#824, `npm run
bench:matrix` from `web/`, needing the built `wasm-full` artifact): the same
three ops through the browser cdylib's public API, time plus the lane's own
linear-memory instrument (rule 13). The CLI's matrix lane is
`tools/perf-cli.py` (#825, `uv run python tools/perf-cli.py`, needing a
release `lat` it is explicitly pointed at — `--lat-bin` / `$LAT_BIN` / this
checkout's build, never `PATH`): one fresh subprocess per cell for both
instruments, `validate` on the shared axis plus the CLI's own read and
write doors under their own op names. For attribution rather than
timing, `laterite-ags4-types/examples/dhat_read.rs` (dev-only, `arrow`-gated) is a
`dhat` heap profile of the read stages — it says whether a stage is
allocation-bound (fixable) or a compute/bandwidth wall; its write-side sibling is
`laterite-ags4-emit/examples/heap_profile.rs`.

Since #821, `tools/bench-vs-python-ags4.py` is also the **memory harness** for
the Python lane (rule of engagement 8) and every run writes the machine-readable
record the ledger's memory tables summarise —
`tools/perf-results/python-lane.json`, whose schema the tool declares in its
docstring.

> [!note] An absent fixture SKIPS rather than fails, so a clean checkout still
> works — but a skipped bench measures nothing, which is exactly how the
> pre-2026-07 bench sat silently dead on every machine but one.

## Related

[[core-perf-baseline]] — the findings this campaign produced.
[[abi3-perf]] — the binding's cost, measured separately.

[[core-perf-baseline]] · [[abi3-perf]] · [[testing-strategy]] · [[coverage-campaign]] · [[crate-map]] · [[reliquary]]
