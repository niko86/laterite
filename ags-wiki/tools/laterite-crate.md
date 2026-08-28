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

## Its own version line

It is the one crate here that does **not** inherit the workspace version. The
engine moves in lockstep on 0.9.x; the facade carries its own 0.1.x because it
is a different promise to a different audience — the engine's version tracks
the shared implementation, the facade's tracks the API a Rust consumer depends
on. The card above states both the number and which line it is on.

## Parity, and what it still owes

The facade does not yet reach the capability floor the other surfaces meet.
[[dec-facade-parity]] is the plan that closes it, and
[[modality-register]] measures the gap per capability with a `facade_verdict`
on each cell — `planned`, `by-design` or `no-floor` — so the shortfall can be
counted rather than read. Excel joins the facade behind an optional feature;
the CLI deliberately does not.

## Relation to the other surfaces

| Thing | Where it lives | Registry | Version line |
|---|---|---|---|
| **this crate** | `repo:rust-packages/laterite` | crates.io | its own 0.1.x |
| [[laterite]] — the wheel | `repo:packages/laterite` | PyPI | product |
| [[laterite-node]] — the Node addon | `repo:rust-packages/laterite-node` | npm | product |
| [[laterite-cli]] — `lat` | `repo:rust-packages/laterite-cli` | not published | product |

The Rust↔Python boundary runs one way — Rust drives Python, never the reverse
([[dec-rust-drives-python]]).

## Gotchas

- **`cargo add laterite` gets this crate; `pip install laterite` gets the
  wheel.** Same word, different artifact, different version number. A version
  like 0.1.2 is this crate; 0.10.x is the product line.
- The facade's dependency edges are the authority for what it can expose
  without a new dependency — see [[crate-dependency-graph]], which is generated
  from the manifests.
