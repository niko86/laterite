# Domain Docs

How the engineering skills should consume this repo's domain documentation when
exploring the codebase.

This repo does **not** use the `CONTEXT.md` + `docs/adr/` layout the skills
otherwise assume. It has that layer already, under a different name and a good
deal older: the wiki at **`ags-wiki/`**. Scaffolding a second, empty docs home
next to it would split the *why* across two places, which is precisely what the
wiki exists to prevent. What follows maps the skills' contract onto it.

## Before exploring, read these

- **`ags-wiki/start-here.md`** — orientation. Read this instead of a root `CONTEXT.md`.
- **`ags-wiki/concepts/crate-map.md`** — the keystone. What each crate is, why it
  is a separate crate, and what depends on what. Read it before touching crate
  layout, the wheel/dep/build split, or the PyO3 / wasm boundary.
- **`ags-wiki/design/dec-*.md`** — the ADRs, by another name. ~20 decision
  records: `dec-facade-parity`, `dec-rust-drives-python`, `dec-monorepo-structure`,
  `dec-dictionary-single-source`, `dec-ags-idx-certificate`, `dec-rust-api-crates-io`,
  and the rest. The other pages under `design/` are working design documents
  rather than settled decisions — `reliquary.md` is the living register of known
  relics.

There is no `CONTEXT-MAP.md` and no per-crate `CONTEXT.md`. Crate-level context
lives in `crate-map.md` and in the per-crate tool pages under `ags-wiki/tools/`.

## Finding what covers the files you're about to change

Don't guess from filenames, and don't conclude from a failed stem lookup that
nothing covers a file — the page covering
`rust-packages/laterite-ags4-reference/src/dict.rs` is called
`edition-resolution.md`. Ask instead:

```bash
uv run --no-project python ags-wiki/.bootstrap/librarian.py --paths <files…>
```

Under a second, no build. It inverts the `repo:` citations already carried on the
pages and prints them ranked with their titles, marking a hit that comes only
from a page citing a parent *directory* as `(directory only — may not describe
this file)`.

## Treat a page as a pointer, not gospel

Before relying on a load-bearing claim, verify it against the repo authority the
page cites — `observations.json`, `ags_dictionary.json`, the validator rule
modules. Code moves under pages.

Several wiki files are **generated views, not sources**: `OBSERVATIONS.md`,
`ags-wiki/index.md`, `concepts/modality-register.md`, `concepts/mutation-sweep.md`,
`concepts/crate-dependency-graph.md`, `design/reliquary.md`. Edit the JSON beside
them and regenerate with the matching `tools/gen_*.py`; never hand-edit a
rendered file.

## Use the wiki's vocabulary

When your output names a domain concept — an issue title, a refactor proposal, a
hypothesis, a test name — use the term as the wiki defines it. A few that matter,
because the near-synonyms are actively wrong here:

- An **observation** is an `O-N` record in the divergence catalogue, whose source
  of truth is `observations.json` at the repo root and which needs a matching
  `ags-wiki/observations/O-NN.md` page. It is not a general remark.
- The **facade** is the `laterite` crate; the **engine** is the tier of
  `laterite-ags4-*` crates beneath it. The **floor** is the parity number, and it
  is computed, never stored.
- **AGS5 is a dormant concept**, never a shipped feature, in anything
  public-facing.
- A **non-falsifiable test** is one an incorrect implementation satisfies as
  readily as a correct one — the thing the mutation sweep exists to catch.

If the concept you need isn't in the wiki yet, that's a signal: either you're
inventing language the project doesn't use (reconsider), or there's a real gap
worth a page.

## Flag decision conflicts

If your output contradicts a `dec-*.md`, surface it explicitly rather than
quietly overriding it:

> _Contradicts `dec-rust-drives-python` — but worth reopening because…_

Then stop and ask. Reversing a recorded decision is the maintainer's call, and
the rationale behind one is often not the axis the new evidence measures.
