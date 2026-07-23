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
> upstream-reportable O-Ns ([[observations-coverage-map]]) plus the
> probe-confirmed Rule 19 finding.

## Tier 1 — concrete rule defects / mis-specifications
1. **Rule 19 prose vs reality** ([[O-06]], [[rule19-spec-allows-numbers-validator-may-not]] — *probe-confirmed*): 4.2 says "≤4, uppercase letters **and numbers**"; every standard group (0/319) and both validators enforce "exactly 4 uppercase letters". A spec-legal user-defined `AB12` is validator-rejected. **Propose: state "exactly 4 uppercase letters".**
2. **Rule 19b field-length** ([[O-07]]): the ≤4-char field-part limit is enforced by all but stated nowhere. **Propose: state it.**
3. **Rule 6 under-spec / python no-op** ([[O-02]]): a bare embedded CR in a quoted field violates Rule 6 but python's `rule_6` is a literal no-op. **Propose: Rule 6 must independently scan embedded CR/LF.**
4. **rule_7_2 crash** ([[O-08]]): duplicate HEADING can `IndexError` the whole python run. **Propose (python-ags4): bound-check.**

## Tier 2 — attribution / inference ambiguities
5. Missing HEADING filed under Rule 4 not 2b ([[O-04]]); ID-uniqueness folded into Rule 8 not 10a ([[O-11]]); non-standard GROUP-name not keyed off ([[O-17]]); no-duplicate-HEADING inferred ([[O-09]]); Rule 10c parentless hardcoded not data-driven ([[O-21]]).

## Tier 3 — encoding & ranges (our VARIANCEs worth raising)
6. Rule 1 "entirely ASCII" vs ubiquitous cp1252 ([[O-01]], [[O-32]]) — spec should mandate UTF-8 + a non-ASCII policy.
7. python's pandas Timestamp range silently rejects spec-valid pre-1678/post-2262 dates ([[O-33]]).

## Related
[[observations-coverage-map]] · [[upstream-reporting]] · ags4-vs-ags5
