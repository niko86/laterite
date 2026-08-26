# python-ags4 / AGS4-spec observations log

A running record of discrepancies, ambiguities, and apparent
defects found while clean-room-porting the AGS4 validator. Two
audiences:

1. **Us** — decisions on how the Rust validator should behave when
   python-ags4 and the spec disagree (we follow the spec; deviations
   are deliberate and noted).
2. **Upstream** — items worth reporting to the AGS Data Format Working
   Group (python-ags4 bugs) or flagged as AGS4-spec ambiguities.

Each entry: what was observed, where, spec reference, our decision,
and whether it's upstream-reportable.

Spec references: `reports/AGS 4_1.pdf` (Edition 4.1, Dec 2020) and
`reports/AGS 4_2.pdf` (Edition 4.2, Dec 2025) — §4.1.1 CSV File Rules.

Severity tags: **[BUG]** likely python-ags4 defect · **[SPEC]** spec
ambiguity/contradiction · **[VARIANCE]** intentional impl-vs-spec
divergence (not a defect) · **[NOTE]** behavioural observation.

---

## Upstream-reportable — the external-facing set

The curated subset worth reporting to the AGS Data Format Working Group —
python-ags4 defects (**[BUG]**) and AGS4-spec ambiguities/contradictions
(**[SPEC]**), plus the intentional variances flagged for AGS-DFWG. Every other
O-N below is an internal decision or behavioural note, not for external circulation.

| O-N | Kind | The case |
|---|---|---|
| O-1 | VARIANCE | Rule 1 "entirely ASCII" is not enforced literally |
| O-2 | BUG | Rule 6 is a complete no-op in python-ags4 |
| O-4 | SPEC | "HEADING row missing" attributed to Rule 4, not Rule 2b |
| O-6 | SPEC | Rule 19 spec prose is looser than the AGS dictionary |
| O-7 | SPEC | Rule 19b_1's field-length limit isn't in the prose (but the dict obeys it) |
| O-8 | BUG | python-ags4 rule_7_2 can raise IndexError on duplicate headings |
| O-11 | SPEC | python-ags4 folds ID-uniqueness into Rule 8 (it's Rule 10a's job) |
| O-17 | SPEC | Rule 18 keys off heading membership only, not GROUP names |
| O-54 | SPEC | Rule 16a — implemented under Rule 16's id, but neither engine applies the spec's default concatenator |
| O-21 | SPEC | Rule 10c's parentless-group list is hardcoded, not dict-derived |
| O-30 | VARIANCE | TRAN_AGS-driven edition selection — deliberate divergences from python |
| O-31 | VARIANCE | Rule 8 — empty `DT` UNIT now flagged (python parity; closes the O-12 degenerate gap) |
| O-32 | VARIANCE | Non-UTF-8 input is decoded lossily, not refused (mirrors python's `errors="replace"`; closes the `NotUtf8` black hole) |
| O-33 | VARIANCE | Rule 8 — DT/datetime bounded to pandas' Timestamp range (closes the value-range gap O-12 missed) |
| O-34 | VARIANCE | `NotAgs4` ↔ python "missing mandatory groups" is a KNOWN_DIVERGENCE |
| O-38 | SPEC | Rule 8 DT validation: python-ags4 forbids non-ISO UNITs |
| O-39 | SPEC | Rule 10c — empty parent KEYs are "no entry", not a missing link |
| O-42 | VARIANCE | TRAN_AGS="4.0" resolves to 4.0.4 (superset-safe), with a content guard; python's static "4.0"→4.0.3 over-reports Rule 10c |
| O-52 | VARIANCE | A DECLINED Rule 10c parentage check is reported — an all-empty parent-KEY child row produces a WARNING naming what could not be checked; python-ags4's silence there is a coincidence of its UNIT/TYPE pseudo-rows, not a decision |
| O-49 | VARIANCE | A numeric TYPE's count (the n in nDP/nSF/nSCI) is read uncapped from the file and fed into a format width — a crafted "9999999999SF" OOMs python-ags4 (~10 GB string); laterite now clamps to 30 |

## V1 — line-level rules (Rules 1, 3, 5, 6)

### O-1 [VARIANCE] Rule 1 "entirely ASCII" is not enforced literally
- **Spec** (4.1 §4.1.1, 4.2 unchanged): *"The data file shall be
  entirely composed of ASCII characters."* ASCII is code points 0–127.
- **python-ags4** (`check.py::rule_1` + `is_ags_ascii`): treats code
  points **0–255** as acceptable; 128–255 is downgraded to an *FYI*
  ("Has extended ASCII character(s)"), only > 255 is a Rule 1 error.
  `is_ags_ascii` docstring explicitly redefines "extended ASCII" as
  ≤ 255.
- **Assessment**: a deliberate, documented relaxation — real
  geotechnical data is full of `°`, `±`, `µ` (Latin-1, 176/177/181).
  Enforcing the literal spec would make the validator unusable. Not a
  bug, but it *is* an undocumented divergence from the written
  standard.
- **Upstream-reportable**: **[SPEC]** — the AGS4 spec text should
  either (a) permit ISO-8859-1/Latin-1 explicitly, or (b) acknowledge
  the de-facto tolerance. The current wording vs. the reference
  implementation contradict each other.
- **Our decision**: match python-ags4's interpretation (>255 = error;
  128–255 = FYI suppressed unless `include_fyi`). Documented in
  `rules/line_format.rs`.

### O-2 [BUG] Rule 6 is a complete no-op in python-ags4
- **python-ags4** (`check.py::rule_6`): body is literally
  `return ags_errors` with the comment *"This will be satisfied if
  rule_2a, rule_4b and rule_5 are satisfied."*
- **Spec** (§4.1.1 Rule 6): *"…separated by a comma (,). No carriage
  returns (ASCII 13) or line feeds (ASCII 10) are allowed in or
  between data VARIABLEs within a DATA row."*
- **Assessment**: the claim that 2a/4b/5 fully subsume Rule 6 is
  **not airtight**. A field containing a bare embedded CR that does
  *not* break the quoting grammar (e.g. `"a<CR>b"`) passes Rule 5
  (quotes balanced) and Rule 4 (field count unchanged) yet violates
  Rule 6's explicit CR/LF prohibition. python-ags4 would not catch it.
- **Upstream-reportable**: **[BUG]** — Rule 6 should at minimum scan
  for embedded CR/LF inside quoted fields rather than no-op.
- **Our decision**: implement the independent embedded-CR check
  python-ags4 skips (`rules/line_format.rs::rule_6`). A *better*
  implementation, not a port.

### O-3 [NOTE] Rule 5 enclosure check special-cases DATA rows
- **python-ags4** (`check.py::rule_5`): the comma-count cross-check
  (`len(split('","')) != len(split(','))`) is applied to HEADING /
  UNIT / TYPE rows but **deliberately skipped for DATA rows**, because
  a legitimate `X`-type DATA value may contain commas. python-ags4
  relies on Rule 4b (field count vs HEADING) to catch unquoted DATA
  fields instead.
- **Assessment**: reasonable design, but it means an unquoted DATA
  field is reported under *Rule 4*, not *Rule 5*, in python-ags4 —
  a finding-attribution difference a naive parity check would trip on.
- **Upstream-reportable**: no — behavioural, by design.
- **Our decision**: our `well_quoted` grammar checks DATA rows
  strictly too, so we may attribute an unquoted DATA field to Rule 5
  where python-ags4 attributes it to Rule 4. Parity harness must
  compare *total* finding presence per file, not per-rule line-exact,
  for this class. Noted for the V-parity cross-check design.

---

## V2 — group-structure rules (Rules 2, 2a, 2b, 4)

### O-4 [SPEC] "HEADING row missing" attributed to Rule 4, not Rule 2b
- **Spec** (§4.1.1 Rule 2b): *"As a minimum, the GROUP HEADER rows
  comprise GROUP, HEADING, UNIT and TYPE rows presented in that
  order."* By the plain text, a missing HEADING row is a **Rule 2b**
  violation (HEADING is one of the four mandated header rows).
- **python-ags4**: `rule_2b` only checks UNIT-missing / UNIT-misplaced
  / TYPE-missing / TYPE-misplaced. A missing HEADING is reported by
  `rule_4_2` instead, as *"Headings row missing."* under **Rule 4**
  with line `'-'`.
- **Assessment**: defensible (rule_4_2 needs the headings list anyway,
  so it's a convenient place to detect absence) but it's a
  spec-vs-impl attribution mismatch — a strict reading puts
  HEADING-presence under 2b.
- **Upstream-reportable**: **[SPEC]** — minor; worth noting to
  AGS-DFWG that the reference impl files a 2b-class defect under 4.
- **Our decision**: follow python-ags4's attribution (Rule 4, line
  `None`) so per-rule finding counts line up for the parity harness.
  Documented in `rules/structure.rs`.

### O-5 [NOTE] HEADING-less groups: we keep validating, python-ags4 may not
- **Observed**: our parser retains a group even if its HEADING row is
  absent (empty `headings`), so Rules 2/2a/2b/4 still evaluate it.
  python-ags4's table-scan rules (`rule_2`, `rule_2b`) iterate
  pandas `tables` built around the HEADING row; a HEADING-less group
  may not form a well-shaped table, so those rules could under-report
  on it (its only guaranteed finding is `rule_4_2`'s "Headings row
  missing."). `rule_4_2` *does* fire (it's per-line), so the group
  isn't entirely silent.
- **Assessment**: **unverified** — confirming requires reading
  `AGS4.py`'s table construction, which the clean-room boundary keeps
  to "understand, don't copy". Flagged as a robustness *difference*,
  not asserted as a bug.
- **Upstream-reportable**: not yet — needs substantiation first. If a
  later phase confirms HEADING-less groups escape the table-scan
  rules, promote to **[BUG]**.
- **Our decision**: retain + fully validate malformed groups; it's
  strictly more thorough and costs nothing.

---

## V3 — name-format rules (Rules 19, 19a, 19b)

### O-6 [SPEC] Rule 19 spec prose is looser than the AGS dictionary
- **Spec** (§4.1.1 Rule 19): *"A GROUP name shall **not be more than 4** characters long and shall consist of **uppercase letters and numbers** only."*
- **python-ags4** (`check.py::rule_19`): flags when `len(name) != 4 or not name.isupper()` — relative to the prose this is too strict on length (`!= 4` vs "not more than 4"), rejects all-digit names, and never enforces the `[A-Z0-9]` charset (so `LO-A` passes).
- **Evidence** (bundled dicts): 0 of the ~319 standard groups are ≠ 4 chars, contain a digit, or are non-uppercase — the dictionary never exercises any of the prose's stated allowances.
- **Assessment**: revised [BUG] → **[SPEC]**. python-ags4 enforces the *de-facto* rule ("exactly 4 uppercase letters") the dictionary universally follows, so it never misfires on real data; the real issue is that the **spec prose and the dictionary disagree** — the prose permits names the format never uses.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG tighten the Rule 19 wording to "exactly 4 uppercase letters" so prose ⇄ dictionary ⇄ validators agree; the looser allowance is dead text.
- **Our decision**: enforce the convention the dictionary follows — **GROUP name = exactly 4 uppercase letters `[A-Z]`** (python's effective behaviour, 319/319 real groups); the looser prose is recorded here as the upstream shortcoming.

### O-7 [SPEC] Rule 19b_1's field-length limit isn't in the prose (but the dict obeys it)
- **Spec** (§4.1.1 Rule 19b): *"HEADING names shall start with the GROUP name followed by an underscore character. e.g. 'NGRP_HED1'."* No constraint on the field part beyond Rule 19a's overall ≤ 9.
- **python-ags4** (`rule_19b_1`): requires `len(split('_')[0]) == 4` **and `len(split('_')[1]) <= 4`** — the second clause is a ≤ 4-char field-name limit found nowhere in the prose.
- **Evidence** (bundled dicts): of ~4199 real headings, 0 have a field part > 4 chars and 0 lack an underscore — the dictionary silently obeys an informal rule the prose doesn't state (as in O-6).
- **Assessment**: python-ags4's ≤ 4 field clause never misfires on real data — an undocumented but accurate encoding of the convention. The prefix==GROUP deferral is reasonable (it needs the dict + the borrowed-heading exception, e.g. `FILE_FSET` inside non-FILE groups).
- **Upstream-reportable**: **[SPEC]** — same recommendation as O-6: AGS-DFWG should state the field-part ≤ 4 convention explicitly in Rule 19b.
- **Our decision**: 19b_1 enforces the convention the dictionary follows — a 4-uppercase-letter prefix + `_` + a 1–4 char `[A-Z0-9]` field part (4199/4199 real headings obey it); the prose's silence is recorded above. The prefix==GROUP / cross-group-borrow semantic is not 19b_1's: it is handled by the dict-aware `19b_2`/`19b_3` checks in `rules/references.rs`, matching python's split.

## V4 — dictionary-aware rules (Rules 7, 9)

### O-8 [BUG] python-ags4 rule_7_2 can raise IndexError on duplicate headings
- **python-ags4** (`check.py::rule_7_2`): builds `temp` = the dictionary heading list filtered to those used (inherently de-duplicated), then iterates the *file's* heading list and indexes `temp[i]` unconditionally. A duplicate that makes the file list longer than `temp` without an earlier `!=` break (e.g. file `[A, B, B]` vs dict `[A, B, C]`: i=2 → `temp[2]`) raises `IndexError` and aborts the whole check run.
- **Spec** (§4.1.1 Rule 7): mandates dictionary order only; says nothing about duplicates (the duplicate check is itself inferred — O-9).
- **Assessment**: a latent defect — the unguarded `temp[i]` *can* `IndexError`. But under default python-ags4 (`rename_duplicate_headers=True`, what `ags4 check` uses) a duplicate HEADING is renamed to `<NAME>_1` before `rule_7_2`, so the subset test fails first and the bad index is never reached. Reachable only with the non-default `rename_duplicate_headers=False`.
- **Upstream-reportable**: **[BUG]** — `rule_7_2` should still bound-check `temp[i]`; latent but real (toggling the rename default exposes it).
- **Our decision**: `rule_7_2` is bounds-guarded — if the used-heading list is longer than the de-duplicated expected list we stop cleanly (the duplicate is the actionable finding, already raised). Defensive against a bug python's own default currently shields.

### O-9 [NOTE] Rule 7's "no duplicate headings" is inferred, not in the prose
- **Spec** (§4.1.1 Rule 7): *"…HEADINGs shall be in the order
  described in the AGS FORMAT DATA DICTIONARY."* No explicit
  prohibition on a HEADING row repeating a field name.
- **python-ags4** (`rule_7_1`): independently flags
  `len(headings) != len(set(headings))` as *"HEADER row has duplicate
  fields."* under Rule 7.
- **Assessment**: defensible — a repeated HEADING name makes "the
  order described in the dictionary" and per-field UNIT/TYPE/DATA
  alignment ill-defined, so the constraint is implied by Rules 4/7/8
  read together. But it is an interpretation, not literal text.
- **Upstream-reportable**: **[NOTE]** — minor; the AGS-DFWG could make
  the no-duplicate-HEADING requirement explicit so it isn't left to
  validators to infer (cf. O-6/O-7 — prose vs. de-facto).
- **Our decision**: keep the duplicate-heading check, attributed to
  Rule 7 with an empty group (matching python-ags4's per-line
  attribution for count parity). Documented in `rules/dictionary.rs`.

### O-10 [VARIANCE] Dictionary version: explicit option, not TRAN_AGS-derived
- **python-ags4** (`pick_standard_dictionary`): selects the standard dictionary from `TRAN_AGS`, defaulting to the latest (`4.1.1`) when absent/unknown, and raises a hard `AGS4Error` if there is neither a DICT group nor a resolvable standard dictionary.
- **Us**: `CheckOptions.dict_version` was explicit (default `4.2`); V4 validated against that bundled edition and did not consult `TRAN_AGS`. A bundled dictionary is always present, so the "no dictionary available" hard-error can't occur — a robustness improvement.
- **Assessment**: a deliberate V4 scope boundary — auto-selecting the edition from `TRAN_AGS` is Rule 14 territory.
- **Upstream-reportable**: no — implementation choice, not a spec / python defect.
- **RESOLVED (post-V8)**: superseded by **O-30** — all five editions are bundled and `check_file` auto-selects from `TRAN_AGS` (`resolve_dict_version`), an explicit `--dict-version` still overriding. The real-data dogfood (AGS 4.0 files) forced it: ~100 spurious edition-drift Rule 7/9/19b findings vanished once the right edition was used.

## V5 — typed-value rule (Rule 8)

### O-11 [SPEC] python-ags4 folds ID-uniqueness into Rule 8 (it's Rule 10a's job)
- **Spec** (§4.1.1 Rule 8): Rule 8 is about a value conforming to its declared UNIT/TYPE. Uniqueness is **Rule 10a**: *"There shall not be more than one row of data in each GROUP with the same combination of KEY field entries."*
- **python-ags4** (`check.py::rule_8`): for a column whose TYPE is `ID` *and* whose name starts with the GROUP name, it additionally flags non-unique values **under Rule 8** (`duplicated(keep=False)`). The same defect is independently reported by `rule_10a` (V7), so a duplicate group ID is double-reported under both Rule 8 and Rule 10a.
- **Assessment**: an attribution over-reach — uniqueness isn't a UNIT/TYPE property, and Rule 10a already owns it. Not wrong as a *detection*, but the rule number is misleading and it inflates the Rule 8 count.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG / the python-ags4 maintainers move ID-uniqueness wholly under Rule 10a so each defect is reported once, under the rule that actually governs it.
- **Our decision**: mirror python's attribution in V5 (flag group-prefixed `ID` duplicates under Rule 8) for finding-count parity, and re-detect under Rule 10a in V7. Documented so the intentional double-report isn't mistaken for a bug.

### O-12 [VARIANCE] DT/T validity engine differs from pandas
- **python-ags4** (`rule_8` DT/T arms): structural check via a per-char regex built from the UNIT (`fullmatch`), plus a semantic check via `pandas.to_datetime(..., format=…|'ISO8601')` (timezone offset stripped first).
- **Us** (`typed_values.rs`): identical per-char structural matcher; semantic validity via `chrono` over the AGS-permitted ISO-8601 shapes (same `Z`-strip). For an **unrecognised UNIT shape** we apply only the structural check and stay lenient on semantics (we won't invent a calendar interpretation we can't justify); pandas would still attempt a parse.
- **Assessment**: behaviourally equivalent for every UNIT the AGS dictionary actually uses (`yyyy-mm-dd`, `…Thh:mm[:ss]`, `hh:mm`, `hh:mm:ss`). Divergence is possible only for non-standard UNIT strings, where "structural-only + lenient" is the defensible choice.
- **Upstream-reportable**: no — implementation choice, no spec / python defect.
- **Our decision**: keep the lean `chrono`-based check (AGS patterns are tiny and hand-matched, no `regex` dep). **(O-33 corrects the scope: the chrono≈pandas equivalence holds only for years within pandas' Timestamp range — out-of-range dates are now bounded to match python.)**

### O-13 [NOTE] TYPE classification: exact vs python's substring test
- **python-ags4**: dispatches with loose membership — `if 'DP' in
  data_type`, `elif 'SCI' in data_type`, `elif 'SF' in data_type`.
- **Us**: exact classification — a positive-integer prefix followed by
  the precision suffix (`12DP`, `3SF`, `1SCI`), else the exact tokens
  `DT/T/U/YN/DMS/ID`.
- **Assessment**: equivalent for *every* valid AGS TYPE code. Ours
  additionally won't misclassify a hypothetical malformed code that
  merely *contains* `DP`/`SF`/`SCI`. Strictly more robust, no
  behavioural change on conformant data.
- **Upstream-reportable**: **[NOTE]** — minor; python's substring
  dispatch is fragile but never misfires on real dictionaries.
- **Our decision**: keep exact classification.

### O-14 [NOTE] nSF parity is behavioural, not source-derived (clean-room)
- The exact `format_numeric_column` python-ags4 uses for the `nSF`
  expected value lives in `AGS4.py`, **not** in the `check.py` copy
  read for this port — so it was never read. Our `nSF` expected-form
  (`format_nsf`) is ported from this workspace's own MIT
  `ags_types::ags4_str`, which was independently fitted to python-ags4
  validator *output* (`0.002` @3SF → `"0.00200"`, `1234` @3SF →
  `"1230"`, never scientific) and is pinned by tests in both crates.
- **Assessment**: the clean-room boundary held — nSF parity is by
  observed behaviour, not by translating python's formatter. Flagged
  so the provenance is explicit.
- **Upstream-reportable**: no.
- **Our decision**: keep the ported MIT algorithm; if a future case
  reveals an nSF mismatch vs python-ags4, treat it as a fixture to fit
  against, not a reason to read `AGS4.py`.

---

## V6 — mandatory / definition groups (Rules 12–18)

### O-15 [NOTE] Rule 12 is a no-op (wholly subsumed by Rule 10b)
- **Spec** (§4.1.1 Rule 12): *"Data does not have to be included
  against each HEADING unless REQUIRED (Rule 10b). The data FIELD can
  be null; a null entry is defined as \"\" (two quotes together)."*
- **python-ags4** (`rule_12`): body is a bare `return ags_errors` —
  "already checked by AGS Format Rule 10b. No additional checking
  necessary."
- **Assessment**: correct — Rule 12 is a definition + a pointer to
  Rule 10b's REQUIRED-fill check, not an independent constraint.
- **Upstream-reportable**: no.
- **Our decision**: Rule 12 emits nothing; the substantive check lands
  in Rule 10b (V7). Documented in `rules/groups.rs`.

### O-16 [NOTE] Rules 13/14 double-report a zero-row PROJ/TRAN with Rule 2
- **Observed**: a PROJ/TRAN group with no DATA rows is flagged by
  Rule 2 (V2 — "every group needs ≥1 DATA row") *and* by Rule 13/14
  ("…shall contain only one data row"). python-ags4 has both `rule_2`
  and `rule_13`/`rule_14` and double-reports identically.
- **Assessment**: defensible — Rule 13/14 specifically govern PROJ/
  TRAN cardinality, Rule 2 is the general case; the spec states both.
  Same shape as O-11 (Rule 8 vs 10a ID-uniqueness).
- **Upstream-reportable**: **[NOTE]** — minor; the duplication is in
  the standard's structure, not a defect.
- **Our decision**: keep both for finding-count parity. Pinned by
  `rule_13_14_flag_missing_proj_and_tran`.

### O-17 [SPEC] Rule 18 keys off heading membership only, not GROUP names
- **Spec** (§4.1.1 Rule 18): *"Each data file shall contain the DICT GROUP where non-standard **GROUP and HEADING** names have been included…"*
- **python-ags4** (`rule_18`): fires only when there is no DICT group **and Rule 9 already produced findings**. Rule 9 itself checks only *heading* membership (it never flags a non-standard GROUP code), so a file with a non-standard GROUP whose headings all resolve would not trigger Rule 18.
- **Assessment**: the reference implementation under-enforces the prose — "non-standard GROUP names" is in the text but nothing keys off it. In practice a non-standard group almost always carries non-standard headings, so Rule 9 fires anyway; the gap is narrow but real.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG / python-ags4 make non-standard *GROUP*-name detection explicit (its own check feeding Rule 18), rather than relying on heading fallout.
- **Our decision**: replicate python's behaviour for V6 parity — `rule_18` follows Rule 9's output. A dedicated non-standard-GROUP check is a candidate for a later phase; recorded here so the spec gap is on file.

### O-18 [NOTE] Rule 18a has no dedicated check
- **Spec** (§4.1.1 Rule 18a): the DICT order of user-defined HEADINGs
  defines their append order and their Record-Link sequence (Rule 11).
- **python-ags4**: no `rule_18a`. The ordering is enforced indirectly
  by `rule_7_2` (heading order vs the merged dict — our V4) and
  `rule_11` (Record Links — V7).
- **Assessment**: Rule 18a is a semantic that other rules enforce; a
  standalone check would be redundant.
- **Upstream-reportable**: no.
- **Our decision**: nothing extra in V6 — covered by V4's
  effective-dictionary ordering and V7's Rule 11.

### O-19 [NOTE] Rule 17 — we skip an empty TYPE cell; python would flag it
- **python-ags4** (`rule_17`): collects every TYPE-row value and
  excludes only the literal `'TYPE'` (a pandas DataFrame artefact —
  the `HEADING` column value leaks into the flattened list). An empty
  type cell (`""`) is **not** excluded, so a column with a blank TYPE
  would be reported as data type `""` "not found in TYPE group".
- **Us**: we additionally skip `""` — an empty cell is never a real
  data type; flagging it as an undefined "type" is noise (the missing
  TYPE is a Rule 4/10 concern). python-ags4 already skips `''` in
  `rule_15`, so this only differs for Rule 17.
- **Assessment**: a deliberate refinement; no behavioural change on
  conformant data (every column has a TYPE there).
- **Upstream-reportable**: **[NOTE]** — minor inconsistency in
  python-ags4 (Rule 15 skips `''`, Rule 17 doesn't).
- **Our decision**: skip `""` in Rule 17; documented in code.

### O-20 [VARIANCE] No AGS rule enforces TRAN_AGS == dictionary edition
- **Clarifies O-10.** That entry deferred "TRAN_AGS ⇄ dictionary
  consistency" to "Rule 14 (V6)". Reading the spec + `check.py`
  confirms there is **no** such rule: `rule_14` only checks TRAN
  presence + single DATA row. `TRAN_AGS` is used solely by
  `pick_standard_dictionary` to *select* which standard dictionary to
  validate against (defaulting to the latest on absence) — it is never
  itself validated for agreement with that dictionary.
- **Assessment**: O-10's "Rule 14/V6" expectation was inaccurate;
  there is nothing to implement in V6 for it. Our explicit
  `CheckOptions.dict_version` (default 4.2) remains the deliberate
  variance vs python's TRAN_AGS-driven auto-selection.
- **Upstream-reportable**: no — this note supersedes the O-10
  forward-reference; no spec/python defect.
- **Our decision**: keep explicit `dict_version`; TRAN_AGS-driven
  auto-selection stays out of scope (revisit only if a consumer needs
  python-parity dict selection). O-10's deferral target is hereby
  closed as "not a rule".

---

### O-54 [SPEC] Rule 16a — implemented under Rule 16's id, but neither engine applies the spec's default concatenator
- **Spec** (§4.1.1 Rule 16a): *"Where multiple abbreviations are required to fully codify a FIELD, the abbreviations shall be separated by a defined concatenation character. This single concatenation character shall be defined in TRAN_RCON.  The default being \"+\" (ASCII character 43)"*, and *"Each abbreviation used in such combinations shall be listed separately in the ABBR GROUP."*
- **Us** (`rules/groups.rs::rule_16`): implemented, and deliberately without an id of its own — a 16a violation is reported as **Rule 16**. The concatenator comes from the TRAN group's first DATA row; every `PA` value is split on it and each part must be defined in ABBR under that heading. An absent or empty `TRAN_RCON` is filtered to `None`, so the value is **not** split: the `"+"` default the prose names is never applied.
- **python-ags4** (`check.py` `rule_16`): the same split and the same omission. A missing TRAN or TRAN_RCON raises `KeyError`, an empty one `ValueError`; both are caught and passed over, leaving the entries unsplit. Its own comments hand the condition to Rules 14 and 11b.
- **Evidence**: one file, a `PA` cell of `"CP+RC"`, ABBR defining `CP` and `RC` separately. With `TRAN_RCON` populated as `"+"` neither engine reports a Rule 16 finding. With the heading removed both report one — ours *"Abbreviation \"CP+RC\" under SAMP_TYPE is not defined in the ABBR group."*, python-ags4 1.2.0's *"\"CP+RC\" under SAMP_TYPE in SAMP not found in ABBR group."* Under the stated default both would split and find each part defined. That extra Rule 16 finding is the SOLE difference between the two runs on our side: nothing else changes, and in particular nothing reports the missing `TRAN_RCON` itself, so the only diagnostic the reader gets names an abbreviation their file never used.
- **Assessment**: 16a's substance is implemented in both engines and neither gives it an id, which is why enumerating catalogue rule ids reads it as a gap — `ags_rules()` and `rules_meta.json` list implemented CHECKS, not spec RULES, the same trap O-18 records for Rule 18a. The one real departure is the default, and the two ways to lack a concatenator do not behave alike. An **empty** `TRAN_RCON` cell raises Rule 11b, so the reader gets the real diagnosis beside the spurious Rule 16. An **absent** `TRAN_RCON` heading raises nothing at all — it is `OTHER` status, so Rule 10b does not ask for it either — and the only finding such a file produces is a Rule 16 naming an abbreviation it never used. The second case is the one worth fixing: ignoring a stated default is defensible when something else names the fault, and misleading when nothing does.
- **Upstream-reportable**: **[SPEC]** — two independent implementations both ignore a default the prose states outright, which is stronger evidence that the sentence is doing no work than either engine would be alone. AGS-DFWG should either say the default applies when TRAN_RCON is absent, or drop it and let Rule 11b carry the requirement.
- **Our decision**: keep parity with python-ags4 — split only on a populated `TRAN_RCON` — and record the departure here rather than diverge silently. Rule 16a stays covered under Rule 16's id: a dedicated `16a` id would split one condition across two rule names for no reader benefit, which is the call O-18 makes for 18a.

## V7 — relational rules (Rules 10a–10c, 11a–11c)

### O-21 [SPEC] Rule 10c's parentless-group list is hardcoded, not dict-derived
- **Spec** (§4.1.1 Rule 10c): *"Every entry made in the KEY fields in any GROUP must have an equivalent entry in its PARENT GROUP."* The dictionary encodes the parent in `DICT_PGRP`.
- **python-ags4** (`rule_10c`): skips a **hardcoded** set — `PROJ, TRAN, ABBR, DICT, UNIT, TYPE, LOCA, FILE, LBSG, PREM, STND` — rather than deriving "parentless" from `DICT_PGRP`. The hardcoded list exists because `DICT_PGRP` doesn't fully encode *checkable* parent linkage: e.g. `LOCA`'s `DICT_PGRP` is `PROJ`, yet a LOCA row carries no PROJ key, so a dict-derived check would emit a bogus orphan finding for every file with LOCA.
- **Assessment**: the list is necessary for correctness but a maintenance hazard — a new root / implicitly-linked GROUP in a future edition would need a code change, not a dictionary update.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG make parentless / implicit-link status a dictionary property so Rule 10c is data-driven, not hardcoded.
- **Our decision**: replicate python's exact list (`PARENTLESS` in `relational.rs`) for parity + correctness; documented as the spec gap to raise.

### O-22 [NOTE] Rules 10a/10b/10c double-report against Rules 8/13/14
- **Observed**: a non-unique group ID is flagged by Rule 8 (O-11)
  *and* Rule 10a; a missing/blank REQUIRED PROJ/TRAN field interacts
  with Rules 13/14's "REQUIRED-fill" prose (deferred to 10b). These
  overlaps exist in python-ags4 too.
- **Assessment**: the spec deliberately layers the rules (10a is the
  general KEY-uniqueness rule; Rule 8's ID-uniqueness is the narrower
  python over-reach from O-11). Same family as O-11/O-16.
- **Upstream-reportable**: **[NOTE]** — consolidation candidate, not a
  defect; tracked with O-11.
- **Our decision**: keep both detections for finding-count parity;
  Rule 10a is the spec-correct home for KEY uniqueness.

### O-23 [NOTE] Rule 11a/11b: absent TRAN_DLIM/RCON *column* is silently OK
- **python-ags4** (`rule_11`): reads `TRAN_DLIM`/`TRAN_RCON` via
  `TRAN.loc[...,'TRAN_DLIM']`. If the **column is absent** this raises
  `KeyError`, caught by the same handler used for "TRAN group missing"
  → **no** 11a/11b finding. 11a/11b fire only when the column exists
  but the value is empty.
- **Assessment**: a quirk, but defensible — `TRAN_DLIM`/`TRAN_RCON`
  are `OTHER` (not `REQUIRED`), so a file with no Record Links need
  not carry them; flagging their absence would be noise. The
  consequence: a file that *does* use RL but omits the columns gets no
  11a/11b (the bad links surface under 11c's "no such record"
  instead).
- **Upstream-reportable**: **[NOTE]** — minor; arguably 11a/11b should
  fire when an RL column exists but TRAN_DLIM/RCON columns don't.
- **Our decision**: mirror python — absent column → silent; empty
  value → 11a/11b. Documented in `rule_11`. Pinned by
  `rule_11a_11b_flag_blank_tran_delims`.

### O-24 [NOTE] Rule 11c record resolution is positional, not KEY-aware
- **Spec** (§4.1.1 Rule 11/11c): a Record Link is *"The GROUP name followed by the KEY FIELDs … in the order presented in the AGS4 DATA DICTIONARY"* and must *"cross-reference to the KEY FIELDs of data rows in the GROUP referred to"*.
- **python-ags4** (`fetch_record`): matches the link's value list against the target group's **leading columns positionally**, not against the dictionary-defined KEY fields. For a well-formed group whose leading columns *are* its KEY fields these coincide; they diverge if a group's KEY fields aren't its first columns.
- **Assessment**: a simplification in the reference impl. It works for conformant files (KEY fields lead the group by convention) but isn't literally "cross-reference to the KEY FIELDs".
- **Upstream-reportable**: **[NOTE]** — python-ags4 could resolve via the dictionary KEY fields for strictness; in practice equivalent.
- **Our decision**: replicate the positional match (`fetch_count`) for parity. Recorded so the semantic gap is on file.

### O-25 [VARIANCE] Effective dict for status/parent built independently of V4
- **Observed**: V4's `dictionary.rs` consumes the file's DICT group
  for *heading names* only; V7 needs DICT_STAT (KEY/REQUIRED) and
  DICT_PGRP too, so `relational.rs` builds its own richer
  `EffectiveDict` from the standard dictionary + the DICT group.
- **Assessment**: deliberate per the plan ("duplicate now, refactor in
  V8 if clearly right"). The two consumers read the same DICT rows but
  answer different questions; a shared `EffectiveDict` is a clean V8
  refactor candidate (would also let Rule 7/9 and 10a–10c share one
  parse of the DICT group).
- **Upstream-reportable**: no — internal structure.
- **Our decision**: keep V7's `EffectiveDict` self-contained; flag the
  consolidation for V8.

---

## V8 — cross-reference rules + integration (Rules 19b_2/19b_3, 20)

### O-26 [NOTE] 19b_2/19b_3 re-report headings already flagged by 19b_1/9
- **python-ags4** (`rule_19b_2`, `rule_19b_3`): both iterate *every* heading and split on `_`. For a heading with no underscore (or a bad structure) `rule_19b_1` (our V3) already fired and `rule_9` (V4) already fired, yet `rule_19b_2` adds *"Group X … could not be found"* and `rule_19b_3` adds *"… does not start with the name of this group …"* — the same defect reported up to three times under Rule 19b plus once under Rule 9.
- **Assessment**: redundant. The borrowed-heading semantic 19b_2/19b_3 add over 19b_1 is only meaningful when the prefix names *another* real group; for malformed headings it is noise.
- **Upstream-reportable**: **[NOTE]** — python-ags4 could gate 19b_2/19b_3 on "prefix ≠ group and heading has an underscore".
- **Our decision** (Stage 9c): the prefix-not-a-group case emits two findings — a `19b_2`-style "could not be found" (hinting a prefix typo) AND a `19b_3`-style "does not start with the name of this group" (hinting a placement mistake) — the two target *different fixes*. We still don't emit python's third redundant variant, so we're at 2 findings vs python's 3.

### O-27 [NOTE] Rule 20 on-disk checks are implemented as opt-in (`--check-files`)
- **python-ags4** (`rule_20`): besides the data-level check (every `FILE_FSET` used must be defined in the FILE group), it stats the filesystem — a `FILE/` sub-folder beside the `.ags`, a `FILE/<fset>/` per defined FSET, and each `FILE_NAME` on disk.
- **Us**: the **data-level** check always runs; the **on-disk** half is `references::rule_20_on_disk`, gated by `check_files` (CLI `lat validate --check-files`). **Default off** — a library validator must stay deterministic and path-independent; `ags4-corpus-qa validate` turns it on by default so the dogfood matches python-ags4's always-on stat.
- **Assessment**: no longer a standing variance — with `check_files` on, Rust and python **agree** on Rule 20; with it off, only the portable data-level core runs, a documented opt-out.
- **Upstream-reportable**: no — implementation/scope choice.
- **Our decision**: data-level always + on-disk opt-in; the corpus-qa dogfood enables it, so the prior `parity.rs` O-27 reconcile arm was removed (no longer a divergence). `db-to-ags4` reconstructs the `FILE/<fset>/<name>` sidecar tree from stored blobs so an exported delivery passes `--check-files`.

### O-28 [VARIANCE] External `--dict` custom-dictionary override — deferred at V8, implemented in laterite-dev#568
- **Plan**: the V8 roadmap listed an `lat validate --dict <path>` runtime override, then deferred it — `Dictionary` was `'static` phf-backed (zero-startup, compiled-in), so a runtime-parsed dictionary looked like a broad, regression-prone lifetime refactor threaded through `DictEntry` / `GroupMeta` and every rule module, for a feature `db-to-ags4 --validate` (bundled 4.2) doesn't need. For a period the flag was plumbed but refused with a `BadDict` error (exit 5).
- **Reality**: implemented in **laterite-dev#568** as a focused seven-phase arc. `Dictionary` became a lifetime-parametric enum (`Bundled` vs `Layered { base, delta }`) that keeps `Copy`; a custom dict is parsed once into a sparse `OwnedDelta` over a base edition, with the base **detected as a property of the dict itself** (structural, defaults to the latest edition when purely additive) so it is fixed before any delivery byte is read. Input is `.ags` or JSON (auto-sniffed). Every surface carries it — CLI `--dict` + `--dict-replace`, Python/Node `dict_path` / `dict_bytes` / `dict_replace`, wasm bytes-only — through one fast Rust core with thin bindings, the bundled-only path left byte-for-byte unchanged.
- **Assessment**: the clean `Cow` / enum-backed `Dictionary` the original deferral predicted was exactly the path taken — the overlay is paid for only when a dict is supplied. `--dict` and `--dict-version` coexist (the latter selects the base edition); `--dict-replace` drops the base for a fully bespoke dictionary. Re-parenting or overriding a STANDARD heading is **honour + warn** (a loud DICT finding, a KEY demotion loudest), never a silent shadow.
- **Upstream-reportable**: no — an implementation capability, not a spec divergence.
- **Our decision**: shipped across all four surfaces. The `.ags.idx` certificate **records** the effective dictionary's `{name, hash}` — a record, not a contract (O-48): a later `validate --index` against a different dictionary re-validates and surfaces `revalidate_reason = dictionary_changed`, it never silently vouches. Custom-dict content is per-invocation input and is never hashed into `ENGINE_FINGERPRINT`. Companion authoring ergonomics (`lat dict export` / `convert` / `generate`) are a tracked fast-follow, not part of this arc.

### O-29 [NOTE] EffectiveDict consolidation (O-25) deliberately not done
- **Context**: V4 (`dictionary.rs`), V7 (`relational.rs`) and V8
  (`references.rs`) each consume the file's DICT group. O-25 flagged a
  possible V8 consolidation into one shared `EffectiveDict`.
- **Decision**: **not** consolidated. V8 made the smallest safe move —
  exposed `dictionary::collect_file_dict` as `pub(crate)` so V4 and V8
  share the heading-name collector — but a full merge of the three
  consumers (different questions: names, status/parent, borrow sets)
  at the final phase would churn five green rule families for an
  internal-tidiness gain. The three consumers are small, independently
  tested, and stable.
- **Upstream-reportable**: no — accepted internal tech debt.
- **Our decision**: closed as "deliberately deferred". A future
  refactor (one `EffectiveDict` answering names + status + parent +
  borrow membership, parsed once) is the clean path if this code is
  next touched; recorded so the intent isn't lost.

---

## Post-V8 — dictionary edition auto-selection

### O-30 [VARIANCE] TRAN_AGS-driven edition selection — deliberate divergences from python
- **Context**: resolves **O-10**. We bundle all five AGS4 editions python-ags4 ships; `resolve_dict_version` picks one per file from its `TRAN_AGS` (explicit `--dict-version` overrides). python-ags4's `pick_standard_dictionary` uses a fixed exact-string map with `LATEST_DICT_VERSION = "4.1.1"`.
- **Deliberate divergences** (user-approved): (1) **bare `"4.0"` → 4.0.4** (newest bundled 4.0 patch) where python maps it to `4.0.3` (the *oldest*) — a file tagged `4.0` is best served by the latest 4.0.x schema. (2) **AGS 3.x → hard `UnsupportedEdition`** where python silently validates it against 4.1.1 — nothing AGS3 is specced here, so we refuse (detected at parse by its `**GROUP` / `<UNITS>` signature). (3) **bare `"4"` → the 4.0 line (4.0.4)** where python → 4.1.1 — "4" colloquially means AGS4(.0), the deterministic per-file choice.
- **Matched on purpose**: truly missing / `None` / non-numeric / `major != 4` / an explicit unbundled minor (`4.3`, `4.9`) → **4.1.1**, exactly python's `LATEST_DICT_VERSION`, so parity divergences there are real defects, not fallback artefacts.
- **Assessment**: intentional, data-driven improvements over the reference impl; the remaining fallback is deliberately python-identical.
- **Upstream-reportable**: **[VARIANCE]** — worth noting to AGS-DFWG that mapping bare `"4.0"` to the *oldest* 4.0 patch is surprising; newest-patch is the safer default.

### O-31 [VARIANCE] Rule 8 — empty `DT` UNIT now flagged (python parity; closes the O-12 degenerate gap)
- **Observed**: python-ags4 flags Rule 8 on a value like `TRAN_DATE = 2025-02-24` when that heading's `UNIT` is **empty** (`… does not match the specified format () …`). Rust stayed *clean* — a real false negative: `structural_dt_match` returned `true` on an empty UNIT ("no declared format → nothing to fail").
- **Spec**: Rule 8 — a value must match its declared format. python builds a per-char regex from the UNIT; an empty UNIT → empty pattern → `''.fullmatch(non_empty)` is `False` → flagged. i.e. "no declared format" means "no non-empty value can match".
- **Assessment**: distinct from **O-12** (non-empty *unrecognised* UNIT shapes stay lenient — they can't be calendar-checked). The empty UNIT is the degenerate case O-12 never covered (O-19 is precedent for empty-cell handling). Rust's old leniency here was a genuine false negative, not a deliberate variance. Scope: DT only.
- **Upstream-reportable**: **[VARIANCE]** — python's message text (`format ()`) is awkward, but flagging a value whose heading declares no format is defensible; an empty UNIT on a `DT` heading is itself a likely producer-side data defect worth noting to AGS-DFWG.
- **Our decision**: `structural_dt_match`'s empty-UNIT branch now returns `value.is_empty()` — a non-empty value with no declared format fails structurally → Rule 8, matching python. Recognised-UNIT and non-empty-unrecognised-UNIT (O-12) behaviour is unchanged.

### O-32 [VARIANCE] Non-UTF-8 input is decoded lossily, not refused (mirrors python's `errors="replace"`; closes the `NotUtf8` black hole)
- **Observed**: a 12,503-file dogfood run. 12 real AGS4 deliveries are cp1252/Latin-1 (`°`/`±`/`µ`/smart-quotes). The Rust validator hard-failed them as `NotUtf8` — **zero rules evaluated**, surfacing as `VALIDITY_DISAGREE`. python-ags4 never hard-fails on encoding: `AGS4.py:771` opens `encoding='utf-8', errors="replace"`, so an undecodable byte becomes `U+FFFD` and the file still validates.
- **Spec**: Rule 1. With the default utf-8 decode a replaced byte is `U+FFFD` (> 255), so `check.py:rule_1` emits **`"AGS Format Rule 1"`**. A real `ags4 check file.ags` (python's own CLI default) therefore *reports a Rule 1 error* on these files; it does not silently accept them.
- **Assessment**: refusing the input outright was the **only** real divergence — and it is worse than python (a black hole vs a finding).
- **Upstream-reportable**: **[VARIANCE]** — flag to AGS-DFWG that python's `errors="replace"`→`U+FFFD` *erases the original byte*: two different cp1252 files can collapse to byte-identical Rule 1 output, and the user is never told the file is most likely cp1252.
- **Our decision**: `parse_file` decodes with `String::from_utf8_lossy` — the stdlib twin of python's `errors="replace"`. Valid UTF-8 takes the borrowed fast path (byte-identical); invalid bytes → `U+FFFD` → `rule_1`'s > 255 arm → **`AGS Format Rule 1`**, so the 12 files now **AGREE with python on a Rule 1 error** instead of `VALIDITY_DISAGREE`. Correctly UTF-8-encoded extended chars stay the tolerated Rule 1 FYI (O-1). `ValidatorError::NotUtf8` is kept but unraised (back-compat).

### O-33 [VARIANCE] Rule 8 — DT/datetime bounded to pandas' Timestamp range (closes the value-range gap O-12 missed)
- **Observed**: a parity dogfood surfaced 8 `PYTHON_ONLY` Rule 8 files; the root cause is a single data defect — `LOCA_STAR = 0018-06-03` (cf. `2025-06-08` a row above — a data-entry error). python flags Rule 8, Rust stayed clean.
- **Spec**: Rule 8 — a value must match its declared TYPE/format *and* be a valid date/time. `0018-06-03` matches `yyyy-mm-dd` and *is* a valid date, so Rust's silence was defensible by the letter of the rule.
- **Assessment**: the divergence is python's engine, not the spec: `check.py:770` runs `pd.to_datetime(..., errors='coerce')`, and pandas' `Timestamp` range is **1677-09-21 .. 2262-04-11** — anything outside it becomes `NaT` → Rule 8. `chrono::NaiveDate` accepts any year, so Rust passed it. **O-12** asserted chrono≈pandas only for *in-range* years; this is the value-range counterexample it never captured. An `0018` year in a 2025 survey is unambiguously corrupt.
- **Upstream-reportable**: **[VARIANCE]** — pandas' Timestamp range is an implementation artifact, not an AGS requirement; flag to AGS-DFWG that python silently rejects spec-valid pre-1678 / post-2262 dates.
- **Our decision**: match python (flagging the bad data is the right validator behaviour, consistent with **O-31**). `dt_semantic_ok` bounds recognised date/datetime values to the pandas range (constants mirrored from pandas' public `Timestamp.min/max` docs — clean-room, not ported). The 8 corpus files now **AGREE**.

### O-34 [VARIANCE] `NotAgs4` ↔ python "missing mandatory groups" is a KNOWN_DIVERGENCE
- **Observed**: a dogfood run — 8 files were `VALIDITY_DISAGREE`: Rust returned `NotAgs4`, python emitted the mandatory-group rules (13/14/15/17 ± line-format noise). Every sampled file is a tab-delimited Excel "save as text" export or empty — **zero spec-valid quoted `"GROUP"` rows**.
- **Spec**: AGS4 Rules 3/4/5 mandate comma-separated, double-quoted fields. A tab-delimited or empty file has no spec-valid GROUP row → it is genuinely *not* AGS4 transfer format.
- **Assessment**: Rust's `NotAgs4("no GROUP rows found")` is the correct, *more informative* verdict; python has no refuse path, so it mislabels the file as merely "missing PROJ/TRAN/TYPE/UNIT". The **exact O-30 shape** one structural level up (Rust refuses; python mis-validates).
- **Upstream-reportable**: **[VARIANCE]** — python-ags4 silently "validating" a tab-delimited or empty file as Rule 13/14/15/17 is misleading; suggest python detect non-CSV / empty input and refuse, as it (eventually) does for AGS3.
- **Our decision**: parity-classifier-only, no parser/validator change. `parity.rs::classify` maps `NotAgs4` + python missing all three mandatory groups (`Rule 13 && 14 && 17`) → `KnownDivergence{O-34}`, keeping these out of the ACTION list (analogous to the O-30 `UnsupportedEdition` arm). The triple-rule guard keeps it narrow — a genuine `NotAgs4`-vs-real-findings disagreement still falls through to `ValidityDisagree`.

### O-35 [NOTE] Presence-only `reconcile` can't whittle a python parse-layer cascade
- **python-ags4**: its parsing layer turns one malformed construct into a *multi-rule* result — a lone embedded CR → universal-newline record split → Rule 2a+3+5 (`rule_6` itself is a no-op, O-2); a valid extended char → Rust FYI-only / python silent (O-1); an unquoted field → python Rule 3 *or* 4 by position vs Rust Rule 5 (O-3).
- **Us** (`ags4-corpus-qa/src/parity.rs`): `reconcile` matches single documented rule-swaps (O-2/O-3/O-26) and only when the *entire* symmetric diff is consumed, so a cascade leaves residue and a known root cause classifies as a false `RUST_ONLY` / `PYTHON_ONLY` ACTION.
- **Assessment**: a real limitation of presence-only parity, not a validator defect. Generic widening is unsafe (Rules 2a/3/5/9/18 fire for many legitimate reasons); only *signature-narrow* arms (à la the O-34 triple-guard) are acceptable.
- **Upstream-reportable**: no — harness / methodology.
- **Our decision**: document it; do **not** broaden `reconcile` generically. Signature-narrow arms (`rust=={Rule 6} ∧ py⊆{2a,3,5} → O-2`, etc.) are the sanctioned follow-up.

### O-36 [NOTE] Parity differential is triage-biased by default
- **Us** (`ags4-corpus-qa/src/parity.rs`): the parity set is `triage ∪ reservoir(rest, --parity-sample)` and `--parity-sample` **defaults to 0**, so by default only files the Rust side already flagged odd are cross-checked against python-ags4. A file Rust handles confidently-but-wrongly (plausible `Findings`) is never sent to the oracle — silent agreement on a wrong verdict is invisible.
- **Assessment**: a sampling bias in the dogfood, not a validator defect — but it overstates the strength of the parity claim.
- **Upstream-reportable**: no — harness / methodology.
- **Our decision**: keep `--parity-sample` (perf), but treat triage-only as the floor, not the ceiling — a non-zero default sample and a per-rule "rules with zero parity evidence" report are the sanctioned follow-ups.

### O-37 [VARIANCE] Native parser is lenient where python-ags4 raises hard
- **python-ags4** (`AGS4.py::AGS4_to_dict`): raises `AGS4Error` on three structural anomalies *before* returning data — duplicate `GROUP` lines for one code, DATA rows with a field count ≠ HEADING, and (when `rename_duplicate_headers=False`) duplicate headings. Read fails; nothing downstream sees the bad row.
- **Us**: the native parser is deliberately lenient — duplicate GROUPs merge into one bucket; ragged DATA rows pass through (extra dropped / short padded, Rule-4-reportable); duplicate headings rename with a warning in compat. The parser's job is to return *something* the validator can catalogue.
- **Assessment**: not a bug — opposite design philosophies. The native validator's value is *reporting* problems through Findings; crashing on malformed input would mean a bad file produces no report at all. python's "first crash, then findings" strictness suits its pipeline; native lenience ("findings first, never crash") suits ours.
- **Upstream-reportable**: no — design choice, not a bug.
- **Our decision**: keep native lenient. `laterite.compat` (the python-ags4 drop-in) interposes a `_strict_pre_check` that scans the raw file via `csv.reader` and raises `Ags4Error` for the three cases, matching python's wording closely enough that their test suite's `pytest.raises(AGS4Error, match=…)` passes. Native callers (`laterite.Validator`, `lat`) never hit the pre-check.

### O-38 [SPEC] Rule 8 DT validation: python-ags4 forbids non-ISO UNITs
- **python-ags4** (`check.py::rule_8`, DT branch): a value passes only if BOTH a structural regex mask AND `pd.to_datetime(value, format='ISO8601')` succeed (except `hh:mm[:ss]` units, which use explicit `%H:%M[:%S]`). The `ISO8601` fallback for any non-time UNIT means a value like `01/12/2020` under UNIT `dd/mm/yyyy` is structurally fine but fails the pandas mask — so python-ags4 cannot validate **any** non-ISO UNIT (`dd/mm/yyyy`, `dd-mm-yyyy`, `mm/dd/yyyy`, … all flag every value, valid or not).
- **Spec** (§4.1.1 Rule 8): a DT value must be of the declared TYPE, and the **UNIT row declares the format**. A UNIT of `dd/mm/yyyy` is legitimate; the spec does not require ISO-8601.
- **Assessment**: a python-ags4 implementation defect, not a laterite divergence — pinning to `ISO8601` inverts the spec's "UNIT declares the format" contract for any non-ISO shape. A DT-format matrix recorded 11 divergences, all "python-ags4 wrongly flags a valid value".
- **Upstream-reportable**: **[SPEC]** — python-ags4's `rule_8` DT branch should translate the UNIT pattern into a `pd.to_datetime` `format=` string rather than hard-coding `ISO8601`. High priority — affects real European/US delivery files.
- **Our decision**: laterite implements the spec-correct path: `lex_unit_value` walks the UNIT pattern token-by-token (context-sensitive `mm` = month-or-minute), extracts calendar fields, and validates ranges + the pandas bound (O-33). All European/US date-format UNITs validate correctly; the matrix goes 27/45 → 34/45 AGREE, the 11 residuals all laterite-correct / python-wrong.

### O-39 [SPEC] Rule 10c — empty parent KEYs are "no entry", not a missing link
- **Spec** (`spec:AGS4-4.2-2025.pdf §4.1.1 Rule 10c`): *"Every entry made in the KEY fields in any GROUP must have an equivalent entry in its PARENT GROUP."* The text hinges on *"entry made"* — an empty cell reads as either "an entry pointing to nothing" (strict) or "no entry made" (permissive).
- **python-ags4** (`check.py::rule_10c`): reports no orphan for such a row, and its fixture `Standalone_SAMP_IDs.ags` (lab-control SAMP rows with no LOCA_ID) is asserted clean — but **not because it skips them**. There is no skip in `rule_10c`; it left-merges child onto parent and reports every unmatched row. The rows go unreported because its `tables` include the UNIT/TYPE pseudo-rows on both sides of that merge, and the parent's UNIT row has empty key cells — so an all-empty child key matches it. The behaviour is the one described here; the mechanism is a coincidence, and [[O-52]] carries the demonstration and what it costs.
- **Us** (pre-Stage 9b): strict — laterite flagged every such row (`No parent entry in LOCA for KEY combination:` — empty tuple). Real geotech workflows produce these legitimately (lab controls, off-site samples), so the strict reading was noise.
- **Assessment**: the spec is ambiguous; the geotech-domain reading is python's. Aligning is the right UX call.
- **Upstream-reportable**: **[SPEC]** — the AGS4 spec should explicitly say whether empty KEY cells participate in Rule 10c's link requirement. Either reading defensible; clarity beats either.
- **Our decision** (Stage 9b): align with python's permissive reading — `rule_10c` skips a child row when all its parent KEY tuple values are empty; a row with even one non-empty parent KEY still gets the full check.

### O-40 [NOTE] The `.ags.idx` byte index records true GROUP line-starts (the csv reader's record positions were off-by-one for CRLF, and absorbed leading blanks)
- **Observed**: the byte-offset index recorded each `"GROUP",…` start from the `csv` crate's `StringRecord::position().byte()` — the byte where the reader *enters* the record, which for a CRLF-terminated previous line is the preceding `\n` (one byte early), and which absorbs leading blank lines (first GROUP at byte 0 instead of after the blanks). LF / BOM / quoted-embedded-newline files already matched.
- **Spec**: the `.ags.idx` sidecar is *our* format, not AGS-specced — but Rule 2a mandates CRLF, so the off-by-one was the **common real-world case**, not an edge. The loose offsets were still valid certs (a slice keeps a harmless leading `\n` the reparse skips), which is why the consistency tests never caught it.
- **Assessment**: a cert should record where a group's bytes *actually* start, so a sliced read or a ranged-GET lands precisely on the `"GROUP"` record. The shared parse leaf (#168) already emits a source-true `group_byte`; sourcing the index from it removes the divergence by construction.
- **Upstream-reportable**: none — `csv`-crate record-position semantics plus our own cert format, not an AGS4-spec or python-ags4 matter.
- **Our decision** (#168 Phase 4): `index_ags4_bytes` sources GROUP offsets from the parse leaf's source-true byte walk instead of the csv reader. The `.ags.idx` format stays locked at v1 — only the offset *values* tighten, so existing certs still deserialize (a re-mint produces tight offsets).

### O-41 [VARIANCE] Rows before the first GROUP are REPORTED as Rule 2 findings, not a hard parser crash
- **Observed**: a HEADING / UNIT / TYPE / DATA row before any GROUP row is structurally invalid (it belongs to no group). The question is what a validator should DO with it.
- **python-ags4**: its PARSER hard-fails — a pre-GROUP HEADING raises `AGS4Error`, a pre-GROUP DATA/UNIT/TYPE a `KeyError`. Because the parser raises, `check.py` never runs — the user gets a traceback and NO findings report for the file.
- **Us** (#189): the shared parse leaf is deliberately LENIENT — it drops the orphan row so the rule engine still runs, and `rule_2_orphan_rows` REPORTS each orphan as an `AGS Format Rule 2` error, line-located. (A code-less `"GROUP"` row is already a Rule 4 finding.)
- **Assessment**: reporting beats crashing — laterite produces a COMPLETE report that *includes* the structural defect where python aborts on the first one and reports nothing (same philosophy as O-32). Rule 2 is the attribution: the row belongs to no GROUP, Rule 2's domain.
- **Upstream-reportable**: **[NOTE]** — python-ags4 could downgrade these parser hard-fails to `check.py` findings for a more useful report, but it's a design-philosophy difference, not a defect; not filing.
- **Our decision**: added `rule_2_orphan_rows`; python-ags4 parity is unchanged at 122/9. The core *reader* path (opt-in strict) hard-fails on the same case instead — a data reader is a different consumer than a validator.

### O-42 [VARIANCE] TRAN_AGS="4.0" resolves to 4.0.4 (superset-safe), with a content guard; python's static "4.0"→4.0.3 over-reports Rule 10c
- **Observed**: `TRAN_AGS="4.0"` is ambiguous — there were two 4.0
  dictionary releases. 4.0.4 is a STRICT SUPERSET of 4.0.3: identical 124
  groups, ZERO headings removed, 8 headings added (`GCHM_DLM`, `GCHM_RTXT`,
  `LOCA_NATD`, `LOCA_ORCO`, `LOCA_ORID`, `LOCA_ORJO`, `RDEN_IDEN`,
  `SAMP_RECL`), and exactly ONE other delta — PMTL's parent is `PMTD` in
  4.0.3, `PMTG` in 4.0.4+. PMTL's columns are byte-identical across both.
- **python-ags4** (`check.py::pick_standard_dictionary`): a static table maps the
  string `"4.0"` → the *4.0.3* dictionary (the older patch; anything not in
  the table → 4.1.1). So a `"4.0"` file is judged against 4.0.3 — its PMTL is
  checked against PMTD and its heading set is 4.0.3's.
- **Us** (`resolve_dict_version` + `guard_4_0_4`): `"4.0"`/bare `"4"` →
  4.0.4 (newest bundled 4.0 patch, O-30). Because 4.0.4 ⊇ 4.0.3 this never
  false-flags the 8 newer headings. A content guard additionally upgrades an
  exact `"4.0.3"` to 4.0.4 when the file uses any of the 8 4.0.4-only
  headings — the one deterministic edition signal — and emits a transparency
  FYI naming the heading. An explicit `--dict-version` (Forced) is never
  overridden.
- **Assessment**: 4.0.4 is the low-false-positive default and, for a real corpus
  file, demonstrably correct: that file declares `"4.0"`, USES 4.0.4-only
  headings (so it is ≥4.0.4) and its PMTL `PMTD_SEQ` (the 4.0.3 chain key) is
  blank in all rows (not using the PMTD chain). python's 4.0.3 read forces
  PMTL→PMTD and reports 150 Rule 10c orphans that are FALSE POSITIVES. Our
  Rule 10c + dictionary are correct: forcing `--dict-version 4.0.3`
  reproduces python's 150 exactly; 4.0.4/auto report 0. The editions are
  content-indistinguishable EXCEPT those 8 headings, so no signal can prove a
  no-heading file is 4.0.3 — the superset (4.0.4) is the only safe default.
- **Upstream-reportable**: **[YES]** — python-ags4's static `"4.0"→4.0.3` alias is stale
  (never bumped when 4.0.4 shipped): it over-reports Rule 10c via the
  PMTL→PMTD hierarchy and would mis-flag the 8 4.0.4 headings as non-standard
  on 4.0.4 files tagged `"4.0"`. Candidate upstream report.
- **Our decision** (#191/#222): KEEP `"4.0"→4.0.4` (O-30) — confirmed correct, no
  dictionary change. Added `guard_4_0_4` (`validator/src/lib.rs`): a 4.0-line
  auto-resolution lands on 4.0.4 when the file uses a 4.0.4-only heading, with
  an FYI when it overrides a declared `"4.0.3"`. PMTL is the ONLY group with
  an edition-varying parent, so the residual blast radius is one group on a
  `"4.0.3"`-exact file carrying no 4.0.4 heading — undetectable by design,
  accepted. compat↔python parity on the real corpus is 800/801 (the lone
  divergence is this file; laterite avoids the 150 phantom orphans).
  - **Reported**: laterite#222 (2026-06-22).

### O-46 [NOTE] The lean read path rejects non-UTF-8 input ("input is not valid UTF-8"); the validator decodes it lossily and flags Rule 1 (O-32) — the deliberate encoding fork, wording pinned at #168 Phase 7
- **Observed**: after the parser convergence (#168), core's lean read path (`read_ags4_bytes` — backing the `.ags.idx` index + the DuckDB extension) REJECTS a non-UTF-8 file with **"input is not valid UTF-8"** (from the shared parse leaf). The validator path does NOT reject — it decodes lossily (O-32).
- **Spec**: Rule 1 — the file "shall be entirely composed of characters from the ASCII character set," so a non-UTF-8 byte is out-of-spec either way; what differs is how each surface *surfaces* it.
- **Assessment**: a deliberate laterite encoding **fork**, not a python-ags4 divergence. Per-consumer: the validator + bindings decode lossily (→ Rule 1, matching python's `errors="replace"`, O-32) because a validator must produce a COMPLETE report; the lean read path rejects because it assumes an already-valid file and the byte index is only meaningful over the exact source bytes, so a loud reject is the honest failure there. The python-facing surface already AGREES with python via O-32.
- **Upstream-reportable**: No — internal cross-surface design; nothing here diverges from python-ags4 (the parity path is O-32).
- **Our decision** (#168 Phase 7): the lean read path's message is pinned to the shared leaf's **"input is not valid UTF-8"**; the fork — lean reject, validator/bindings lossy (O-32) — is retained as intended.

### O-53 [VARIANCE] A blank TRAN_AGS earns its Rule 10b error and nothing else — python-ags4 stacks its unrecognised-edition FYI on the same cell, while laterite carries the schema-fallback fact on the report envelope, where every run shows it
- **Observed**: `TRAN_AGS` is present as a heading but its DATA cell is **blank**. Neither validator can resolve an edition from it, so both fall back to 4.1.1 ([[O-30]]) and judge the file against a schema its author never declared.
- **Spec** (`spec:AGS4-4.2-2025.pdf` §4.1.1 Rules 10b/14): requires the TRAN group and its REQUIRED headings — a blank REQUIRED cell is a Rule 10b breach — and says nothing about what a validator should report once it has fallen back.
- **python-ags4** (`check.py::is_TRAN_AGS_valid`): applies its unrecognised-edition advisory to the blank as well as to an unknown label, emitting `"'' in TRAN_AGS is not a recognized AGS4 version. Therefore, v4.1.1 ... will be used"` as a top-level `FYI` — on top of the Rule 10b error the same cell already earns. Because that tier is opt-in, a DEFAULT python run reports the blank but never says which schema it then used.
- **Us** (`rules/groups.rs::tran_ags_unrecognised`): returns early on an empty value, so a blank produces the Rule 10b error and nothing more; the unrecognised-edition finding is reserved for a value that is present but unknown ([[O-45]]). The fallback is not dropped, it is carried somewhere else — on the report envelope (`Report.dict_version` + `Report.resolution`, which `lat validate` renders as `dictionary 4.1.1 (fallback)` against the seed's `dictionary 4.1 (exact)`), printed on every run at every tier.
- **Assessment**: one fact, attached in two different places. python attaches it to the cell, which costs a second finding about a cell already flagged and puts it behind an opt-in tier; we attach it to the verdict, where it is unconditional and answers the question once for the file rather than once per offending cell. Ours is the more visible of the two, and it is why the wrong-schema risk [[O-45]] promoted to WARNING for a *present* unknown edition needs no promotion here — the envelope already states it.
- **Upstream-reportable**: **[NO]** — python's extra line is redundant rather than wrong, and which tier it sits in is their categorisation to make.
- **Our decision**: keep the early return. One finding per broken cell; the schema a verdict was actually reached against is a property of the verdict, not of a cell, and belongs on the report.

## Post-V8 — laterite-originated checks (no python-ags4 equivalent)

### O-43 [VARIANCE] A self-declared but non-standard PA abbreviation is a laterite-originated FYI (Related to Rule 16); python-ags4 has no such check
- **Observed**: Rule 16 requires every `PA` value to be defined in the file's ABBR group. A file can SATISFY that by self-declaring a code in ABBR that is not in the standard picklist for its heading — a typo (`"Borng"`) or an invented code (`"ZZ"`). Spec-legal, but the non-standard code doesn't interoperate with tooling that keys off the standard picklist.
- **python-ags4**: `rule_16` checks PA values only against the file's OWN ABBR; `fyi_16_1` flags only DESCRIPTION drift on an otherwise-standard code. Its Warnings section is literally `# TO BE ADDED`. So it has NO check that a self-declared abbreviation is itself non-standard.
- **Us**: for a `(heading, code)` where the heading has a bundled standard picklist but the code isn't in it, emit one `FYI (Related to Rule 16)`. Bounded to standard-picklist headings — a genuinely custom / DICT-defined `PA` heading has no standard set to judge against and is skipped.
- **Assessment**: a clean-room data-quality signal that catches typo'd / invented abbreviations the error-tier rules cannot (the file IS Rule-16-valid). Informational (FYI) and opt-in, never changes the error verdict, so python-parity is untouched.
- **Upstream-reportable**: **[NO]** — an additive laterite feature, not a python-ags4 defect (though their unimplemented Warnings section could adopt it).
- **Our decision** (#199): ship as an FYI under the existing `FYI (Related to Rule 16)` bucket — no new finding key, so the compat severity classifier treats it as FYI unchanged. FYI, not WARNING: the file breaks no rule.

### O-44 [VARIANCE] Structural validation of a file-level DICT group is a laterite-originated WARNING (Related to Rule 18); python-ags4 only consumes DICT, never validates it
- **Observed**: Rule 18 requires a DICT group for non-standard names but says nothing about the DICT's OWN well-formedness. A file can declare custom groups/headings through a MALFORMED DICT (a missing `DICT_TYPE`/`DICT_GRP`/`DICT_HDNG` column, a blank `DICT_GRP`, a `HEADING`-row with a blank `DICT_HDNG`). The engine only *consumes* DICT, so a malformed one degrades every downstream check with zero feedback.
- **python-ags4**: `rule_18` does NO structural validation — it only flags non-standard headings with no DICT group at all (our error-tier `rule_18`, O-17). It never inspects the DICT's own structure.
- **Us**: flags the clearest defects as opt-in WARNINGs under `Warning (Related to Rule 18)` — a missing required column, a blank `DICT_GRP`, a `HEADING`-row with a blank `DICT_HDNG` (branching on `DICT_TYPE` first so a GROUP-row's legitimately-blank `DICT_HDNG` isn't flagged).
- **Assessment**: a clean-room structural check for a malformed dictionary the spec is silent on and python-ags4 ignores. WARNING (shown by default since #203, opt out with `--no-warnings`), not Error — the file breaks no rule. WARNING over FYI (cf. O-43): a malformed DICT is genuinely broken, not merely spec-legal-but-unusual, and it satisfies the #321 rule that a warning predicts a downstream **surprise** — a consumer resolving the custom headings gets something other than the author meant.
- **Upstream-reportable**: **[NO]** — an additive laterite feature, not a python-ags4 defect.
- **Our decision** (#200): ship. The separate `Warning (Related to Rule 18)` label keeps the compat severity classifier from miscounting it as an error, and the first WARNING-tier producer gives the `warnings=True` flag a defensible producer.
- **Amended** (#321): the claim above that the default verdict was untouched was false when written — a warning was counted in the exit code, so this check could fail a build over a file that breaks no rule. It is true now: errors alone decide the verdict, this is shown and not fatal, and `--warnings-as-errors` is how a caller opts into failing on it.

### O-45 [VARIANCE] An unrecognised TRAN_AGS edition is a laterite WARNING (Related to Rule 14), shown by default; python-ags4 emits it only as an opt-in FYI
- **Observed**: a `TRAN_AGS` value that is PRESENT (Rule 14 satisfied) but not a recognised AGS4 edition — a typo, an old `"4"`, or a bespoke label. The edition can't be matched, so the validator falls back to a default dictionary (4.1.1) and validates against a schema that may not be the author's — a wrong-schema risk.
- **python-ags4**: emits a top-level opt-in `"FYI"` (`"TRAN_AGS is not a recognized AGS4 version: ..."`), never an error.
- **Us**: classifies the SAME condition as a WARNING under `Warning (Related to Rule 14)`, shown by default (#203) so the schema-fallback risk is visible without a flag. In FYI-only mode (`compat`) it instead emits the python-ags4-matching `"FYI"`, preserving drop-in fidelity.
- **Assessment**: a deliberate laterite-STRICTER categorisation (cf. O-43/O-44) — an unrecognised edition can yield a verdict against the wrong dictionary, so it belongs in the default-visible WARNING tier. It is the archetype of the #321 rule that a warning predicts a downstream **surprise**: the file is judged against a schema that may not be the author's, and nothing in the report would otherwise say so. WARNING, not Error: the file breaks no rule, and error-parity is untouched (the parity gate compares only `AGS Format Rule N` keys).
- **Upstream-reportable**: **[NO]** — a laterite categorisation choice, not a python-ags4 defect.
- **Our decision** (#203): promote FYI → WARNING, shown by default; `compat` keeps python's FYI for drop-in fidelity. The second WARNING-tier producer after O-44.
- **Amended** (#321): "shown by default" used to mean "and exits 1", which made this the worked example in the issue that split the two — a file whose only blemish was an unrecognised edition failed a CI job over a rule nobody broke. Shown by default still; fatal no longer, unless `--warnings-as-errors` asks for it.

### O-51 [VARIANCE] Overrides a custom-dictionary OVERLAY makes to the standard schema are laterite-originated findings — WARNING when row identity changes, FYI when it does not; python-ags4's custom dictionary replaces rather than overlays, so it cannot ask the question
- **Observed**: a `--dict` OVERLAY (laterite-dev#568) may redefine parts of the STANDARD schema, not just add to it. Three distinct conditions: re-parenting a standard group, demoting a standard KEY heading to a non-KEY status, and a plain type/status override of a standard heading. The overlay is honoured either way, so without feedback a bespoke dictionary could reshape the standard schema silently.
- **python-ags4**: **does** take a custom dictionary — `-d/--dictionary_path` on `check` (`ags4_cli.py`), `check_file(standard_AGS4_dictionary=…)` (`AGS4.py`) — but it **replaces** wholesale: an argument that is not a version string is read as a path and becomes `tables_std_dict` entire, discarding the bundled standard. There is then nothing left to compare an override against, so this is not a check they declined to write — their data model makes the question unaskable.
- **Us**: two modes, and these findings belong to only one. `emit_override_findings` opens with `if !custom.fall_through { return; }` — a full replacement redefines the schema by declaration, so nothing there is an override. In overlay mode each override is reported, tiered by the #321 surprise test.
- **Assessment**: **WARNING** for re-parenting and KEY demotion — both change row identity silently, so Rules 10a/10c start answering differently about rows nobody edited, and a consumer receives something other than the author meant. **FYI** for a plain type/status override: the caller declared it in the dictionary they passed, it is honoured exactly as declared, and nothing downstream differs from what the file says. Worth being able to see; not worth interrupting. Neither tier decides the verdict (#321), so a custom dictionary can never fail a file merely for overriding something.
- **Upstream-reportable**: **[NO]** — a laterite-only feature. python-ags4's replacement model cannot express the question, so there is no defect to report.
- **Our decision** (laterite-dev#568 / #321): ship the split. Note the population this matters least for: a python-ags4 user porting a `-d` workflow lands in **full-replacement** mode, the exact mode in which none of these can fire — so for the migrant the hazard is not small, it is structurally zero. Two corrections shipped with the tiering. The findings did not honour `include_warnings` / `include_fyi` at all (`emit_override_warnings` took no `CheckOptions`), so `--no-warnings` did not suppress them and the in-crate tests asserted them under `CheckOptions::default()` — errors-only — which is what a missing gate looks like. And the demoted finding moved to its own `FYI (Related to DICT)` label, leaving the `DICT` bucket warning-pure: the same separation `Warning (Related to Rule 18)` exists for, and the reason `compat`'s `count_errors` can classify it (a bare `DICT` key matched none of its branches and counted as nothing).

### O-52 [VARIANCE] A DECLINED Rule 10c parentage check is reported — an all-empty parent-KEY child row produces a WARNING naming what could not be checked; python-ags4's silence there is a coincidence of its UNIT/TYPE pseudo-rows, not a decision
- **Observed**: Rule 10c declines to check a child row whose parent-KEY cells are **all** empty — the row claims no parent, so there is no link to verify ([[O-39]]). The decline was **silent**: the row simply produced no finding, which is exactly what a row that passed the check produces. A legitimate standalone record (a lab control, an off-site sample) and a key blanked or typo'd by accident are indistinguishable from the report, and only the author knows which they meant.
- **Spec** (`spec:AGS4-4.2-2025.pdf §4.1.1 Rule 10c`): says what must have a parent entry, and nothing about what a validator should report when it decides the requirement does not apply. The ambiguity [[O-39]] is about is whether an empty cell is an entry; this record is about the silence that follows either answer.
- **python-ags4** (`check.py::rule_10c`): has **no such skip**. It left-merges the child table onto the parent on the parent's KEY fields and reports every `left_only` row. The reason an all-empty-key row is not reported there is a **coincidence of its data model**: `tables` carries the HEADING/UNIT/TYPE pseudo-rows as DataFrame rows on BOTH sides of the merge, and the parent's UNIT row has empty cells in the key columns — so an all-empty child key MATCHES it. (The TYPE rows match each other for a different reason: both carry the same declared type token, `ID` against `ID`. Two coincidences, one cause — pseudo-rows in a data merge.) Demonstrated with two files identical except for the parent LOCA's UNIT row carrying `m` under `LOCA_ID` instead of empty: the empty-unit file reports no Rule 10c, the other reports exactly two — the child's own UNIT row and its standalone DATA row, the two rows keyed empty. The child's TYPE row is not among them, because it never depended on the emptiness.
- **Us** (#656): the skip stays exactly as it is — the verdict does not move, and a standalone row is still not an orphan — and it now **says it happened**, as `Warning (Related to Rule 10c)` naming the parent group and the key fields it could not check. The warning tier is the one a reader actually meets: warnings are shown by default where FYI is not, and neither decides the verdict.
- **Assessment**: declining to check and checking-and-finding-nothing are different answers, and a report that renders them identically is answering a question it did not ask. This is a laterite-originated check: python-ags4 cannot have an equivalent, because it never knows it declined anything.
- **Upstream-reportable**: **[BUG]** — python-ags4's Rule 10c merge includes the UNIT/TYPE pseudo-rows on both sides. That is benign for a well-formed file, where the coincidence lines up, but a parent group whose UNIT row carries a value in a KEY column — legal, and no other rule objects — turns the child's own UNIT row into a reported orphan. Worth filing: the merge should exclude the pseudo-rows and decide the empty-key case deliberately, whichever way they choose.
- **Our decision** (#656): emit the warning; leave the skip and the verdict untouched. It is a new laterite-only rule key, so it shows as a rust-only label wherever dual validation runs with warnings on — forge does, corpus-qa's default validate stage does not — and `classify` reconciles it to this record rather than filing it as an action — expected, and [[O-39]] is the record of why the two engines agree on the verdict there anyway.

## Post-V8 — #422 quote-aware universal-newline line splitting

### O-47 [NOTE] Quote-aware universal-newline parsing (#422): a lone-CR terminator now splits into rows + Rule 2a (converging with python), an embedded CR/LF stays in-field + Rule 6 (diverging)
- **python-ags4**: reads lines with a quote-BLIND universal-newline reader (`open(newline='')` + `enumerate(f)`), splitting on `\r\n` / `\n` / lone `\r` regardless of quoting. A lone-CR terminator → one **Rule 2a** per row (probed: an old-Mac file of N rows yields N Rule-2a findings). An embedded CR/LF *inside a quoted field* is TORN at the newline, so the fragments trip **Rule 4** (field count) + **Rule 5** (unquoted); its `rule_6` is a no-op (O-2), so it never reports the newline as Rule 6 — e.g. `tests/test_files/4.1-rule6_2.ags` (`test_rule_6_2`).
- **Us (before #422)**: the parse leaf split on `\n` ONLY. A lone-CR terminator survived as an interior byte, was mislabelled an embedded CR (**Rule 6**), and `StripEmbeddedCr` deleted it — WELDING the two rows on a *fix* (silent data corruption). An embedded `\r\n` was torn at the `\n` like python, incidentally matching `test_rule_6_2`.
- **Us (after #422)**: line-finding is quote-aware + universal-newline (`laterite-ags4-parse::line_spans`, the ONE splitter now shared by the parser AND `apply_fixes`, so their line numbering agrees by construction). A CR/LF *outside* quotes is a terminator — `\r\n` conforming, lone `\r`/`\n` → **Rule 2a**; a CR/LF *inside* a quoted field is embedded content → **Rule 6** (widened to catch LF as well as CR), the row kept whole. So lone-CR files now parse into proper rows with per-row Rule 2a (CONVERGING toward python), while embedded-newline fields keep the row and report Rule 6 where python tears them into Rule 4/5 (DIVERGING — the O-2 'better than python' behaviour, now realised end-to-end).
- **Spec** (§4.1.1): Rule 2a mandates CR+LF line termination; Rule 6 bans carriage returns AND line feeds within/between data VARIABLEs. A lone-CR *terminator* is therefore a Rule 2a matter and an *embedded* CR/LF a Rule 6 matter — the change makes both attributions spec-correct.
- **Assessment**: a correctness gain on both axes. The lone-CR corruption is eliminated by construction — `StripEmbeddedCr` can no longer receive a terminator-CR, because terminators are consumed by the splitter and never left in the line body. Embedded newlines are now caught precisely (Rule 6) instead of as tear-symptoms. Concretely `test_rule_6_2` flips from pass to a documented divergence (laterite Rule 6 vs python Rule 4/5), shifting the python-ags4 parity baseline **122/9 → 121/10**.
- **Upstream-reportable**: partially — python's quote-blind reader plus its `rule_6` no-op (O-2) mean it structurally cannot report an embedded newline as Rule 6; already captured under O-2. No new upstream item.
- **Our decision**: ship the quote-aware universal-newline splitter shared by the parser and `apply_fixes`. Accept the `test_rule_6_2` divergence as correct (documented here + in `docs/history/python_ags4_parity_baseline.md`). The splitter's line boundaries are pinned to agree with `split_ags_line` by a property test (`laterite-ags4-parse/tests/line_split.rs`).

## V1 — line-level rules (Rules 1, 3, 5, 6)

### O-48 [NOTE] Rule 1's severity is a property of the DECODER, not the file: the same bytes are an error under one --encoding and an FYI under another — on both validators — so any CACHED verdict must record its decoder
- **Observed**: one unchanged file, two `--encoding` labels, two different **error** verdicts. A clean minimal AGS4 delivery whose `PROJ_NAME` carries a Greek capital omega — UTF-8 bytes `CE A9`. Read as UTF-8 that is ONE code point (937), above the extended-ASCII range Rule 1 tolerates, so `AGS Format Rule 1` (an ERROR). Read as windows-1252 the very same two bytes are TWO code points (206, 169), both inside it, so `FYI (Related to Rule 1)`. Probed 2026-07-14 through **both** validators on the same bytes: they agree, finding for finding, in both directions.
- **python-ags4**: the same behaviour, and it says so out loud — `check.py::rule_1` sorts on `is_ags_ascii(line)` (128–255 → FYI; above → error) and words its own message *"Has Non-ASCII character(s) (assuming that file encoding is '{encoding}')"*. The verdict is decoder-relative by construction, and python-ags4 documents that in the finding text.
- **Us**: identical severities, by the same route (see O-1 for the 0–255 tolerance, O-32 for the lossy decode). Not a divergence — a shared, faithful property of a rule that sorts characters by code point when *which characters* the bytes become is the decoder's answer, not the file's.
- **Spec**: Rule 1 ("the data file shall be entirely composed of ASCII characters") names a character property, and a byte sequence has no characters until something decodes it. The spec never says which decoder, so it cannot say what Rule 1 means for a non-ASCII byte — see the `rule1-ascii-strict-vs-extended` insight and O-1.
- **Assessment**: harmless while a verdict is computed fresh each time (you get the answer for the decoder you asked about) — **and a false-clean generator the moment a verdict is CACHED**. A validation is a function of `(bytes, decoder)`; the `.ags.idx` certificate sealed only the bytes. Proven exploitable on the shipped build: certify the file above under `windows-1252` (no error under that decoder, so it mints), then read it back with the default decoder offering the cert, and it came back `count = 0, certified = true, is_valid = true` — while a plain validate of the very same bytes reported the Rule 1 error. The general lesson, and the reason this is catalogued rather than merely fixed: **CONTENT is not the same as SEALED.** The encoding is genuinely content (the text is a pure function of the bytes and the label), which is exactly why it was allowed onto the certificate's fast path; what was missing is that the certificate never recorded WHICH label. Every input the findings depend on must be in the certificate.
- **Upstream-reportable**: **[NO]** — both validators behave the same, and python-ags4's finding text already discloses the assumption. The consequence is ours: it lands on anything that caches a verdict.
- **Our decision** (2026-07-14, the `.ags.idx` v2 trust model): the decoder is part of the question and part of the stamp. `ValidationStamp.encoding` records what the bytes were READ as; `Question.encoding` carries what the caller is reading them as; `Sidecar::decide` refuses a certificate minted through a different decoder (`RevalidateReason::EncodingDiffers`) and the engine runs. The decoder a certificate WAS minted under still gets the fast path — a match, not a ban. Gated on output values at three levels: `laterite-ags4-core` (`decide` returns `EncodingDiffers`), `laterite-ags4-trust` (the omega file: minted under cp1252, refused for UTF-8, the Rule 1 error reported, cp1252 still vouched), and both surfaces (Python + Node, through the public API). Each gate first asserts the PREMISE — that the two decoders really do disagree — because the rest is only interesting if they do.

## V5 — typed-value rule (Rule 8)

### O-49 [VARIANCE] A numeric TYPE's count (the n in nDP/nSF/nSCI) is read uncapped from the file and fed into a format width — a crafted "9999999999SF" OOMs python-ags4 (~10 GB string); laterite now clamps to 30
- **Observed**: the `n` in an AGS4 numeric TYPE ("3SF", "3DP", "3SCI") is parsed straight from the file's own TYPE row and used as a significant-figure / decimal-place WIDTH when a value is rendered to its expected form — Rule 8's grammar check, the fixes engine, and the XLSX writer all do this. No edition caps it; real AGS types are single-digit (0–6). A crafted or corrupt TYPE like "9999999999SF" is a valid parse and reaches the formatter on raw bytes, before any rule has vetted the TYPE.
- **python-ags4** (`AGS4.py::_format_SF`, and the sibling DP/SCI paths): `i = int(TYPE.strip('SF')) - 1 - floor(log10|v|)` at arbitrary Python-int precision, then `f"{v:.{i}f}"`. Probed 2026-07-20 through the real upstream fn: `len(output)` grows linearly and UNCAPPED (10000000SF → a 10,000,001-char string). The crafted "9999999999SF" computes i ≈ 9,999,999,998 → Python attempts a ~10 GB string → MemoryError/DoS. `_format_DP`/SCI (`f"{v:.{i}f/e}"`) share the identical unbounded read.
- **Assessment**: the count-DoS is a SHARED latent defect (both engines mis-behave on the same crafted input), not a value-divergence. It is genuinely reachable pre-validation — the formatters run on parsed bytes before Rule 8 vets the TYPE. The clamp is bounded-output, not a semantics change: it only bites at counts > 30, which no legitimate AGS TYPE reaches, so every real value renders byte-identically (regression tests in both crates assert a `usize::MAX` count stays bounded AND that legit counts are unchanged).
- **Upstream-reportable**: **[YES]** — python-ags4's `_format_SF` / `_format_DP` / `_format_SCI` OOM on a crafted numeric-TYPE count (arbitrary-precision int → a billions-wide format). A malformed or hostile AGS4 file DoSes any python-ags4 caller that renders a value to its expected form (Rule 8 fixes, XLSX export). Candidate upstream report.
- **Our decision** (laterite-dev#610): clamp the count to `MAX_NUMERIC_COUNT` (30) at all six sites (laterite-ags4-types nDP/nSF/nSCI + laterite-ags4-excel Dp/Sci/format_sf) before it reaches a format width. A SEPARATE 0DP-integer truncation — laterite's `f as i64` SATURATES at ±i64::MAX where python-ags4's `f"{float(s):.0f}"` keeps full precision (e.g. 1E30 → 9223372036854775807 vs 1000000000000000019884624838656; 99999999999999999999 → 9223372036854775807 vs 100000000000000000000) — is a real value-divergence in the OTHER direction (we lose precision, python doesn't) hardened in laterite-dev#611 (see O-50), not this PR.

### O-50 [VARIANCE] 0DP integer CONVERSION: laterite range-guards an out-of-i64 value to Null (laterite-dev#611, was a fabricated i64::MAX); both validators already flag it, and python-ags4's numeric conversion keeps full precision
- **Observed**: a 0DP (integer) cell whose value cannot be a clean in-range i64 — a huge "1E30", a tiny "1E-30", a fractional "5.7". This is NOT a validation divergence: BOTH validators flag all of these via the identical strict Rule 8 regex `^-?\d+\.?$` (laterite's `is_ndp(s, 0)`). The divergence lives only in the SEPARATE string→number CONVERSION — laterite's `parse_value`/typed-read (which feeds `_content_hash`) vs python-ags4's `convert_to_numeric`.
- **python-ags4** (`AGS4.py::convert_to_numeric` / `int(float(s))`): arbitrary Python-int precision — "1E30" converts to the exact 1000000000000000019884624838656, "5.7"→5, "1E-30"→0. It never fabricates a value; conversion preserves. (Its Rule 8 VALIDATION separately flags the same cells; the two operations are independent.)
- **Us** (`laterite_ags4_types::parse_ags_integer`, the laterite-dev#611 single source for `parse_value` + `ags4_str` + laterite-py's PyO3 wrapper): the Integer arm used a saturating `f as i64`, so "1E30" became a FABRICATED 9223372036854775807 — a wrong number that was never in the file. laterite-dev#611 replaced that with a range guard: out-of-i64 → None (Null / Python `None`); in-range is untouched ("5.0"→5, "5.7"→5, "1E-30"→0), so `_content_hash` for all valid data is byte-identical. Finishes the laterite-dev#531 dedup (which single-sourced date/time/bool but left this Integer arm copied three ways).
- **Assessment**: empirically grounded — real geotech 0DP columns are single-digit to low-thousands; the one integer that grows, a cyclic-triaxial cycle count, is ~1e4 even in demanding tests, so i64 (~9.2e18) is never approached. The guard only fires on a ≥19-digit value, which in a whole-number column is an Excel/export error. Reject-to-Null surfaces that error (the value already carries a Rule 8 finding) instead of fabricating a wrong number, and leaves every genuine value + hash unchanged. Full-precision preservation (matching python's conversion) was considered and DEFERRED: it needs arbitrary-precision storage threaded through `_content_hash` + typed-read, and buys nothing when the validator already flags the giant value (a forward-looking note in `parse_ags_integer` records exactly what to change if that ever becomes necessary).
- **Upstream-reportable**: **[NO]** — python-ags4's numeric conversion is not defective here; it preserves precision, and WE were the lossy side (silent saturation). laterite-dev#611 fixes it on our side. Recorded for our own decision trail, and as the sibling of O-49 (the Class B count-DoS, where python-ags4 IS the reportable side).
- **Our decision** (laterite-dev#611): range-guard `parse_ags_integer` so an out-of-i64 0DP value converts to Null, not a saturated integer (the "reject overflow" option). Single-sourced across the leaf's `parse_value`/`ags4_str` and laterite-py so the typed-read object and the hash canonicalisation cannot drift (the laterite-dev#503 RL lesson). See O-49 for the sibling numeric-TYPE-count DoS.

## How to add an entry

Edit `observations.json` — **never `OBSERVATIONS.md`**, which is generated from it
and gated in CI (`tools/gen_observations.py --check`). Regenerate with
`uv run --no-sync python tools/gen_observations.py`.

Append under the current phase heading and use the next free `O-N`. Aim for the
house style — **observed / spec / assessment / upstream-reportable / our
decision** — but write what the case needs rather than padding to five: `Spec` is
meaningless for a laterite-internal fork, and `python-ags4` is the usual concrete
form of `Observed`. `--lint` reports departures without rewriting them, so the
convention guides new entries instead of being enforced retroactively over a
catalogue that predates it.

Set `upstream: true` for anything worth sending to the AGS Data Format Working
Group; the `## Upstream-reportable` table is rendered from that flag, so there is
no second list to keep in step. When an item is actually filed, add a
`- **Reported**: <url or ref> (<date>)` line so we don't double-file.

Each O-N also has a wiki page (`ags-wiki/observations/O-NN.md`) that links and
cross-references but never copies the fields. `--check-wiki` holds the two in
agreement.
