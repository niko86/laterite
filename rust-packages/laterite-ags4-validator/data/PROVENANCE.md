# Standard dictionary provenance & licence note

## What these files are

`Standard_dictionary_v4_0_3.ags`, `Standard_dictionary_v4_0_4.ags`,
`Standard_dictionary_v4_1.ags`, `Standard_dictionary_v4_1_1.ags`, and
`Standard_dictionary_v4_2.ags` are the machine-readable AGS4 **Data
Dictionary** for each published edition — the canonical list of group
codes, heading names, AGS data types, units, status, and descriptions
that the AGS4 transfer-format standard defines. They are the **origin**
of the dictionary the validator checks files against (Rules 7, 9, 16,
17, 19).

These files are read by **one tool only** — `tools/gen_dictionary.py`,
which projects all five editions into the single consolidated union
`rust-packages/laterite-ags4-reference/data/ags_dictionary.json`. The
validator's `build.rs` then reads **that union** (not these `.ags`
directly) and projects each edition back out into its compiled lookup
tables; so does every other consumer (the typed-graph codegen, the web).
One source, no second parser. (A CI gate, `tests/test_dictionary_faithful.py`,
re-runs `gen_dictionary.py` and asserts the committed union still
reconstructs each edition from these files exactly.)

The validator auto-selects the matching edition from a file's
`TRAN_AGS` (see `src/lib.rs::resolve_dict_version`); all five editions
are carried so real-world deliveries (overwhelmingly AGS 4.0/4.1, not
4.2) are validated against their own schema, not a newer one.

## Source

Retrieved 2026-05-16 (4.0.3/4.0.4/4.1.1 added; 4.1/4.2 re-synced) from
the bundled copies shipped in the **`python-ags4`** package (v1.2.0),
the official validator maintained by the **AGS Data Format Working
Group** at <https://gitlab.com/ags-data-format-wg/ags-python-library>.
These five editions are exactly the set `python-ags4` itself ships
(its `STANDARD_DICT_FILES` map).

`python-ags4` is the AGS-DFWG's own reference implementation; the
dictionary files it ships are the authoritative machine-readable form of
the published AGS4 standard. Identical content is described in the AGS4.1
specification document (`reports/AGS 4_1.pdf`,
ISBN 978-0-9957482-1-7).

## Licence position (read this)

The AGS4 standard, including its data dictionary, is:

> © The Association of Geotechnical and Geoenvironmental Specialists
> (AGS). **All rights reserved.**

The specification is *freely downloadable* from <https://www.ags.org.uk>,
but "all rights reserved" carries no explicit grant to redistribute the
verbatim dictionary dataset.

**These files are bundled here as a deliberate, documented decision by
the repository owner**, who accepts the associated risk. The rationale:

1. The AGS Data Format Working Group's *own* official tooling
   (`python-ags4`, LGPL-3.0) already redistributes these exact files
   openly, signalling that redistribution in conformance tooling is
   the intended use.
2. The AGS4 format exists expressly to be implemented by third-party
   software; a machine-readable schema is functional reference data,
   and conformance checkers necessarily need it.
3. This crate's *code* is an independent clean-room implementation of
   the AGS4 rules (see `../README.md` and each `src/rules/*.rs`
   header) — it is **not** derived from `python-ags4`'s LGPL source.
   Only the dictionary *data* originates with AGS.

This note exists so the provenance and the decision are explicit and
auditable.

<!-- dict-fallback-claim -->
**The stated fallback now exists (laterite-dev#568).** If AGS objects to the bundled
dictionary, the retreat is the runtime `--dict` custom-dictionary override:
ship no bundled copy and require the user to supply their own
freely-downloaded dictionary (or an entirely bespoke one) at validation time.
`lat validate --dict <path>` accepts an AGS4 `.ags` **or** JSON dictionary and
validates the delivery against it; the same capability reaches every surface
(`dict_path` / `dict_bytes` on the Python and Node APIs, bytes-only in the
browser). `--dict-replace` drops the bundled base entirely for a fully
self-contained custom dictionary. So the bundling decision is no longer a
one-way door — a consumer who cannot rely on the embedded ©AGS data has a
documented, working way to substitute their own.

This note used to read "the fallback is the runtime `--dict <path>` mode
(already supported)" while the flag actually refused (`external --dict override
is not implemented`, O-28) — a capability asserted in one document and
contradicted in another, with nothing comparing them. That gap is now closed on
both sides: the capability is real (O-28 records the implementation across all
four surfaces), and `tests/test_provenance_dict_fallback.py` pins this paragraph
to it — the test fails if this claim is removed OR if `lat validate --dict`
stops working (a clean delivery validated against a custom dictionary must not
exit 5), so the claim cannot silently un-become true again.
<!-- /dict-fallback-claim -->

## This claim is enforced

Everything under **Source** above is checked, not asserted:
`tests/test_vendored_authority_faithful.py` compares these five files
**byte-for-byte** against the `python-ags4` copies installed in the dev
environment (it is a declared dev dependency — the parity oracle), checks
the file *set* still equals upstream's `STANDARD_DICT_FILES`, and checks
that every place in the tree stating a python-ags4 version agrees with the
one actually installed. It runs in the root suite, offline.

It exists because the claim was previously unguarded in a way that is easy
to misread. `tests/test_dictionary_faithful.py` re-runs `gen_dictionary.py`
and asserts the committed union matches — but that proves the union is
faithful to *whatever these files currently say*, not that these files are
faithful to AGS. Measured: appending a fabricated group to
`Standard_dictionary_v4_2.ags` and regenerating took the union from 174 to
175 groups, and that test still reported **5 passed**. The invented group
would have been compiled into the validator, the wasm build and the typed
graph with every gate green.

**What it does not prove:** that `python-ags4`'s dictionaries match the AGS
specification. These files come from our own parity oracle, so the parity
suite cannot detect a divergence the two of us share — both sides read the
same bytes. The argument for the source is the one made above (it is the
AGS-DFWG's own tooling); that argument is a *position*, and this test pins
us to the source we chose rather than auditing it.

## Refresh

`tools/sync-standard-dicts.sh` (Mac/Linux) or `tools/sync-standard-dicts.ps1`
(Windows) copies them from the installed `python-ags4`. When a new AGS4
edition is published, re-copy from the AGS-DFWG source, bump the retrieval
date above, and regenerate the union with `tools/gen_dictionary.py`. The
test named above will fail until the copy and this note agree again.
