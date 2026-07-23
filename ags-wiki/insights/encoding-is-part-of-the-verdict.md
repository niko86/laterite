---
type: insight
title: "Rule 1's severity is a property of the DECODER, not the file — so a cached verdict must record which decoder produced it"
status: ratified
tags: [insight]
gap_kind: behaviour
severity: high
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-01-ascii]
proposes_observation: true
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: D
related: [rule-01-ascii, O-01, O-32, O-48, cert-trust-v2, dec-ags-idx-certificate]
sources: [spec-4.2]
---
# The decoder is part of the verdict

## Claim
> [!confirmed] One unchanged file, two `--encoding` labels, two different **error**
> verdicts — on **both** validators. Rule 1 sorts characters by code point (128–255
> tolerated as FYI; above that, an error), and which code points a byte sequence
> *becomes* is the decoder's answer, not the file's. So a validator's verdict is a
> function of `(bytes, decoder)`, and anything that **caches** a verdict — an
> `.ags.idx` certificate — must record the decoder or it is caching an incomplete
> statement.

## Evidence

Probed 2026-07-14 through **both** validators, same bytes, same file on disk:

| decoder | laterite | python-ags4 1.2.0 |
|---|---|---|
| `utf-8` | `AGS Format Rule 1` × 1 (**error**) | `AGS Format Rule 1` × 1 (**error**) |
| `windows-1252` | `FYI (Related to Rule 1)` × 1 | `FYI (Related to Rule 1)` × 1 |

The file is a clean minimal AGS4 delivery whose `PROJ_NAME` carries a Greek capital
omega — UTF-8 bytes `CE A9`. Read as UTF-8 that is **one** code point (937), above the
extended-ASCII range Rule 1 tolerates → an error. Read as windows-1252 the very same two
bytes are **two** code points (206, 169), both inside it → an FYI. Nothing about the file
changed; only the label did.

The agreement is not a coincidence. python-ags4's `check.py::rule_1` sorts on
`is_ags_ascii(line)` and words its own message *"assuming that file encoding is
'{encoding}'"* — it says out loud that its verdict is decoder-relative. Ours reaches the
same two answers by the same route ([[O-01]], [[O-32]]).

**It was exploitable.** Before the gate landed, on the shipped build:

```
plain validate (utf-8):   1 finding(s), is_valid = False
certified under cp1252:   omega.ags.idx      <- mints: no ERROR under that decoder
validate --index (utf-8): count = 0 | certified = True | is_valid = True
```

Certify under the lenient decoder, read back under the strict one, and a file with a
Rule 1 error reported clean — the certificate vouching for a verdict reached under a
decoder nobody had asked for.

## Why it matters

This is the case that shows **CONTENT is not the same as SEALED**. The
[[cert-trust-v2]] trust model partitions every input into CONTENT (a pure function of
the certified bytes — a certificate may speak for it) and WORLD (state the bytes do not
contain — no certificate may ever speak for it). `encoding` is *correctly* CONTENT: the
text is a pure function of the bytes and the label. But the certificate sealed only the
**bytes**, and the label is the other half of that function — so the fast path was taken
on a question the certificate had never been asked.

The rule the model needed, and now enforces: **every input the findings depend on must be
in the certificate.** `ValidationStamp.encoding` records the decoder; `Sidecar::decide`
refuses a request made through another (`RevalidateReason::EncodingDiffers`). The decoder
a cert *was* minted under still gets the fast path — a match, not a ban.

## Related
[[rule-01-ascii]] · [[O-01]] · [[O-32]] · [[O-48]] · [[cert-trust-v2]] · [[dec-ags-idx-certificate]]
