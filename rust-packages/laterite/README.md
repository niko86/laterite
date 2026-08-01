# laterite

Read, validate and write **AGS4** — the data transfer format the UK
geotechnical and geoenvironmental industry uses to exchange ground
investigation data.

```rust
use laterite::ags4;

let mut doc = ags4::read("delivery.ags").run()?;
for group in doc.groups() {
    println!("{} — {} rows", group.code(), group.len());
}

let report = ags4::validate("delivery.ags").warnings(true).run()?;
for finding in report.findings() {
    println!("{}: {}", finding.rule(), finding.description());
}

doc.set_cell("PROJ", 0, "PROJ_NAME", "Renamed site")?;
ags4::write(&doc).to_path("out.ags")?;
```

## What it does

- **Read.** From a path or from bytes, with encoding handling — legacy delivery
  files are frequently `windows-1252` because of `°` and `±` in descriptions.
  Values come back **verbatim**, so `write(read(x))` preserves what the file
  carried. An `.ags` is often the contractual artefact; a reader that quietly
  normalises it is not doing you a favour.
- **Validate.** The full numbered rule set (Rules 1–20), against five bundled
  standard dictionaries with per-file edition auto-selection from `TRAN_AGS`.
  Findings carry a rule label, a group, a line and a severity.
- **Write.** Emit valid AGS4, deriving the `UNIT` and `TYPE` catalogue groups
  from the data. Choose whether to auto-fix, report, or refuse outright.

The validator is **clean-room** — written from the AGS4.1 specification, not
derived from any existing implementation — and is cross-checked against the
reference Python library on its own test corpus.

## Two things worth knowing

**`TRAN` is never invented.** If you do not state a transmission, no `TRAN`
group is written and validation reports its absence. A synthesised placeholder
would be a claim about who transferred what, to whom, and when — not something a
writer can make up on your behalf.

**A duplicate heading is refused, not silently survived.** AGS4 forbids a group
declaring the same heading twice; rows are keyed by heading name, so read
naively the second column overwrites the first and you get a column that looks
fully populated and is not. Pass `.recover_duplicate_headings(true)` to rescue
the data instead, at the cost of a document that is deliberately no longer valid
AGS4.

## Stability

This crate is a facade. The work happens in a tier of `laterite-ags4-*` engine
crates that move on their own version and reshape as the format work demands;
this crate exists so that reshaping does not reach you. Concretely:

- Everything AGS4-specific is under `laterite::ags4` — the crate root stays
  format-neutral, because AGS4 is not the last version of the format.
- Handles are opaque, with private fields.
- **No third-party type appears in any public signature.** Encodings are WHATWG
  label strings, dates are ISO strings. No dependency's major version can force
  one of ours.
- One `Error` with a coarse, `#[non_exhaustive]` `ErrorKind` and a stable
  `kind_str()` shared with the Python, Node and CLI surfaces.

The `unstable-engine` feature is the only way past the facade, and it is a
feature rather than a hidden module so that reaching past a stability boundary
is something you wrote down in your own `Cargo.toml`.

## Scope of 0.1

Read, validate, write. Diff, merge, typed cell access and an indexed scan path
all exist in the engine already and will surface here in 0.2 — additively.

## Other surfaces

The same engine ships as a Python package (`pip install laterite`), a Node
binding, a browser wasm build, and the `lat` command-line tool.

## Licence

MIT. The bundled standard dictionaries are ©AGS reference data — see
`PROVENANCE.md` in `laterite-ags4-validator`.
