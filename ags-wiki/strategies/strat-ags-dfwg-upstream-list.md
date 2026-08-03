---
type: strategy
title: "AGS 4.2 improvement / AGS-DFWG upstream proposal list"
status: confirmed
tags: [strategy, register]
targets: [rule-19-group-name-format, rule-06-comma-no-embedded-crlf, rule-07-heading-order, rule-01-ascii]
divergence_hypothesis: "n/a — consolidated upstream register"
probe_files: []
expected_rust: ""
expected_python: ""
evidence: "synthesis of the upstream-reportable O-Ns + the probe-confirmed Rule 19 finding"
feeds_ags5_req: []
related: [observations-coverage-map, rule19-spec-allows-numbers-validator-may-not, upstream-reporting]
sources: []
---
# AGS 4.2 improvement / AGS-DFWG upstream proposal list

> [!note] The actionable output of the campaign for the *spec*: what
> to propose to the AGS Data Format Working Group. Drawn from the
> upstream-reportable O-Ns ([[observations-coverage-map]], whose set is
> generated from `observations.json`) plus the probe-confirmed Rule 19
> finding. The tiering and the wording of each proposal are ours; the
> membership is the catalogue's `upstream` flag.

## Tier 1 — concrete rule defects / mis-specifications
1. **Rule 19 prose vs reality** ([[O-06]], [[rule19-spec-allows-numbers-validator-may-not]] — *probe-confirmed*): 4.2 says "≤4, uppercase letters **and numbers**"; every standard group (0/319) and both validators enforce "exactly 4 uppercase letters". A spec-legal user-defined `AB12` is validator-rejected. **Propose: state "exactly 4 uppercase letters".**
2. **Rule 19b field-length** ([[O-07]]): the ≤4-char field-part limit is enforced by all but stated nowhere. **Propose: state it.**
3. **Rule 6 under-spec / python no-op** ([[O-02]]): a bare embedded CR in a quoted field violates Rule 6 but python's `rule_6` is a literal no-op. **Propose: Rule 6 must independently scan embedded CR/LF.**
4. **rule_7_2 crash** ([[O-08]]): duplicate HEADING can `IndexError` the whole python run. **Propose (python-ags4): bound-check.**
5. **Uncapped numeric-TYPE count → OOM** ([[O-49]]): the *n* in `nDP`/`nSF`/`nSCI` is read from the file and fed straight into a format width, so a crafted `9999999999SF` makes python-ags4 build a ~10 GB string. Any caller that renders a value to its expected form — Rule 8 fixes, XLSX export — is DoS-able by a malformed or hostile file. **Propose (python-ags4): clamp the count** (laterite clamps at 30). The strongest Tier 1 item: a defect, not an ambiguity.
6. **Rule 8 DT UNITs hard-coded to ISO8601** ([[O-38]]): python-ags4's `rule_8` DT branch passes `ISO8601` rather than translating the heading's declared UNIT pattern into a `format=` string, so spec-legal non-ISO date formats are flagged. **Propose (python-ags4): derive `format=` from the UNIT.** High priority — it fires on real European/US delivery files.

## Tier 2 — attribution / inference ambiguities
7. Missing HEADING filed under Rule 4 not 2b ([[O-04]]); ID-uniqueness folded into Rule 8 not 10a ([[O-11]]); non-standard GROUP-name not keyed off ([[O-17]]); no-duplicate-HEADING inferred ([[O-09]]); Rule 10c parentless hardcoded not data-driven ([[O-21]]).
8. **Rule 10c and empty KEY cells** ([[O-39]]): the spec never says whether an empty parent KEY *participates* in the link requirement. We read it as "no entry" rather than a missing link; the other reading is equally defensible, which is the problem. **Propose: say which.**

## Tier 3 — edition resolution (spec-silent, so both readings are legal)
9. **Bare `TRAN_AGS = "4.0"`** ([[O-30]], [[O-42]]): the spec doesn't say which 4.0 *patch* a bare `"4.0"` means. python-ags4 maps it statically to 4.0.3 — an alias never bumped when 4.0.4 shipped — which over-reports Rule 10c through the PMTL→PMTD hierarchy and mis-flags the eight 4.0.4-only headings as non-standard. laterite resolves to 4.0.4 (superset-safe) behind a content guard. **Propose: state that an unpatched edition means the newest patch** — and, to python-ags4, that the static alias is stale.

## Tier 4 — encoding, formats & ranges (our VARIANCEs worth raising)
10. Rule 1 "entirely ASCII" vs ubiquitous cp1252 ([[O-01]], [[O-32]]) — spec should mandate UTF-8 + a non-ASCII policy.
11. An empty UNIT on a `DT` heading ([[O-31]]) declares no format at all, so a value can't be checked against one. Flagging it is defensible, but python's `format ()` message text is opaque; the underlying condition is a producer-side data defect the spec could name.
12. python's pandas Timestamp range silently rejects spec-valid pre-1678/post-2262 dates ([[O-33]]).

## Related
[[observations-coverage-map]] · [[upstream-reporting]] · ags4-vs-ags5
