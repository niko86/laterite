# Standard dictionary provenance & licence note

## What these files are

`Standard_dictionary_v4_0_3.ags`, `Standard_dictionary_v4_0_4.ags`,
`Standard_dictionary_v4_1.ags`, `Standard_dictionary_v4_1_1.ags`, and
`Standard_dictionary_v4_2.ags` are the machine-readable AGS4 **Data
Dictionary** for each published edition — the canonical list of group
codes, heading names, AGS data types, units, status, and descriptions
that the AGS4 transfer-format standard defines. They are the reference
schema this validator checks files against (Rules 7, 9, 16, 17, 19).
The validator auto-selects the matching edition from a file's
`TRAN_AGS` (see `src/lib.rs::resolve_dict_version`); all five are
bundled so real-world deliveries (overwhelmingly AGS 4.0/4.1, not 4.2)
are validated against their own schema, not a newer one.

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
auditable. If AGS objects, the fallback is the runtime `--dict <path>`
mode (already supported): ship no bundled dictionary and require the
user to supply their own freely-downloaded copy.

## Refresh

`tools/sync-standard-dicts.ps1` documents the retrieval command. When a
new AGS4 edition is published, re-copy from the AGS-DFWG source and bump
the retrieval date above.
