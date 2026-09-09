---
type: tool
title: "laterite (the Rust crate)"
status: drafted
tags: [tool, rust, crate, published, architecture]
tool_kind: crate
language: rust
artifact: "laterite (crates.io) — the Rust facade over the engine crates"
ags_editions: []
repo_refs:
  root: "repo:rust-packages/laterite"
  manifest: "repo:rust-packages/laterite/Cargo.toml"
  readme: "repo:rust-packages/laterite/README.md"
related: [laterite, laterite-node, crate-map, dec-rust-api-crates-io, dec-facade-parity, crate-dependency-graph]
sources: []
---

# laterite (the Rust crate)

> [!warning] **Three different things are called `laterite`.** This page is the
> **Rust crate on crates.io** (`rust-packages/laterite`). It is not [[laterite]],
> the PyPI wheel (`packages/laterite`), and it is not the Python import root
> `import laterite` that the wheel installs. They ship on separate version lines
> to separate registries. The page stem here is `laterite-crate` because the stem
> `laterite` was already the wheel's — **`laterite-crate` is not a package name
> and `cargo add laterite-crate` resolves to nothing.** The crate is `laterite`.

<!-- BEGIN GENERATED: crate-card — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
> [!note] **Cleared for crates.io** — `laterite` declares `publish = true`, so it is a public API under semver, not an internal detail. It is versioned on its own line.
> **Used by** — nothing else in this workspace.
<!-- END GENERATED: crate-card -->

## What it is

The **facade**: one crate a Rust caller adds, over the engine crates that do the
work. Without it a Rust user faced a list of `laterite-ags4-*` crates and no
front door — the problem [[dec-rust-api-crates-io]] exists to answer.

It re-exports rather than reimplements. The engine crates
([[laterite-ags4-core]], [[laterite-ags4-emit]], [[laterite-ags4-parse]],
[[laterite-ags4-reference]], [[laterite-ags4-validator]]) stay independently
publishable and independently useful; this crate is the curated surface over
them, and `unstable-engine` is the escape hatch for callers who want the raw
crates without waiting for the facade to grow a verb.

## Its version line — the product's

Since parity ([[dec-facade-parity]] phase 8, 2026-09-04) the facade carries
the **product** number — the same version as the wheel, the npm package and
`lat`. The engine crates underneath version per-crate (#781) and their numbers
are not readable from this crate's. The card above states the number and which
line it is on.

## Parity — reached

The facade reached the capability floor 2026-09-04 ([[dec-facade-parity]]
phase 8): `tools/gen_modality.py --summary` reads to add = 0, UNDECIDED = 0,
which is decision 7's ratified predicate. [[modality-register]] still measures
per capability with a `facade_verdict` on each cell — `planned`, `by-design`
or `no-floor` — and records the one standing by-design exclusion
(`read-output-arrow`, whose engine door is `laterite-ags4-types`' `arrow`
feature). Excel joined the facade behind an optional feature; the CLI
deliberately did not.

## Relation to the other surfaces

| Thing | Where it lives | Registry | Version line |
|---|---|---|---|
| **this crate** | `repo:rust-packages/laterite` | crates.io | product |
| [[laterite]] — the wheel | `repo:packages/laterite` | PyPI | product |
| [[laterite-node]] — the Node addon | `repo:rust-packages/laterite-node` | npm | product |
| [[laterite-cli]] — `lat` | `repo:rust-packages/laterite-cli` | not published | product |

The Rust↔Python boundary runs one way — Rust drives Python, never the reverse
([[dec-rust-drives-python]]).

## Gotchas

- **`cargo add laterite` gets this crate; `pip install laterite` gets the
  wheel.** Same word, different artifact — one shared version line since
  parity (both carry the product number).
- The facade's dependency edges are the authority for what it can expose
  without a new dependency — see [[crate-dependency-graph]], which is generated
  from the manifests.
