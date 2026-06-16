# `laterite.compat` vs `python-ags4` — differences

This document catalogues every behavioural difference between
`laterite.compat` (the drop-in shim) and `python-ags4 1.2.0` (the
upstream library it tracks). It exists so you can:

- Decide whether `laterite.compat` is a safe swap-in for your code,
- Understand any divergent finding you see in a report,
- Know which differences are deliberate, which are bugs (in either
  direction), and which are spec ambiguities the AGS-DFWG could
  clarify.

**Status**: `122/131` of python-ags4's own test suite passes
through `laterite.compat` (93%). The 9 residual failures are all in
the deliberate-non-closure category — listed below with rationale.

The authoritative behavioural catalogue is
[`OBSERVATIONS.md`](OBSERVATIONS.md). This page summarises the
user-facing parts.

---

## Table of contents

1. [Quick mental model](#quick-mental-model)
2. [Identity & versioning](#identity--versioning)
3. [Configuration knobs](#configuration-knobs)
4. [Rule-by-rule behavioural differences](#rule-by-rule-behavioural-differences)
5. [Parser strictness](#parser-strictness)
6. [Error handling philosophy](#error-handling-philosophy)
7. [API surface](#api-surface)
8. [Encoding handling](#encoding-handling)
9. [Upstream-reportable items](#upstream-reportable-items)
10. [The full O-N catalogue](#the-full-o-n-catalogue)

---

## Quick mental model

The two libraries agree on the **vast majority** of AGS4 validation
behaviour. Where they differ, the differences fall into a few
predictable buckets:

- **More correct than python-ags4** — we fix python-ags4 bugs
  (Rule 8 DT non-ISO UNITs, BOM detection wording, etc.) and
  produce more accurate findings.
- **More strict than python-ags4** — we refuse genuinely-not-AGS4
  input (non-CSV, AGS3) rather than mis-validating it.
- **More lenient than python-ags4** — the native parser tolerates
  malformed structure (duplicate groups, ragged rows) so the
  validator can *report* the problem rather than crash. `compat`
  layers strictness back on to mirror python-ags4 for
  drop-in users.
- **Identical findings, different wording** — `_compat_desc.py`
  translates laterite's more-precise wording into python-ags4's
  phrasing on the compat surface (opt-out via
  `match_python_ags4_wording=False`).
- **Identity claims** — `__version__` and `Metadata.Checker`
  honestly identify as laterite, not python-ags4. We don't lie about
  provenance.

---

## Identity & versioning

`laterite.compat.__version__` uses [PEP 440 local-version
syntax](https://peps.python.org/pep-0440/#local-version-identifiers):

```python
>>> import laterite.compat
>>> laterite.compat.__version__
'0.1.0+compat.python-ags4.1.2.0'
```

- `0.1.0` — the laterite release.
- `+compat.python-ags4.1.2.0` — the python-ags4 version we hold
  parity against. The pin is exact (python-ags4 minors can change
  behaviour silently — see the dev-dep comment).

The compat-pin segment doesn't have to track laterite's own version;
laterite can release `0.1.1`, `0.2.0`, etc. while still tracking
`python-ags4 1.2.0`. When we update the python-ags4 pin
(deliberately — never silent), the local-version segment changes too,
e.g. `0.2.0+compat.python-ags4.1.3.0`.

`Metadata.Checker` in validation reports reads:

```
laterite 0.1.0 — compat: python-ags4 1.2.0 — clean-room laterite_ags4_validator engine
```

This is the source of 5 parity-test failures (which expect us to
identify as `python_ags4 v1.2.0`). We deliberately don't lie about
identity: downstream tools reading the JSON report can tell which
validator produced it.

### Reading the parity-pin programmatically

The python-ags4 version laterite holds parity against is also
exposed as a constant:

```python
>>> from laterite.compat import PYTHON_AGS4_COMPAT
>>> PYTHON_AGS4_COMPAT
'1.2.0'
```

Downstream tools that need the parity-pin should read this constant
rather than parsing the local-version segment of `__version__`. A
paired test asserts the two stay in sync.

### Versioning migration path

The PEP 440 local-version pin is **transient by design**. Its job is
to communicate "this release was parity-tested against python-ags4
X.Y.Z" — useful while "drop-in for python-ags4" is laterite's main
value proposition. Once laterite stands on its own, the pin gets
dropped.

The migration is monotone-cheap (each phase strictly removes
coupling; you never need to add it back):

**Phase 1 — current (laterite 0.x)**. Both the local-version pin and
the constant exist:

```python
__version__ = "0.1.0+compat.python-ags4.1.2.0"
PYTHON_AGS4_COMPAT = "1.2.0"
```

The compat surface is the headline feature, so `__version__` carries
the parity claim.

**Phase 2 — maturing (still 0.x, post compat-stabilisation)**. The
pin drops out of `__version__` while the constant stays:

```python
__version__ = "0.x.y"            # just laterite's version
PYTHON_AGS4_COMPAT = "1.2.0"     # parity claim
```

Downstream tools that *care* about parity read `PYTHON_AGS4_COMPAT`;
tools that don't care just see clean semver.

**Phase 3 — dominant (laterite 1.0+)**. Pin is gone entirely. Laterite
has its own user base, the compat shim is a backwards-compat layer
rather than the main draw. `PYTHON_AGS4_COMPAT` may either stay
(harmless, documents the historical parity) or be retired alongside
the compat shim.

**Signals for each transition**:

- **1 → 2**: when downstream consumers stop asking "which python-ags4
  are you parity-tested against?" — typically 6-12 months of stable
  usage.
- **2 → 3**: when laterite has its own user base who landed via the
  native API, not via porting from python-ags4.
- **Optional 3 → compat-as-separate-package**: only if the compat
  shim is actively rotting (python-ags4 evolves in ways we don't
  track, LGPL-vs-MIT separation becomes load-bearing, etc.).

**Upgrade workflow** when re-targeting python-ags4 (e.g. when 1.3.0
releases):

1. Update `python-ags4==1.3.0` in `[dependency-groups] dev` in the
   repo's `pyproject.toml`.
2. Run `./tools/run_python_ags4_tests.sh` — see what regressed and
   what behaviours changed.
3. Decide per change: align laterite (update O-N) or catalogue as
   new deliberate divergence.
4. Bump `PYTHON_AGS4_COMPAT = "1.3.0"`. The `__version__` string
   auto-updates via f-string.
5. Update the compatibility matrix below.

### Compatibility matrix

| laterite | tracks python-ags4 | notes |
|---|---|---|
| 0.1.x | 1.2.0 | initial release |

This table grows by one row each time we re-target python-ags4 or
ship a notable laterite-internal release.

---

## Configuration knobs

`laterite.compat` exposes several toggles that affect its python-ags4
fidelity:

| Knob | Default | Effect |
|---|---|---|
| `check_file(..., match_python_ags4_wording=False)` | `True` | Disable the desc translator. Findings carry laterite's native (more precise) wording. |
| `set_backend("polars")` / `LATERITE_COMPAT_BACKEND=polars` | `"pandas"` | Return polars / pyarrow frames instead of pandas. Useful when pandas isn't installed. |
| `check_file(..., encoding="cp1252")` | `"utf-8"` | Decode the source file as cp1252 / latin1 / iso-8859-1 etc. (powered by `encoding_rs` in Rust). |

The validator always runs with `include_fyi=True` when invoked
through `compat.check_file` — python-ags4 emits FYI keys (`FYI`,
`FYI (Related to Rule 1)`, `FYI (Related to Rule 16)`) which the
parity tests assert on. The native `laterite.validate(...)` /
`lat-check` CLI default `include_fyi=False`.

---

## Rule-by-rule behavioural differences

For each AGS4 rule where laterite and python-ags4 disagree, this
section describes:

- What the spec says,
- What python-ags4 does,
- What laterite does and why,
- Which way is **more correct** by the spec.

If a rule isn't listed here, the two validators agree.

### Rule 1 — non-ASCII characters

- **Spec** (§4.1.1 Rule 1): "The data file shall be entirely
  composed of ASCII characters." Strict ASCII is 0–127.
- **python-ags4**: relaxes to 0–255 (allows Latin-1 extended ASCII
  like `°`, `±`, `µ`). 128–255 are demoted to an FYI ("Has
  extended ASCII character(s)") that's emitted only when
  `include_fyi=True`. Above 255 → Rule 1 error.
- **laterite**: matches python-ags4's interpretation. 0–127 clean;
  128–255 FYI (gated by `include_fyi`); >255 Rule 1 error.
- **BOM (UTF-8 BOM, `EF BB BF`)**: both validators detect it.
  laterite emits a BOM-specific Rule 1 message *plus* an FYI
  recommending the file be saved without BOM — wording matches
  python-ags4's `test_file_with_BOM`. See **O-1** for the relaxed
  ASCII rationale; the spec language is the upstream-reportable
  ambiguity.

### Rule 5 — quoting

- **Spec** (§4.1.1 Rule 5): "Each data VARIABLE shall be enclosed
  in double quotes. Any double quote within a data VARIABLE shall
  be doubled."
- **python-ags4**: distinguishes two sub-violations:
  - "Contains quotes within a data field. All such quotes should
    be enclosed by a second quote." (embedded `"` not doubled)
  - "Contains fields that are not enclosed in double quotes."
    (unterminated field / missing `"`)
- **laterite** (Stage 7g): walks the line via `check_quoting` and
  classifies the deviation type — `EmbeddedQuote` vs `NotEnclosed`
  — emitting python-ags4's matching wording for each. Both
  validators now agree on which case fired and what to say about it.

### Rule 6 — embedded carriage returns

- **Spec**: no CR or LF between or within data variables.
- **python-ags4**: `rule_6` is a no-op (`return ags_errors` with a
  comment that 2a/4b/5 will catch it).
- **laterite**: actually implements Rule 6 — detects an embedded
  `\r` in line content and flags it on the line it appears.
- **Status**: see **O-2 [BUG]** — upstream-reportable. python-ags4
  *also* mis-classifies a lone embedded CR as Rule 2a + Rule 3 +
  Rule 5 (cascade via Python's universal-newline reader). laterite
  emits exactly Rule 6 on the same input — a cleaner diagnostic.
- **Test impact**: laterite refuses extreme cases (non-CSV files)
  outright as `NotAgs4Error`; python-ags4 walks them and emits
  Rule 3/5/19a "errors". One parity test stays red here
  (`test_rule_6_1`); we believe our behaviour is more correct.

### Rule 8 — typed values

Largest divergence area. Multiple sub-cases:

**nSF (significant figures)**:
- python-ags4 emits `Value <v> in <head> not of data type <type>.
  (Expected: <rounded>)` — appends the SF-rounded reference value.
- laterite (Stage 7b): emits the same suffix verbatim via
  `format_nsf` in Rust. Native API + compat both benefit.

**DT (date/time)**:
- python-ags4 has two checks: a regex from the UNIT pattern, plus
  `pd.to_datetime(value, format='ISO8601')` for any non-time UNIT.
  The ISO8601 hard-code means **non-ISO UNITs cannot be validated**
  — values like `01/12/2020` under UNIT `dd/mm/yyyy` are
  structurally fine but mask1-flagged.
- laterite (Stage 8): walks the UNIT pattern token-by-token
  (`yyyy`/`yy`/`mm`/`dd`/`hh`/`ss` with context-sensitive `mm` =
  month-or-minute), extracts calendar fields from the value,
  validates ranges + pandas Timestamp bound. Spec-correct for
  European (`dd/mm/yyyy`), US (`mm/dd/yyyy`), 2-digit year
  (`dd/mm/yy`), month-precision (`yyyy-mm`, `mm-yyyy`).
- **Status**: see **O-38 [SPEC]** — upstream-reportable. The DT-
  format probe matrix (run against both validators) showed 34/45
  cases agree; the 11 python-ags4-only divergences are all
  "python-ags4 wrongly flags a valid value".

**Year-only / month-precision (`yyyy`, `yyyy-mm`, `mm-yyyy`)**:
- python-ags4 accepts via ISO8601 lenient parsing.
- laterite (Stage 7c, refined Stage 8): explicit handlers for each
  shape, validates month range (1–12), day defaults to 1.

**DT/yyyy-mm bug (pre-Stage 7c, now fixed)**:
- Earlier laterite versions wrongly rejected valid `2023-11` under
  UNIT `yyyy-mm`. Resolved Stage 7c.

**DT pandas-Timestamp range bound**:
- See **O-33 [VARIANCE]**: laterite bounds DT values to pandas'
  Timestamp window (1677-09-21 to 2262-04-11) — matches what
  python-ags4 does indirectly via `pd.to_datetime`. A corrupt
  year like `0018-06-03` flags Rule 8 in both validators.

**Empty DT UNIT (O-31)**:
- A DT field with no UNIT used to be silently accepted by laterite;
  python-ags4 flags it. Now aligned — see **O-31**.

**ID uniqueness folded into Rule 8 (O-11)**:
- python-ags4 emits ID-uniqueness violations under Rule 8.
  Spec-wise this is Rule 10a's territory. laterite folds it the
  same way (mirroring python-ags4's bucket) but the spec is
  imprecise — see **O-11 [SPEC]**.

### Rule 10c — parent-child links

- **Spec** (`spec:AGS4-4.2-2025.pdf §4.1.1 Rule 10c`):
  *"Every entry made in the KEY fields in any GROUP must have an
  equivalent entry in its PARENT GROUP."*
- **python-ags4**: reads "entry made" permissively — empty KEY
  cells are not entries, so a row whose parent-KEY cells are all
  empty is "standalone" and skipped.
- **laterite** (pre-Stage 9b): read strictly — every row's empty
  KEY counts as an entry that points to nothing → Rule 10c flag.
- **laterite (Stage 9b, current)**: aligned with python-ags4. A
  child row whose parent-KEY tuple is *all* empty is skipped.
- **Status**: see **O-39 [SPEC]** — upstream-reportable. The spec
  should explicitly say whether empty KEY cells count as entries.
  Real geotech workflows produce standalone rows (lab controls,
  off-site samples) — the permissive reading is the better UX.

### Rule 16 — abbreviations / standard ABBR mismatch FYI

- **python-ags4**: `fyi_16_1` compares each ABBR row's `ABBR_DESC`
  against the bundled standard ABBR list (case-insensitive). Emits
  `FYI (Related to Rule 16)` per mismatch.
- **laterite** (Stage 7e): same behaviour. We codegen the standard
  ABBR data into the Rust binary (`build.rs` emits a
  `DICT_<edition>_ABBRS` phf map per edition), then `rule_16_fyi`
  in `rules/groups.rs` performs the same comparison. Wording
  matches python-ags4 verbatim. Native + compat both benefit.

### Rule 19b — heading prefix validation

- **python-ags4**: for a malformed heading like `XXXX_425` in group
  LLPL, emits up to three Rule 19b findings:
  1. `rule_19b_1`: "Heading X is more than 9 characters" (when
     applicable) — a length check.
  2. `rule_19b_2`: "Group X referred to in Y could not be found in
     either the standard dictionary or the DICT group." — when the
     prefix doesn't name a defined group.
  3. `rule_19b_3`: "Y does not start with the name of this group,
     nor is it defined in another group." — when the heading
     isn't defined anywhere.
- **laterite** (Stage 9c — half-revert of O-26): emits the
  `rule_19b_2` message; *also* emits the `rule_19b_3` message
  when the heading isn't defined anywhere. We don't emit the
  rule_19b_1 length-check redundancy.
- **Status**: see **O-26 [NOTE]** — partial deliberate divergence.
  laterite emits 2 findings vs python-ags4's 3 on a malformed
  heading. The two we emit target different fixes (prefix typo vs
  placement mistake); the third is mostly redundant.

### Rule 20 — FILE attachments

- **python-ags4**: always runs both the data-level check (every
  `FILE_FSET` used must be defined in the FILE group) *and* the
  on-disk check (sidecar `FILE/<fset>/<name>` must exist).
- **laterite**: data-level check is always on. On-disk check is
  **opt-in** via `lat-check --check-files` (or
  `CheckOptions::check_files=true`). Compat passes
  `check_files=True` to match python-ags4's always-on behaviour
  for parity.
- **Status**: see **O-27 [NOTE]** — design choice (the data-level
  check is path-independent and deterministic; the on-disk check
  is a packaging/QA concern).

---

## Parser strictness

The native laterite parser is deliberately **lenient** —
malformed structure (duplicate GROUP declarations, ragged DATA
rows, duplicate headings) is *reported* by the validator, not
*raised* by the parser. python-ags4 raises hard.

**laterite.compat** layers strictness back on via
`_strict_pre_check` (in `compat.py`) which scans the file with
`csv.reader` and raises `Ags4Error` for:

- Duplicate GROUP declarations
- DATA rows with field count ≠ HEADING row
- (When `rename_duplicate_headers=False`) duplicate headings

This is exclusively a compat surface concern — native callers see
lenient parsing. See **O-37 [VARIANCE]**.

---

## Error handling philosophy

This is the most significant **Python-idiom** difference.

**python-ags4** catches its own exceptions inside `check_file` and
returns them as dict entries under a `"Validator Process Error"`
key:

```python
el = AGS4.check_file('bad-input.ags', dictionary='not-a-dict.txt')
# Doesn't raise. Returns:
# {"Validator Process Error": [{...}], "Metadata": {...}}
```

**laterite.compat** raises specific exception classes (the
Pythonic / EAFP pattern):

```python
try:
    el = AGS4.check_file('bad-input.ags', dictionary='not-a-dict.txt')
except BadDictError as e:
    print(f"can't validate: {e}")
```

Why we kept the Pythonic pattern:

- **Type safety**: callers catch by class. python-ags4's pattern
  uses magic-string membership (`'Validator Process Error' in el`)
  which IDEs / type checkers can't validate.
- **Easy-to-forget bug**: a caller iterating `el.items()` for
  findings will silently treat the error entry as just-another-
  finding.
- **Asymmetric in python-ags4 itself**: `AGS4_to_dict` raises;
  `check_file` catches-and-wraps. The convention isn't even
  internally consistent in python-ags4.

If you need byte-faithful python-ags4 behaviour at this level, the
trade-off is on the table (one config flag away). Two parity tests
stay red because of this: `test_checking_without_dictionary_raises_error`
and `test_duplicate_groups_raises_error` — both surface a typed
exception we believe carries strictly more information than the
upstream wrapped report.

---

## API surface

`laterite.compat` mirrors python-ags4's public API. Functions that
exist verbatim:

| python-ags4 | laterite.compat | notes |
|---|---|---|
| `AGS4.AGS4_to_dict` | `compat.AGS4_to_dict` | python-ags4-shaped output |
| `AGS4.AGS4_to_dataframe` | `compat.AGS4_to_dataframe` | backend-configurable (default pandas) |
| `AGS4.AGS4_to_dataframe_AGS3` | `compat.AGS4_to_dataframe_AGS3` | raises `UnsupportedEditionError` per O-30 |
| `AGS4.AGS4_to_excel` | `compat.AGS4_to_excel` | Rust-backed (calamine + rust_xlsxwriter) |
| `AGS4.excel_to_AGS4` | `compat.excel_to_AGS4` | Rust-backed; `dictionary=` works post-Stage 6d |
| `AGS4.dataframe_to_AGS4` | `compat.dataframe_to_AGS4` | byte-faithful spec emit |
| `AGS4.convert_to_numeric` | `compat.convert_to_numeric` | strips UNIT/TYPE, casts numeric |
| `AGS4.convert_to_text` | `compat.convert_to_text` | `dictionary=<edition>` works post-Stage 6d |
| `AGS4.check_file` | `compat.check_file` | python-ags4-shaped dict; new `match_python_ags4_wording` kwarg |
| `AGS4.format_numeric_column` | `compat.format_numeric_column` | matches python-ags4's per-column TYPE formatter |
| `AGS4.count_errors` | `compat.count_errors` | categorises by error/warning/FYI prefix |
| `AGS4.sort_groups` | `compat.sort_groups` | hierarchical / dictionary / alphabetical |
| `AGS4.write_error_report` | `compat.write_error_report` | byte-exact text report |
| `AGS4.AGS4Error` | `compat.AGS4Error` | aliased to native `Ags4Error` |
| `check.get_TRAN_AGS` | `compat.get_TRAN_AGS` | reads TRAN_AGS from a tables dict |
| `utils.get_DICT_table_from_json_file` | `compat.get_DICT_table_from_json_file` | parses AGS-DFWG JSON to AGS4 DICT shape |
| `utils.get_ABBR_table_from_json_file` | `compat.get_ABBR_table_from_json_file` | parses to ABBR shape |
| `utils.get_TYPE_table_from_json_file` | `compat.get_TYPE_table_from_json_file` | parses to TYPE shape |
| `utils.get_UNIT_table_from_json_file` | `compat.get_UNIT_table_from_json_file` | parses to UNIT shape |

**Extensions beyond python-ags4**:

- `compat.set_backend(name)` / `compat.get_backend()` — switch
  output frames to polars / pyarrow / pandas.
- `check_file(..., match_python_ags4_wording=False)` — bypass the
  desc translator.
- `check_file(..., encoding="cp1252")` — non-UTF-8 file decoding.

**Not mirrored**: python-ags4's CLI (`ags4_cli`). laterite ships
`lat-check` (Rust binary, byte-faithful JSON / NDJSON output) as
a different CLI surface. See the README for the two-error-JSON-shape
table.

---

## Encoding handling

python-ags4 accepts `encoding=` on `check_file` and decodes the file
accordingly.

laterite (Stage 7b) threads encoding through the **Rust shared lib**:

- `lat-check --encoding cp1252 file.ags` — CLI flag.
- `laterite.validate(path, opts=CheckOptions{encoding=...})` — native.
- `laterite.compat.check_file(path, encoding="cp1252")` — compat.

All three surfaces share the same `encoding_rs` decoder. Accepted
labels: `utf-8` / `cp1252` / `windows-1252` / `latin1` / `iso-8859-1` /
`iso-8859-15` / `latin9` plus any [WHATWG encoding
label](https://encoding.spec.whatwg.org/#names-and-labels).

BOM detection works for UTF-8 BOM (`EF BB BF`). Other encodings'
BOMs (UTF-16 LE/BE) trigger encoding_rs's transparent BOM-strip and
*don't* surface a Rule 1 finding — UTF-16 AGS4 is out of spec
anyway.

---

## Upstream-reportable items

The following observations are candidates for AGS-DFWG / python-ags4
issues. None have been reported yet.

| O-N | Tag | Subject |
|---|---|---|
| O-1 | VARIANCE | Spec text says "ASCII" (0–127); de facto practice is 0–255 |
| O-2 | BUG | python-ags4 Rule 6 is a no-op (`return ags_errors` body) |
| O-4 | SPEC | "HEADING row missing" attribution: Rule 4 vs Rule 2b |
| O-6 | SPEC | Rule 19 prose looser than the AGS dictionary |
| O-7 | SPEC | Rule 19b_1 field-length limit not in prose |
| O-8 | BUG | python-ags4 `rule_7_2` can raise IndexError on duplicate headings |
| O-9 | NOTE | Rule 7 "no duplicates" inferred, not in prose |
| O-11 | SPEC | python-ags4 folds ID-uniqueness into Rule 8 (Rule 10a's job) |
| O-17 | SPEC | Rule 18 keys off heading membership only |
| O-21 | SPEC | Rule 10c parentless-group list is hardcoded |
| O-30 | VARIANCE | TRAN_AGS-driven edition selection deliberate divergences |
| O-31 | VARIANCE | Rule 8 empty `DT` UNIT now flagged |
| O-32 | VARIANCE | Non-UTF-8 input decoded lossily, not refused |
| O-33 | VARIANCE | Rule 8 DT/datetime bounded to pandas Timestamp range |
| O-34 | VARIANCE | `NotAgs4` ↔ python "missing mandatory groups" KNOWN_DIVERGENCE |
| **O-38** | **SPEC** | **Rule 8 DT: python-ags4 forbids non-ISO UNITs** |
| **O-39** | **SPEC** | **Rule 10c: empty parent KEYs are "no entry" or "missing link"?** |

The two boldface entries are the most actionable for python-ags4:
both are real defects that affect real-world delivery files (European
date formats, standalone samples), and both have small, well-scoped
fixes.

---

## The full O-N catalogue

39 entries total. The full text (5-field house style: observed /
spec / assessment / upstream-reportable / our decision) lives at
[`OBSERVATIONS.md`](OBSERVATIONS.md).

A summary table by tag:

| Tag | Count | Meaning |
|---|---|---|
| VARIANCE | 12 | Intentional impl-vs-spec divergence (not a defect) |
| SPEC | 8 | Spec ambiguity / contradiction the AGS-DFWG could clarify |
| BUG | 2 | Likely python-ags4 defect |
| NOTE | 17 | Behavioural observation (internal documentation) |

### The 9 residual parity-test failures

| Test | Category | Why |
|---|---|---|
| `test_version` | Identity | We don't claim to be python-ags4 v1.2.0 |
| `test_rule_2` | Identity | Metadata.Checker says laterite, not python-ags4 |
| `test_rule_2b_1` | Identity | Same |
| `test_rule_LBSGCheck` | Identity | Same |
| `test_rule_STNDandPREMCheck` | Identity | Same |
| `test_rule_AGS3` | O-30 | We refuse AGS3 rather than mis-validate as AGS4 |
| `test_rule_6_1` | O-2 / O-34 | We refuse non-CSV input as `NotAgs4Error` |
| `test_checking_without_dictionary_raises_error` | H-1 | We raise, python-ags4 wraps |
| `test_duplicate_groups_raises_error` (check.py) | H-1 | Same |

Each is defensible; closing them would unwind a deliberate design
decision. The pass rate (122/131, 93%) is the honest signal of
where the two validators actually agree.

---

## See also

- [`README.md`](README.md) — install + quick start
- [`OBSERVATIONS.md`](OBSERVATIONS.md) — full O-N catalogue
- [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md) — test-level coverage map of laterite ↔ python-ags4
- [python-ags4 upstream](https://gitlab.com/ags-data-format-wg/ags-python-library) — the library we track
