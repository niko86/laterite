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
- **Spec** (§4.1.1 Rule 19): *"A GROUP name shall **not be more than
  4** characters long and shall consist of **uppercase letters and
  numbers** only."*
- **python-ags4** (`check.py::rule_19`): flags when
  `len(name) != 4 or not name.isupper()`. Relative to the *prose*
  this is: (1) too strict on length (`!= 4` vs "not more than 4");
  (2) rejects all-digit names (`str.isupper()` is `False` with no
  cased chars); (3) never enforces the `[A-Z0-9]` charset so `LO-A`
  passes.
- **Evidence** (counted from the bundled dicts, 2026-05-15): of
  **148** groups in AGS4.1 and **171** in AGS4.2, **zero** are ≠ 4
  chars, **zero** contain a digit, **zero** are non-uppercase. The
  data dictionary the working group itself publishes never exercises
  any of the spec's stated allowances.
- **Assessment**: revised down from [BUG] → **[SPEC]**. python-ags4
  is enforcing the *de-facto/informal* rule ("exactly 4 uppercase
  letters") that the standard's own dictionary universally follows.
  It never misfires on real conformant data. The real issue is that
  the **spec prose and the data dictionary disagree** — the prose
  permits names the format never actually uses.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG tighten
  the Rule 19 wording to "shall be exactly 4 uppercase letters" so
  prose ⇄ dictionary ⇄ validators agree. The prose's "≤4 / letters
  and numbers" allowance is dead text — no standard group uses it.
- **Our decision (per user, revised — enforce de-facto + flag spec):**
  the validator enforces the convention the dictionary actually
  follows: **GROUP name = exactly 4 uppercase letters `[A-Z]`**
  (matching python-ags4's effective behaviour and 319/319 real
  groups). The looser spec prose is recorded here as the shortcoming
  to raise upstream. Verified: our rule_19 flags 0/319 standard
  groups (de-facto rule is exactly consistent with the dictionary).
  Unit tests pin the de-facto behaviour
  (`rule_19_short_name_flagged_de_facto`,
  `rule_19_digits_flagged_de_facto`,
  `rule_19_flags_too_long_and_punctuation`).

### O-7 [SPEC] Rule 19b_1's field-length limit isn't in the prose (but the dict obeys it)
- **Spec** (§4.1.1 Rule 19b): *"HEADING names shall start with the
  GROUP name followed by an underscore character. e.g. 'NGRP_HED1'."*
  No constraint on the field part beyond Rule 19a's overall ≤ 9.
- **python-ags4** (`rule_19b_1`): requires
  `len(item.split('_')[0]) == 4` **and `len(item.split('_')[1]) <=
  4`** — the second clause is a ≤4-char field-name limit found
  nowhere in the prose. It also only checks the prefix is *some*
  4-char token, not that it equals the GROUP (defers to 19b_2/3).
- **Evidence** (bundled dicts, 2026-05-15): of **1879** AGS4.1 and
  **2320** AGS4.2 headings, **zero** have a field part > 4 chars and
  **zero** lack an underscore. Same pattern as O-6 — the dictionary
  silently obeys an informal rule the prose doesn't state.
- **Assessment**: python-ags4's ≤4 field clause never misfires on
  real data; it's an undocumented but accurate encoding of the
  informal convention. The prefix==GROUP deferral is reasonable
  (needs the dict + the borrowed-heading exception, e.g. `FILE_FSET`
  inside non-FILE groups).
- **Upstream-reportable**: **[SPEC]** — same recommendation as O-6:
  AGS-DFWG should state the field-part ≤ 4 convention explicitly in
  Rule 19b rather than leave validators to infer it.
- **Our decision (per user, revised — enforce de-facto + flag spec):**
  19b_1 enforces the convention the dictionary follows: 4 uppercase-
  letter prefix + `_` + a **1–4 char `[A-Z0-9]` field part** (the
  python-ags4 field-≤4 constraint, which 4199/4199 real headings
  obey). The prose's silence on field length is recorded above as the
  spec shortcoming to raise. Verified: our rule_19b flags 0/4199
  standard headings. The prefix==GROUP / valid cross-group borrow
  semantic remains deferred to V8 (dict-aware), matching python's
  19b_2/3. Tests `rule_19b_enforces_de_facto_field_length` and
  `rule_19b_accepts_borrowed_heading_shape` pin this.

---

## V4 — dictionary-aware rules (Rules 7, 9)

### O-8 [BUG] python-ags4 rule_7_2 can raise IndexError on duplicate headings
- **python-ags4** (`check.py::rule_7_2`): builds `temp` = the
  reference (dictionary) heading list filtered to those used, which is
  inherently de-duplicated (the dictionary has no repeats). It then
  iterates the *file's* heading list with `enumerate` and indexes
  `temp[i]` unconditionally. If the HEADING row repeats a name such
  that the file list is longer than `temp` and the divergence isn't
  caught by an earlier `!=` break (e.g. headings `[A, B, B]` against
  dictionary `[A, B, C]`: i=0 A==A, i=1 B==B, i=2 → `temp[2]`), this
  raises `IndexError` and aborts the whole check run.
- **Spec** (§4.1.1 Rule 7): mandates dictionary order only; says
  nothing about duplicates (the duplicate-heading check is itself an
  *inferred* constraint — see O-9).
- **Assessment**: a latent defect — the unguarded `temp[i]` *can*
  `IndexError`. **But** when probed it does **not** fire under
  default python-ags4 1.2.0:
  `AGS4.check_file(..., rename_duplicate_headers=True)` — the default,
  and exactly what `ags4 check` does — renames a duplicate HEADING to
  `<NAME>_1` *before* `rule_7_2`, so the subset test fails first and
  the bad index is never reached (python instead emits Rule 7+9+18).
  Reachable only with `rename_duplicate_headers=False` (non-default).
  Real in the source, effectively unreachable via a HEADING-row
  duplicate in normal use.
- **Upstream-reportable**: **[BUG]** — `rule_7_2` should still
  bound-check `temp[i]`; latent but real (toggling the rename default
  to `False` exposes it).
- **Our decision**: `rule_7_2` is bounds-guarded — if the used-heading
  list is longer than the de-duplicated expected list we stop cleanly
  (the duplicate is the actionable finding, already raised by the
  duplicate-heading facet). Defensive against a bug python-ags4's own
  default currently shields. Pinned by
  `rule_7_flags_duplicate_heading`.

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
- **python-ags4** (`pick_standard_dictionary`): selects the standard
  dictionary from the file's `TRAN_AGS` DATA value, and **defaults to
  the latest (`4.1.1`)** when TRAN/TRAN_AGS is absent or unknown. It
  also raises a hard `AGS4Error` if there is neither a DICT group nor a
  resolvable standard dictionary.
- **Us**: `CheckOptions.dict_version` is explicit (default `4.2`); V4
  validates Rules 7/9 against that bundled edition and does **not** yet
  consult `TRAN_AGS`. A bundled standard dictionary is always present,
  so the "no dictionary available" hard-error path cannot occur — a
  robustness improvement.
- **Assessment**: a deliberate scope boundary. Auto-selecting the
  edition from `TRAN_AGS`, and the `TRAN_AGS` ⇄ dictionary consistency
  check, are Rule 14 territory (V6). Validating a 4.x file against the
  4.2 dictionary can yield edition-drift Rule 7/9 differences vs.
  python until V6 lands; acceptable for the phased build.
- **Upstream-reportable**: no — implementation choice, not a spec/
  python defect.
- **Our decision**: keep the explicit `dict_version` for V4; revisit
  `TRAN_AGS`-driven selection in V6 alongside Rule 14. The effective
  dictionary already overlays the file's own DICT group (consumed in
  V4, validated in V6), matching python's `combine_DICT_tables`
  ordering (standard dictionary first, file DICT appended).
- **RESOLVED (post-V8):** superseded by **O-30**. All five editions
  (4.0.3/4.0.4/4.1/4.1.1/4.2) are now bundled and
  `check_file` auto-selects from `TRAN_AGS` by default
  (`lib.rs::resolve_dict_version`); an explicit `--dict-version`
  still overrides. The real-data dogfood run (large.ags / 251.ags,
  both AGS 4.0) is what forced this — ~100 spurious edition-drift
  Rule 7/9/19b findings vanished once the right edition was used.

---

## V5 — typed-value rule (Rule 8)

### O-11 [SPEC] python-ags4 folds ID-uniqueness into Rule 8 (it's Rule 10a's job)
- **Spec** (§4.1.1 Rule 8): *"Data VARIABLEs shall be presented in the
  units of measurement and type that are described by the appropriate
  data field UNIT and data field TYPE…"* — Rule 8 is about a value
  conforming to its declared UNIT/TYPE. Uniqueness is **Rule 10a**:
  *"There shall not be more than one row of data in each GROUP with the
  same combination of KEY field entries."*
- **python-ags4** (`check.py::rule_8`): for a column whose TYPE is
  `ID` *and* whose name starts with the GROUP name, it additionally
  flags non-unique values **under Rule 8** (`duplicated(keep=False)`).
  The same defect is independently reported by `rule_10a` (V7), so a
  duplicate group ID is double-reported under both Rule 8 and Rule 10a.
- **Assessment**: an attribution over-reach — uniqueness isn't a
  UNIT/TYPE property, and Rule 10a already owns it. Not wrong as a
  *detection*, but the rule number is misleading and it inflates the
  Rule 8 count.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG / the
  python-ags4 maintainers move ID-uniqueness wholly under Rule 10a so
  each defect is reported once, under the rule that actually governs
  it.
- **Our decision**: mirror python-ags4's attribution in V5 (flag
  group-prefixed `ID` duplicates under Rule 8) **for finding-count
  parity**, and re-detect under Rule 10a in V7. Documented here so the
  intentional double-report isn't mistaken for a bug. Pinned by
  `rule_8_flags_non_unique_group_id`.

### O-12 [VARIANCE] DT/T validity engine differs from pandas
- **python-ags4** (`rule_8` DT/T arms): structural check via a
  per-char regex built from the UNIT (`fullmatch`), plus a semantic
  check via `pandas.to_datetime(..., format=…|'ISO8601')`. Timezone
  offset after `Z` is stripped before the semantic parse.
- **Us** (`typed_values.rs`): identical per-char structural matcher;
  semantic validity via `chrono` `Naive{Date,Time,DateTime}` over the
  AGS-permitted ISO-8601 shapes (same `Z`-strip). For an **unrecognised
  UNIT shape** we apply only the structural check and are lenient on
  semantics (we won't invent a calendar interpretation we can't
  justify); pandas would still attempt a parse.
- **Assessment**: behaviourally equivalent for every UNIT the AGS
  dictionary actually uses (`yyyy-mm-dd`, `…Thh:mm[:ss]`, `hh:mm`,
  `hh:mm:ss`). Divergence is possible only for non-standard UNIT
  strings, where "structural-only + lenient" is the defensible choice.
- **Upstream-reportable**: no — implementation choice, no spec/python
  defect. Recorded for parity-harness expectations.
- **Our decision**: keep the lean `chrono`-based check (no `regex`
  dependency; AGS patterns are tiny and hand-matched). `chrono` is
  already in the workspace lockfile via `ags5db`. **(O-33 corrects
  the scope: the chrono≈pandas equivalence for `yyyy-mm-dd` holds only
  for years within pandas' Timestamp range — out-of-range dates are
  now bounded to match python.)**

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
  (`format_nsf`) is ported from this workspace's own MIT `ags5db`
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
- **Spec** (§4.1.1 Rule 18): *"Each data file shall contain the DICT
  GROUP where non-standard **GROUP and HEADING** names have been
  included…"*
- **python-ags4** (`rule_18`): fires only when there is no DICT group
  **and Rule 9 already produced findings**. Rule 9 itself checks only
  *heading* membership (it never flags a non-standard GROUP code — see
  V4 notes). So a file with a non-standard GROUP whose headings happen
  to all resolve would not trigger Rule 18.
- **Assessment**: the reference implementation under-enforces the
  prose — "non-standard GROUP names" is in the text but nothing keys
  off it (Rule 9 only looks at headings). In practice a non-standard
  group almost always carries non-standard headings, so Rule 9 fires
  anyway; the gap is narrow but real.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG / python-
  ags4 make non-standard *GROUP*-name detection explicit (its own
  check feeding Rule 18), rather than relying on heading fallout.
- **Our decision**: replicate python's behaviour for V6 parity —
  `rule_18` follows Rule 9's output (run after `dictionary::check`).
  A dedicated non-standard-GROUP check is a candidate for a later
  phase; recorded here so the spec gap is on file. Pinned by
  `rule_18_follows_rule_9` / `rule_18_silent_without_rule_9`.

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

## V7 — relational rules (Rules 10a–10c, 11a–11c)

### O-21 [SPEC] Rule 10c's parentless-group list is hardcoded, not dict-derived
- **Spec** (§4.1.1 Rule 10c): *"Every entry made in the KEY fields in
  any GROUP must have an equivalent entry in its PARENT GROUP."* The
  AGS dictionary encodes the parent in `DICT_PGRP`.
- **python-ags4** (`rule_10c`): skips a **hardcoded** set —
  `PROJ, TRAN, ABBR, DICT, UNIT, TYPE, LOCA, FILE, LBSG, PREM, STND` —
  rather than deriving "parentless" from `DICT_PGRP`. Verified against
  the bundled 4.2 dictionary: `LOCA`'s `DICT_PGRP` is **`PROJ`** (not
  `-`), yet a LOCA row carries no PROJ key, so a dict-derived check
  would emit a bogus *"PROJ_ID defined as KEY in the parent group
  (PROJ) but not in the child"* for **every** file containing LOCA.
  The hardcoded list exists precisely because the dictionary's
  `DICT_PGRP` does not fully encode *checkable* parent linkage
  (LOCA→PROJ is implicit/singular, not a repeated-KEY relation).
- **Assessment**: the list is necessary for correctness, but it is a
  maintenance hazard — a new root/implicitly-linked GROUP in a future
  AGS edition would need a python-ags4 code change, not just a
  dictionary update. The standard would be cleaner if `DICT_PGRP`
  (or a new flag) marked "no checkable parent" explicitly.
- **Upstream-reportable**: **[SPEC]** — recommend AGS-DFWG make
  parentless / implicit-link status a dictionary property so Rule 10c
  is data-driven, not hardcoded.
- **Our decision**: replicate python-ags4's exact list (`PARENTLESS`
  in `relational.rs`) for parity + correctness. Documented as the
  spec gap to raise. Pinned by `rule_10c_flags_orphan_child_row`.

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
- **Spec** (§4.1.1 Rule 11/11c): a Record Link is *"The GROUP name
  followed by the KEY FIELDs … in the order presented in the AGS4
  DATA DICTIONARY"* and must *"cross-reference to the KEY FIELDs of
  data rows in the GROUP referred to"*.
- **python-ags4** (`fetch_record`): matches the link's value list
  against the target group's **leading columns positionally**
  (`columns[1:][0:len-1]`), not against the dictionary-defined KEY
  fields. For a well-formed group whose leading columns *are* its KEY
  fields these coincide; they diverge if a group's KEY fields aren't
  its first columns.
- **Assessment**: a simplification in the reference impl. It works for
  conformant files (KEY fields lead the group by convention) but
  isn't literally "cross-reference to the KEY FIELDs".
- **Upstream-reportable**: **[NOTE]** — python-ags4 could resolve via
  the dictionary KEY fields for strictness; in practice equivalent.
- **Our decision**: replicate the positional match (`fetch_count`) for
  parity. Recorded so the semantic gap is on file. Pinned by
  `rule_11c_flags_bad_record_link`.

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
- **python-ags4** (`rule_19b_2`, `rule_19b_3`): both iterate *every*
  heading and split on `_`. For a heading with no underscore (or a
  bad structure) `rule_19b_1` (our V3) already fired, `rule_9` (V4)
  already fired, yet `rule_19b_2` adds *"Group X … could not be
  found"* and `rule_19b_3` adds *"… does not start with the name of
  this group …"* — the same defect reported up to three times under
  Rule 19b plus once under Rule 9.
- **Assessment**: redundant. The borrowed-heading semantic 19b_2/19b_3
  add over 19b_1 is only meaningful when the prefix names *another*
  real group; for malformed headings it is noise.
- **Upstream-reportable**: **[NOTE]** — python-ags4 could gate
  19b_2/19b_3 on "prefix ≠ group and heading has an underscore".
- **Our decision** (revised Stage 9c): the **prefix-not-a-group**
  case now emits two findings — `rule_19b_2`-style "Group X referred
  to in Y could not be found..." AND, when the heading isn't defined
  anywhere, `rule_19b_3`-style "X does not start with the name of
  this group, nor is it defined in another group." The two
  messages target *different fixes*: the first hints at a prefix
  typo, the second at a placement mistake — diagnostic value the
  original (pre-9c) consolidation lost. We still don't emit
  python-ags4's third redundant variant (the "Heading X is more
  than 9 chars" overlap with 19b_1), so we're at 2 findings vs
  python's 3 on a malformed heading — half-revert, not full
  consolidation.

### O-27 [NOTE] Rule 20 on-disk checks are implemented as opt-in (`--check-files`)
- **python-ags4** (`rule_20`): besides the data-level check (every
  `FILE_FSET` used must be defined in the FILE group), it also stats
  the filesystem — a `FILE/` sub-folder beside the `.ags`, a
  `FILE/<fset>/` per defined FSET, and each `FILE_NAME` on disk.
- **Us**: the **data-level** check always runs. The **on-disk** half
  is now implemented too, as `references::rule_20_on_disk`, gated by
  `CheckOptions.check_files` (CLI `lat-check --check-files`,
  `std::fs` only). **Default off**: a library validator must stay
  deterministic and path-independent and `db-to-ags4 --validate`
  validate` turns it **on** by default (`--no-check-files` to opt
  out) so the dogfood matches python-ags4's always-on stat.
- **Assessment**: no longer a standing variance. With `check_files`
  on, Rust and python-ags4 **agree** on Rule 20 (both flag a missing
  `FILE/` tree; both clean when it is present); with it off, only the
  portable data-level core runs — a deliberate, *documented opt-out*,
  not an unacknowledged scope gap. (The earlier framing of an "out of
  scope" decision was not one the maintainer had knowingly taken;
  implementing the check resolved it.)
- **Upstream-reportable**: no — implementation/scope choice.
- **Our decision**: data-level Rule 20 always + on-disk Rule 20
  opt-in; the corpus-qa dogfood enables it, so the prior
  `parity.rs` O-27 reconcile arm + its unit test were **removed** (no
  longer a divergence). `db-to-ags4` (Rust `attachments.rs` + Python
  `blobs.py`) reconstructs the `FILE/<FILE_FSET>/<FILE_NAME>` sidecar
  tree from stored blobs (FSET recovered via `blob.parent_id =
  v_file.id`; orphan → flat + warn) so an exported delivery passes
  `lat-check --check-files`. Pinned by
  `rule_20_on_disk_opt_in_and_default_off` + the attachment
  round-trip e2e / `test_cli` tests.

### O-28 [VARIANCE] External `--dict` override deliberately deferred beyond V8
- **Plan**: listed an `lat-check --dict <path>` runtime override.
- **Reality**: [`Dictionary`] is `'static` phf-backed (zero-startup,
  compiled-in). A runtime-parsed dictionary needs an owned variant and
  a non-`'static` lifetime threaded through `DictEntry`/`GroupMeta`
  and every rule module — a broad, mechanical, regression-prone change
  across V3/V4/V7/V8 for a power-user feature that
  `db-to-ags4 --validate` (bundled 4.2) does not need.
- **Assessment**: doing this risky refactor at the final phase trades
  real regression risk for marginal value. Per the methodology
  ("escalate rather than cross a line / over-reach"), this is recorded
  as a conscious scope decision, not a silent skip. The flag is
  plumbed and returns a clear `BadDict` error (exit 5) pointing at
  `--dict-version`; nothing is silently ignored.
- **Upstream-reportable**: no.
- **Our decision**: keep bundled 4.1/4.2 only. If a real consumer
  needs custom dictionaries, the clean implementation is a
  `Cow`/enum-backed `Dictionary` (bundled vs parsed) — a focused
  follow-up, not an end-of-project bolt-on. This supersedes the
  earlier "wired in V8" note on [`CheckOptions::custom_dict`].

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
- **Context**: resolves **O-10**. We now bundle all five AGS4
  editions python-ags4 ships and `resolve_dict_version` picks one per
  file (explicit `--dict-version` overrides). python-ags4's
  `pick_standard_dictionary` uses a fixed exact-string map
  (`STANDARD_DICT_FILES`) and `LATEST_DICT_VERSION = "4.1.1"`.
- **Deliberate divergences** (user-approved):
  1. **`4.0` → 4.0.4** (newest bundled 4.0 patch). python maps the
     bare `"4.0"` → `4.0.3` (the *oldest*). AGS versioning is
     traditionally major.minor, so a file tagged `4.0` is best served
     by the latest 4.0.x schema. Generalised: an exact bundled string
     wins (so `4.1`→4.1, `4.1.1`→4.1.1, `4.2`→4.2 keep python-parity);
     otherwise the newest bundled patch of that major.minor
     (`4.0`→4.0.4, `4.1.5`→4.1.1, `4.2.7`→4.2).
  2. **AGS 3.x → hard `UnsupportedEdition` error.** python silently
     falls 3.x back to 4.1.1 and validates it against an AGS4 schema.
     Nothing is specced for AGS3 here; we refuse. **Amended
     2026-05-16:** AGS3 is now detected at *parse* by its
     unambiguous signature (`**GROUP` / `<UNITS>` / `<CONT>`) and
     raised as `UnsupportedEdition { found: "3.x (AGS3 format)" }` —
     a clear edition error, not the misleading generic
     `NotAgs4("no GROUP rows found")` it used to fall through to.
     (The TRAN_AGS `major == 3` path also still → `UnsupportedEdition`
     for the rare AGS3 file that has `GROUP` rows.) The corpus-QA
     parity classifier folds "Rust refuses AGS3 + python validated it
     (typically Rule 3)" into **`KNOWN_DIVERGENCE` (O-30)** so the 57
     *expected* AGS3 divergences in a real 12.5k corpus leave the
     parity ACTION list instead of swamping it as `VALIDITY_DISAGREE`.
  3. **bare `TRAN_AGS = "4"` (major-4, no usable minor: `"4"`,
     `"4."`, `"4.x"`) → the 4.0 line (4.0.4).** *Amended 2026-05-16,
     reverses the original "matched python" stance below.* python has
     no bare-`4` key so it → `4.1.1` (latest). A 12,503-file real
     dogfood run showed **41% declare bare `"4"`**, and the *same
     producers'* `"4.0"` files already resolve to 4.0.4 — i.e. ~5,100
     4.0-era files were being mis-editioned to 4.1.1. "4" colloquially
     means AGS4(.0); the original/most-common line is the safer,
     deterministic, per-file choice. (Dynamic/statistical/
     findings-minimising selectors were considered and rejected: they
     break per-file determinism, `--seed` reproducibility and the
     python-parity premise, and "fewest findings" masks real defects.)
- **Matched on purpose**: *truly* missing / `None` / non-numeric /
  `major != 4` / an **explicit unbundled numeric minor** (`4.3`,
  `4.9`) → **4.1.1**, *exactly* python's `LATEST_DICT_VERSION`, so
  dogfood parity divergences there are real defects, not fallback
  artefacts. (Bare `"4"` was in this set until 2026-05-16 — see
  divergence #3.)
- **Assessment**: bare-`4.0`→newest-patch, bare-`4`→4.0-line, and
  AGS3-explicit-hard-fail are intentional, data-driven improvements
  over the reference impl, not defects; the remaining fallback is
  deliberately python-identical.
- **Upstream-reportable**: **[VARIANCE]** — worth noting to AGS-DFWG
  that mapping bare `"4.0"` to the *oldest* 4.0 patch is surprising;
  newest-patch is the safer default.
- **Our decision**: as implemented + unit-tested
  (`lib.rs::resolve_dict_version` table tests). The corpus-QA report
  records `dict_used` per file so a batch run shows the corpus's
  edition mix and *why* each file was judged against a given schema.
  It now also records `dict_resolution`
  (`forced`/`exact`/`guessed`/`fallback`) so a batch distinguishes a
  genuine `TRAN_AGS` edition from this fallback — the blind spot O-31
  surfaced (294 fallback files were indistinguishable from genuine
  4.1.1). `resolve_dict_version` returns `(DictVersion, DictResolution)`.
  Older 4.0.x dictionaries are Latin-1 (cp1252); `build.rs` decodes
  them byte→char (ISO-8859-1) — lossless, dependency-free, and exactly
  the 0–255 tolerance **O-1** documents. The bare-`"4"`→4.0.4 and
  AGS3-detection changes are unit-tested
  (`lib.rs::resolve_*`, `parse::tests::ags3_is_unsupported_edition*`,
  `parity::tests::ags3_*`). Expected delta on the next full
  `sandbox/` re-run: ~5,100 files move `4.1.1 (fallback)` →
  `4.0.4 (guessed)`; parity ACTION list ≈69 → ≈12 (the 57 AGS3 →
  `KNOWN_DIVERGENCE`, the ~12 `NotUtf8` remain a real disagreement —
  resolved in **O-32**: they now AGREE on a Rule 1 error).

### O-31 [VARIANCE] Rule 8 — empty `DT` UNIT now flagged (python parity; closes the O-12 degenerate gap)
- **Observed**: an 807-file real-share dogfood run. python-ags4 flags
  Rule 8 on a value like `TRAN_DATE = 2025-02-24` when that heading's
  `UNIT` is **empty** (`Value 2025-02-24 in TRAN_DATE does not match
  the specified format () or is an invalid date/time.`). Rust stayed
  *clean* on the same file — a real false negative: `structural_dt_match`
  returned `true` on an empty UNIT ("no declared format → nothing to
  fail").
- **Spec**: Rule 8 — a DATA value must match its declared TYPE/format.
  python builds a per-char regex from the UNIT; an empty UNIT → empty
  pattern → `''.fullmatch(non_empty)` is `False` → flagged. i.e. python
  treats "no declared format" as "no non-empty value can match".
- **Assessment**: distinct from **O-12** (non-empty *unrecognised*
  UNIT shapes stay lenient — they can't be calendar-checked). The
  empty UNIT is the degenerate case O-12 never covered; **O-19** is
  precedent for empty-cell special handling. Rust's old leniency here
  was a genuine divergence (false negative), not a deliberate variance.
  Scope: **DT only** — `T` (elapsed time) with an empty UNIT is a
  possible follow-up (not seen in the corpus; left lenient for now).
- **Upstream-reportable**: **[VARIANCE]** — python's message text
  (`format ()`) is awkward, but flagging a value whose heading
  declares no format is defensible; an empty UNIT on a `DT` heading is
  itself a likely producer-side data defect worth noting to AGS-DFWG.
- **Our decision**: `structural_dt_match`'s empty-UNIT branch now
  returns `value.is_empty()` — a non-empty value with no declared
  format fails structurally → Rule 8, matching python. Fixture
  `tests/fixtures/rule8_dt_empty_unit.ags` + regression
  `rule8_empty_unit_dt_flags_like_python`. Recognised-UNIT and
  non-empty-unrecognised-UNIT (O-12) behaviour is unchanged.

### O-32 [VARIANCE] Non-UTF-8 input is decoded lossily, not refused (mirrors python's `errors="replace"`; closes the `NotUtf8` black hole)
- **Observed**: a 12,503-file dogfood run (`sandbox/`). 12 real AGS4
  deliveries are cp1252/Latin-1 (`°`/`±`/`µ`/smart-quotes). The Rust
  validator hard-failed them as `NotUtf8` — **zero rules evaluated**,
  surfacing as `VALIDITY_DISAGREE` in parity. python-ags4 never
  hard-fails on encoding: `AGS4.py:771` opens
  `encoding='utf-8', errors="replace"`, so an undecodable byte becomes
  `U+FFFD` and the file still validates.
- **Spec**: Rule 1 — "the file shall be entirely composed of ASCII
  characters." python interprets 0–255 leniently (the same 0–255
  tolerance **O-1** documents) but it *reports* the violation: with the
  default utf-8 decode a replaced byte is `U+FFFD` (code point 65533),
  `is_ags_ascii` (`ord ≤ 255`) is `False`, so `check.py:rule_1` emits
  **`"AGS Format Rule 1"`**. A real `ags4 check file.ags` (python's own
  CLI default — no auto-detect, no fallback) therefore *reports a
  Rule 1 error* on these files; it does not silently accept them.
- **Assessment**: refusing the input outright was the **only** real
  divergence from the reference — and it is worse than python (a black
  hole vs a finding). The wrapper's `--encoding-fallback` cp1252 retry
  was **dead code** (`errors="replace"` never raises
  `UnicodeDecodeError`), so the wrapper was *faithful to python's
  default by accident*. Making the wrapper probe→`cp1252` (the
  originally-planned fork) would have made it diverge from python's own
  CLI — i.e. *mask* real behaviour — so it was rejected. BOM is
  explicitly out of scope: a UTF-8 BOM is valid UTF-8 → leading
  `U+FEFF` (>255) → Rule 1 at line 1, which is exactly what python does
  too (`check.py:372-377`), so no action needed.
- **Upstream-reportable**: **[VARIANCE]** — flag to AGS-DFWG that
  python's `errors="replace"`→`U+FFFD` *erases the original byte*: two
  different cp1252 files can collapse to byte-identical Rule 1 output,
  and the user is never told the file is most likely cp1252 (the
  `AGS4.py:833-839` hint only fires when a finding already exists).
  Also that `is_ags_ascii` admits code points 128–255, which a strict
  reading of Rule 1 (ASCII = 0–127) forbids — python's own documented
  leniency, mirrored here for parity (see O-1).
- **Our decision**: `parse_file` now decodes with
  `String::from_utf8_lossy` — the exact stdlib twin of python's
  `open(…, errors="replace")`. Valid UTF-8 takes the `Cow::Borrowed`
  fast path (byte-identical, no rebuild); invalid bytes → `U+FFFD` →
  `rule_1`'s >255 arm → **`AGS Format Rule 1`** (independent of
  `--show-fyi`), so the 12 files now **AGREE with python on a Rule 1
  error** instead of `VALIDITY_DISAGREE`. Correctly UTF-8-encoded
  extended chars (e.g. `0xC2 0xB0` → `U+00B0`, ≤255) stay the tolerated
  Rule 1 FYI (suppressed by default, O-1) — correct encoding is
  rewarded, mis-encoding is flagged. `ValidatorError::NotUtf8` is
  **kept but unraised** (public-API/back-compat; downstream `match`
  arms stay exhaustive). The python wrapper is **cleanup-only** (dead
  `except UnicodeDecodeError` removed, `--encoding-fallback` re-doc'd
  inert) — **no behaviour change, no cp1252 probe**; no Rust/parity
  change. No-masking audited: the wrapper's sole stdout transform is
  the symmetric (O-1) rule-key filter; its one non-FYI drop
  (`Validator Process Error`, AGS3 termination) is already
  O-30-reconciled, so no `AGS Format Rule N` finding is hidden. Tests:
  `parse::tests::{invalid_utf8_input_is_decoded_lossily_not_rejected,
  valid_utf8_extended_char_is_byte_faithful}` (inline temp files — no
  non-UTF-8 fixture, which would trip corpus-qa's e2e
  `hard_error==0`), `regression::{invalid_utf8_input_flags_rule1_not_hard_error,
  valid_utf8_extended_char_is_fyi_only_not_rule1}`.

### O-33 [VARIANCE] Rule 8 — DT/datetime bounded to pandas' Timestamp range (closes the value-range gap O-12 missed)
- **Observed**: a 5,492-file parity dogfood of the independent
  `sandbox\corpus` (run `20260516T123345Z`). 8 files were
  `PYTHON_ONLY` Rule 8; the root cause is a *single* data defect
  replicated across delivery copies — `LOCA_STAR = 0018-06-03` (line
  1621; cf. `2025-06-08` one row above — a data-entry error). python
  flags Rule 8, Rust stayed clean.
- **Spec**: Rule 8 — a DATA value must match its declared TYPE/format
  *and* be a valid date/time. `0018-06-03` matches `yyyy-mm-dd` and
  *is* a valid proleptic-Gregorian date, so a strict-spec reading
  makes Rust's silence defensible — Rust was **not** wrong by the
  letter of the rule.
- **Assessment**: the divergence is python's engine, not the spec:
  `check.py:770` runs `pd.to_datetime(..., format=…, errors='coerce')`,
  and pandas' `Timestamp` range is **1677-09-21 .. 2262-04-11** — any
  date outside it becomes `NaT` → Rule 8. `chrono::NaiveDate` accepts
  any year, so Rust passed it. **O-12** asserted chrono≈pandas "for
  every UNIT the AGS dictionary actually uses (`yyyy-mm-dd`, …)" — true
  only for *in-range* years; this is the value-range counterexample
  O-12 never captured. An `0018` year in a 2025 geotechnical survey is
  unambiguously corrupt data.
- **Upstream-reportable**: **[VARIANCE]** — pandas' Timestamp range is
  an implementation artifact, not an AGS requirement; flag to AGS-DFWG
  that python-ags4 silently rejects spec-valid pre-1678 / post-2262
  dates (and conversely that the validator *should* flag implausible
  years — the useful side-effect).
- **Our decision**: match python (user call — flagging the bad data is
  the right validator behaviour, consistent with **O-31**'s "match
  python's Rule 8"). `dt_semantic_ok` (`typed_values.rs`) now bounds
  recognised date/datetime values to the pandas range via
  `in_pandas_range` + `PANDAS_MIN/MAX` consts (mirrored from pandas'
  public `Timestamp.min/max` docs — clean-room behavioural constant,
  not ported from `check.py`; sub-second tail dropped — exact at AGS's
  ≤second resolution for every realistic value). A date-only value is
  lifted to midnight first, so `1677-09-21` (00:00 < the 00:12:43 min)
  fails just as python's NaT does. Time-only units (`hh:mm`,
  `hh:mm:ss`) are unaffected (python's `pd.to_datetime` on a bare time
  gets an in-range default date). The 8 corpus files now **AGREE** (no
  parity reconcile arm — equal rule sets). Tests:
  `typed_values::tests::dt_semantic_bounds_to_pandas_range`, fixture
  `tests/fixtures/rule8_dt_out_of_range.ags` (a normal *findings*
  file, never a hard error — keeps the corpus-qa e2e `hard_error==0`
  invariant), regression `rule8_date_out_of_pandas_range_flagged`.

### O-34 [VARIANCE] `NotAgs4` ↔ python "missing mandatory groups" is a KNOWN_DIVERGENCE
- **Observed**: same 5,492-file dogfood. 8 files were
  `VALIDITY_DISAGREE`: Rust returned `NotAgs4`, python emitted the
  mandatory-group rules (13/14/15/17, ± line-format noise 3/4/5/19a/1).
  Every sampled file is a tab-delimited Excel "save as text" export
  (`GROUP⇥PROJ…`) or empty — **zero spec-valid quoted `"GROUP"`
  rows**.
- **Spec**: AGS4 Rule 3/4/5 mandate comma-separated, double-quoted
  fields. A tab-delimited or empty file has no spec-valid GROUP row →
  it is genuinely *not* AGS4 transfer format.
- **Assessment**: Rust's `NotAgs4("no GROUP rows found")`
  (`parse.rs:214`, guarded by `group_order.is_empty()`; AGS3 markers
  excluded → O-30) is the correct, *more informative* verdict. python
  has no refuse path, so it mislabels the file as merely "missing
  PROJ/TRAN/TYPE/UNIT". This is the **exact O-30 shape** one
  structural level up (Rust refuses; python mis-validates).
- **Upstream-reportable**: **[VARIANCE]** — python-ags4 silently
  "validating" a tab-delimited or empty file as Rule 13/14/15/17 is
  misleading; suggest python detect non-CSV / empty input and refuse
  like it (eventually) does for AGS3.
- **Our decision**: parity-classifier-only, **no parser/validator
  change** (refusing non-AGS4-CSV is correct). `parity.rs::classify`'s
  `HardError` arm now maps `v == "NotAgs4"` **and** python having all
  three mandatory groups absent (`Rule 13 && Rule 14 && Rule 17`) →
  `KnownDivergence{observation:"O-34"}`, keeping these out of the
  ACTION list — exactly analogous to the O-30 `UnsupportedEdition`
  arm. The triple-rule guard keeps it narrow: a genuine
  `NotAgs4`-vs-real-findings disagreement (python sees *some*
  structure) still falls through to `ValidityDisagree`. Test:
  `parity::tests::o34_notags4_vs_missing_groups_is_known_divergence`
  (positive + a negative-guard case).

### O-35 [NOTE] Presence-only `reconcile` can't whittle a python parse-layer cascade
- **python-ags4**: its parsing layer turns one malformed construct
  into a *multi-rule* result — a lone embedded CR → universal-newline
  record split → Rule 2a+3+5 (`rule_6` itself is a no-op, O-2); a
  valid extended char → Rust FYI-only / python silent (O-1); an
  unquoted field → python Rule 3 *or* 4 by position vs Rust Rule 5
  (O-3). Established via probe runs across both validators.
  documented rule-swaps (O-2/O-3/O-26) and only when the *entire*
  symmetric diff is consumed, so a cascade leaves residue and a known
  root cause classifies as a false `RUST_ONLY`/`PYTHON_ONLY` ACTION.
- **Assessment**: a real limitation of presence-only parity, not a
  validator defect. Generic widening is unsafe (Rules 2a/3/5/9/18
  fire for many legitimate reasons); only *signature-narrow* arms
  (à la the O-34 triple-guard) are acceptable.
- **Upstream-reportable**: no — harness/methodology.
- **Our decision**: document it; do **not** broaden `reconcile`
  generically. Signature-narrow arms — `rust=={Rule 6} ∧
  py⊆{2a,3,5} → O-2`; `rust=={FYI 1} ∧ py==∅ → O-1`; `rust⊇{5} ∧
  py⊇{3} → O-3` — are the sanctioned follow-up.

### O-36 [NOTE] Parity differential is triage-biased by default
  `triage ∪ reservoir(rest, --parity-sample)` and `--parity-sample`
  **defaults to 0**, so by default only files the Rust side already
  flagged odd (HardError/Panic/`surprising`) are cross-checked against
  python-ags4. A file Rust handles confidently-but-wrongly (plausible
  `Findings`) is never sent to the oracle — silent agreement on a
  wrong verdict is invisible. Corollary: "N-file corpus mostly
  AGREEs" says nothing about rules with *zero* differential evidence
  (the `strat-parity-matrix` blind-spot list quantifies that
  complement).
- **Assessment**: a sampling bias in the dogfood, not a validator
  defect — but it overstates the strength of the parity claim.
- **Upstream-reportable**: no — harness/methodology.
- **Our decision**: keep `--parity-sample` (perf), but treat
  triage-only as the floor, not the ceiling: a non-zero default
  sample (or an explicit "differential is triage-only" banner) + a
  per-rule "rules with zero parity evidence" report are the
  sanctioned follow-ups; the per-rule matrix
  (`parity_matrix_dogfood`) is the first instalment.

### O-37 [VARIANCE] Native parser is lenient where python-ags4 raises hard
- **python-ags4** (`AGS4.py::AGS4_to_dict`, AGS4.py:67-180): raises
  `AGS4Error` on three structural anomalies *before* returning data
  — duplicate `GROUP` lines for the same group code, DATA rows with
  a field count ≠ HEADING row, and (when `rename_duplicate_headers=
  False`) duplicate headings. Read fails; nothing downstream sees the
  bad row.
- **Us** (`rust-packages/laterite-ags4-validator/src/parse.rs`): the native
  parser is deliberately lenient — duplicate GROUP declarations are
  silently merged into one row bucket; ragged DATA rows pass through
  (extra fields dropped or short fields padded — finding-reportable
  via Rule 4); duplicate headings rename with a warning when
  `_rename_dups` is invoked in compat. The parser's job is to return
  *something* the validator can then catalogue.
- **Assessment**: not a bug — opposite design philosophies. The
  native validator's value is *reporting* problems through Findings;
  crashing on malformed input would mean a bad file produces no
  report at all. python-ags4's strictness is appropriate for its
  "first crash, then findings" pipeline. Native lenience is
  appropriate for ours ("findings first, never crash").
- **Upstream-reportable**: no — design choice, not a bug.
- **Our decision**: keep native lenient. `laterite.compat` (the
  python-ags4 drop-in surface) interposes a `_strict_pre_check` pass
  (`packages/laterite/python/laterite/compat.py:_strict_pre_check`)
  that scans the raw file via `csv.reader` and raises `Ags4Error`
  for the three cases above, matching python-ags4's wording closely
  enough that `pytest.raises(AGS4Error, match=...)` against their
  test suite passes. Native callers (`laterite.Validator`,
  `lat-check`) never hit the pre-check.

### O-38 [SPEC] Rule 8 DT validation: python-ags4 forbids non-ISO UNITs
- **python-ags4** (`check.py::rule_8`, DT branch): builds two masks
  per declared DT UNIT:
  - **mask2** (regex): each `y/m/d/h/s` in the UNIT becomes `\d`,
    `+` becomes `[+-]`, anything else is a literal — `str.fullmatch`
    against the value (the structural shape check).
  - **mask1** (pandas): `pd.to_datetime(value, format='ISO8601')`
    must succeed — except for `hh:mm` / `hh:mm:ss` units which use
    explicit `%H:%M[:%S]`.
  A value is flagged unless **both** masks pass. The mask1 fallback
  to `ISO8601` for any non-time UNIT means a value like `01/12/2020`
  under UNIT `dd/mm/yyyy` is **structurally fine** but mask1 fails
  (not ISO-8601), so it's flagged. Effect: python-ags4 cannot
  validate **any** non-ISO UNIT — `dd/mm/yyyy`, `dd-mm-yyyy`,
  `dd.mm.yyyy`, `mm/dd/yyyy`, `mm-dd-yyyy`, `dd/mm/yy`, `mm-yyyy`,
  `mm/yyyy` all become un-validatable in practice (every value, valid
  or invalid, flags).
- **Spec** (AGS4.1 §4.1.1 Rule 8): "Data variables shall be of the
  data type specified in the TYPE row. UNIT row specifies the units
  in which the variable is expressed." The DT type with a UNIT of
  `dd/mm/yyyy` is a legitimate, documented usage; the spec does not
  require ISO-8601 — the UNIT itself declares the format.
- **Assessment**: a python-ags4 implementation defect, not a
  laterite divergence. The pandas-pinning to `ISO8601` is convenient
  but inverts the spec's "UNIT declares the format" contract for
  any non-ISO shape. Evidence: an empirical DT-format matrix run
  against both validators recorded 11 PY_ONLY divergences, all
  "python-ags4 wrongly flags a valid value".
- **Upstream-reportable**: **[SPEC]** — python-ags4's `rule_8` DT
  branch should translate the UNIT pattern into a `pd.to_datetime`
  `format=` string rather than hard-coding `ISO8601`. Open candidate
  for an AGS-DFWG / python-ags4 issue (priority: high — affects
  real European/US delivery files).
- **Our decision**: laterite implements the spec-correct path. New
  `lex_unit_value` in `rust-packages/laterite-ags4-validator/src/rules/typed_values.rs`
  walks the UNIT pattern token-by-token (yyyy/yy/mm/dd/hh/ss with
  context-sensitive `mm` = month-or-minute), extracts calendar
  fields from the value, and validates ranges + pandas bound
  (still O-33 for the year). Bonus closures (matrix-driven):
  - `yyyy` (year-only) now accepted (was a RUST_ONLY).
  - Leap-second `hh:mm:ss / xx:xx:60` tolerated (matches chrono `%S`).
  - All European/US date-format UNITs validated correctly.
  Probe verifies: 34/45 AGREE (was 27/45 pre-fix); the 11 residuals
  are all this divergence — laterite correct, python-ags4 wrong.

### O-39 [SPEC] Rule 10c — empty parent KEYs are "no entry", not a missing link
- **Spec** (`spec:AGS4-4.2-2025.pdf §4.1.1 Rule 10c`): *"Every entry
  made in the KEY fields in any GROUP must have an equivalent entry
  in its PARENT GROUP."* The text hinges on *"entry made"* — an
  empty cell can be read as either "an entry that points to nothing"
  (strict) or "no entry made" (permissive).
- **python-ags4** (`check.py::rule_10c`): permissive — rows whose
  parent KEY columns are *all* empty are skipped; they're treated
  as intentionally standalone. Fixture
  `tests/test_files/Standalone_SAMP_IDs.ags` (lab-control SAMP rows
  with no LOCA_ID) is asserted clean.
- **Us** (pre-Stage 9b): strict — laterite flagged every such row
  with `No parent entry in LOCA for KEY combination:` (note the
  empty tuple). Real geotech workflows produce these legitimately
  (lab controls, off-site samples, calibration runs with no
  borehole), so the strict reading was producing noise.
- **Assessment**: spec is ambiguous; the geotech-domain reading
  is python-ags4's. Aligning is the right user-experience call.
- **Upstream-reportable**: **[SPEC]** — the AGS4 spec text should
  explicitly say whether empty KEY cells participate in Rule 10c's
  link requirement. Either reading defensible; clarity beats either.
- **Our decision** (Stage 9b): align with python-ags4's permissive
  reading. `rules/relational.rs::rule_10c` now skips a child row
  when all of its parent KEY tuple values are empty (`trim().is_empty()`
  on every component). A row with even one non-empty parent KEY
  still gets the full check. New test
  `rule_10c_skips_rows_with_all_empty_parent_keys` pins the
  behaviour. Native API + Rust CLI also benefit (fewer false
  alarms on real corpora). Closes `test_file_with_standalone_SAMP_IDs`.

---

### O-40 [NOTE] The `.ags.idx` byte index records true GROUP line-starts (the csv reader's record positions were off-by-one for CRLF, and absorbed leading blanks)
- **Observed**: the byte-offset index in the `.ags.idx` certificate
  recorded each `"GROUP",…` section's start from the `csv` crate's
  `StringRecord::position().byte()`. That offset is the byte where the
  reader *enters* a record, which for a CRLF-terminated previous line is
  the preceding `\n` — one byte before the GROUP record's true
  line-start. It also absorbs leading blank lines, reporting the first
  GROUP at byte 0 instead of after the blanks. The independent,
  hand-computed `byte_offset_ground_truth.rs` oracle pins both:
  `two_group_crlf` TRAN true=70 / csv=69; `leading_blank` PROJ true=2 /
  csv=0. LF / BOM / quoted-embedded-newline files already matched.
- **Spec**: the `.ags.idx` sidecar is *our* format, not AGS-specced — but
  Rule 2a mandates CRLF terminators, so the off-by-one was the **common
  real-world case**, not an edge. The loose offsets were still *valid*
  certs: a slice taken at the csv offset keeps a harmless leading
  `\n`/blank that the reparse skips, which is exactly why the
  `slice_parity` consistency tests never caught the looseness.
- **Assessment**: a cert should record where a group's bytes *actually*
  start, so a sliced read or a remote ranged-GET lands precisely on the
  `"GROUP"` record. The shared parse leaf (`laterite_ags4_parse`, #168)
  already emits a source-true `group_byte` from its one-pass byte walk;
  sourcing the index from it removes the divergence by construction.
- **Upstream-reportable**: none — this is `csv`-crate record-position
  semantics plus our own cert format, not an AGS4-spec or python-ags4
  matter.
- **Our decision** (#168 Phase 4): `index_ags4_bytes` now sources GROUP
  offsets from the parse leaf's source-true byte walk instead of the csv
  reader. The `.ags.idx` format stays **locked at v1** — only the offset
  *values* tighten; the structure is unchanged, so existing certs still
  deserialize and reparse (their loose offsets remain usable via the
  leading-byte tolerance; a re-mint produces tight offsets). The oracle
  now asserts byte-identity to ground truth for **all** fixtures, and the
  two `csv_index_is_loose_for_*` snapshots were retired with the
  csv-based index they documented. First concrete csv-removal step in
  core (Phase 5 retires `ags4_codec`'s read path; Phase 7 drops the dep).

### O-41 [VARIANCE] Rows before the first GROUP are REPORTED as Rule 2 findings, not a hard parser crash
- **Observed**: a HEADING / UNIT / TYPE / DATA row that appears
  before any GROUP row is structurally invalid — it belongs to no
  group. The question is what a validator should DO with it.
- **python-ags4** (`AGS4.py`): its PARSER hard-fails. A pre-GROUP HEADING
  raises `AGS4Error('HEADER row in Line N is not associated with a
  GROUP …')`; a pre-GROUP DATA/UNIT/TYPE raises a `KeyError` on
  `headings[group]` (group is `None`). Because the parser raises,
  `check.py` never runs — the user gets a traceback and NO findings
  report for the file at all.
- **Us** (#189): the shared parse leaf is deliberately LENIENT — it
  drops the orphan row (so the rule engine still runs over the rest of
  the file). `rules/structure.rs::rule_2_orphan_rows` then REPORTS each
  orphan as an `AGS Format Rule 2` error finding, line-located, and the
  remaining groups validate normally. (The sibling case — a code-less
  `"GROUP"` row — is already a Rule 4 finding.)
- **Assessment**: reporting beats crashing. Laterite produces a COMPLETE
  findings report that *includes* the structural defect, where
  python-ags4 aborts on the first one and reports nothing. Same
  philosophy as O-32 (invalid UTF-8 → lossy decode + a Rule 1 finding,
  not a crash). Rule 2 is the attribution — the row belongs to no
  GROUP, which is Rule 2's domain (data is organised into GROUPs).
- **Upstream-reportable**: **[NOTE]** — python-ags4 could downgrade these parser
  hard-fails to `check.py` findings for a more useful report, but it's
  a design-philosophy difference, not a defect; not filing.
- **Our decision** (#168 Phase-5 follow-up): added `rule_2_orphan_rows`
  (walks `raw_lines` up to the first GROUP — one comparison for a
  well-formed file). python-ags4 parity is UNCHANGED at 122/9 (no
  parity-test file carries a pre-GROUP row). The core *reader* path
  (`ags4_codec`, opt-in strict — Phase 5) hard-fails on the same case
  instead, because a data reader is a different consumer than a
  validator.

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

## Post-V8 — laterite-originated checks (no python-ags4 equivalent)

### O-43 [VARIANCE] A self-declared but non-standard PA abbreviation is a laterite-originated FYI (Related to Rule 16); python-ags4 has no such check
- **Observed**: AGS Rule 16 requires every value in a `PA`-typed field to be
  defined in the file's ABBR group. A file can SATISFY that by self-declaring
  a code in ABBR that is not in the standard abbreviation picklist for its
  heading — a typo (`SAMP_TYPE="Borng"` for `"Boring"`) or an invented code
  (`SAMP_TYPE="ZZ"`). The file is spec-legal, but the non-standard code does
  not interoperate with tooling that keys off the standard picklist.
- **python-ags4** (`check.py`): `rule_16` checks PA values only against the file's
  OWN ABBR (matching our error-tier Rule 16). `fyi_16_1` flags only
  DESCRIPTION drift on an *otherwise-standard* code (matching our
  `rule_16_fyi`). The Warnings section is literally `# TO BE ADDED`. So
  python-ags4 has NO check that a self-declared abbreviation is itself
  non-standard — this is a laterite-originated signal, not a parity gap.
- **Us** (`validator/src/rules/groups.rs::rule_16_fyi_nonstandard_abbr`,
  gated under `include_fyi`): for each `(ABBR_HDNG, ABBR_CODE)` the file
  declares where the heading HAS a bundled standard picklist
  (`Dictionary::abbr_codes` non-empty) but the code is NOT in it
  (`abbr_desc` is `None`), emit one `FYI (Related to Rule 16)`. Bounded to
  standard-picklist headings — a genuinely custom / DICT-defined `PA` heading
  has no standard set to judge against and is skipped, so the FYI stays quiet
  on bespoke schemas. Complementary to `rule_16_fyi` (which fires on a
  standard code's description drift — the case this one skips).
- **Assessment**: a clean-room data-quality signal that catches typo'd / invented
  abbreviations the error-tier rules cannot (the file IS Rule-16-valid). It is
  informational (FYI) and opt-in (`include_fyi` / `--show-fyi`), and never
  changes the error verdict, so python-ags4 parity is untouched
  (`compat.check_file` does not set `include_fyi`). Deliberately an FYI, not a
  WARNING: the file breaks no rule, so over-stating it as a warning would be
  wrong (owner decision, #199). The WARNING tier therefore remains
  unpopulated — kept empty until a genuinely spec-ambiguous-but-suspicious
  condition warrants it.
- **Upstream-reportable**: **[NO]** — this is an additive laterite feature, not a defect in
  python-ags4. It could be suggested upstream as an enhancement (their
  Warnings section is unimplemented), but there is no divergence to report.
- **Our decision** (#199): SHIP as an FYI under the existing
  `FYI (Related to Rule 16)` bucket — no new finding key, so the compat
  severity classifier (which keys off the label substring) treats it as FYI
  with no change. Reuses the already-bundled standard picklist
  (`Dictionary::abbr_codes` / `abbr_desc`); no dictionary change. First member
  of a new "laterite-originated checks" family that python-ags4 lacks.

### O-44 [VARIANCE] Structural validation of a file-level DICT group is a laterite-originated WARNING (Related to Rule 18); python-ags4 only consumes DICT, never validates it
- **Observed**: AGS Rule 18 requires a DICT group when non-standard GROUP /
  HEADING names are used, but says nothing about the DICT's OWN
  well-formedness. A file can declare custom groups/headings through a
  MALFORMED DICT — a missing `DICT_TYPE` / `DICT_GRP` / `DICT_HDNG` column, a
  row with a blank `DICT_GRP`, or a `HEADING`-type row with a blank
  `DICT_HDNG`. The engine only *consumes* DICT (Rules 7/9 —
  `collect_file_dict` / `EffectiveDict::build` both bail or skip such rows
  silently), so a malformed DICT degrades every downstream check with ZERO
  feedback.
- **python-ags4** (`check.py`): `rule_18` does NO structural validation — like our
  error-tier `rule_18` (O-17) it only flags non-standard headings that have no
  DICT group at all. It never inspects the DICT group's own structure or
  completeness.
- **Us** (`validator/src/rules/groups.rs::rule_18_structure`, gated under
  `include_warnings`): flags the clearest defects as opt-in WARNINGs under a
  `Warning (Related to Rule 18)` label — a missing required column, a blank
  `DICT_GRP`, and a `HEADING`-type row with a blank `DICT_HDNG`. Branches on
  `DICT_TYPE` first so a GROUP-type row (legitimately blank `DICT_HDNG`) is not
  flagged. Softer `DICT_STAT` / `DICT_UNIT` / `DICT_PGRP` cells are deliberately
  deferred. The **first WARNING-tier producer** end-to-end (validator →
  PyO3/CLI/wasm → the dataframe `severity` column).
- **Assessment**: a clean-room structural check that catches a genuinely malformed
  dictionary the spec is silent on and python-ags4 ignores. WARNING, not
  Error: an error would break the 122/9 parity baseline (python-ags4 emits
  none); opt-in (`include_warnings` / `--show-warnings`) leaves the default
  verdict and the compat path untouched (`compat.check_file` does not set
  `include_warnings`). The separate `Warning (Related to Rule 18)` label keeps
  the compat severity classifier (label-substring keyed) from miscounting it as
  an error, and the error-tier Rule 18 bucket byte-stable. WARNING over FYI
  (cf. O-43): a malformed DICT is genuinely broken (it degrades downstream),
  not merely spec-legal-but-unusual.
- **Upstream-reportable**: **[NO]** — an additive laterite feature, not a python-ags4
  defect. Could be suggested upstream as an enhancement (their Warnings section
  is unimplemented), but there is no divergence to report.
- **Our decision** (#200): SHIP as the FIRST WARNING-tier producer — the
  shipped-but-inert `--show-warnings` / `warnings=True` flag now has a
  defensible producer. The unrecognised-`TRAN_AGS` FYI is a candidate SECOND
  warning (a laterite-stricter categorisation divergence; tracked, owner-gated).

## How to add an entry

Append under the current phase heading. Use the next `O-N`. Keep the
five fields (observed / spec / assessment / upstream-reportable /
our decision). When an item is reported upstream, add a
`- **Reported**: <url or ref> (<date>)` line so we don't double-file.
