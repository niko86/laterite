# laterite

Read, validate and write **AGS4** — the data transfer format the UK
geotechnical and geoenvironmental industry uses to exchange ground
investigation data.

> **This crate is not yet at parity with laterite's other surfaces.** The Python,
> Node, browser, DuckDB and CLI surfaces are in beta; this one is still being
> built out, and its API will change. The engine underneath is the same one they
> all run. Use it if that suits you — just don't expect the surface to hold still
> yet. Cargo will not carry you across a `0.x` minor on the caret requirement
> `cargo add` writes, so the upgrade is yours to take; don't force one.
> [What beta means here](https://docs.laterite.dev/reference/support/).

```rust,no_run
use laterite::ags4;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = ags4::read("delivery.ags").run()?;
    for group in doc.groups() {
        println!("{} — {} rows", group.code(), group.len());
    }

    let report = ags4::validate("delivery.ags").warnings(true).run()?;
    for finding in report.findings() {
        println!("{}: {}", finding.rule(), finding.description());
    }

    // Or validate bytes, with no filesystem in the picture at all.
    let upload: Vec<u8> = std::fs::read("delivery.ags")?;
    let report = ags4::validate_bytes(upload).run()?;
    println!("{} finding(s)", report.findings().len());

    doc.set_cell("PROJ", 0, "PROJ_NAME", "Renamed site")?;
    ags4::write(&doc).to_path("out.ags")?;
    Ok(())
}
```

<!-- BEGIN GENERATED: availability — DO NOT EDIT BY HAND. Regenerate: uv run --no-project python tools/gen_crate_graph.py -->
## Install it

```bash
cargo add laterite
```

This crate versions independently of the engine.
<!-- END GENERATED: availability -->

## What it does

- **Read.** From a path or from bytes, with encoding handling — legacy delivery
  files are frequently `windows-1252` because of `°` and `±` in descriptions.
  Values come back **verbatim**, so `write(read(x))` preserves what the file
  carried. An `.ags` is often the contractual artefact; a reader that quietly
  normalises it is not doing you a favour.
- **Validate.** The full numbered rule set (Rules 1–20), against five bundled
  standard dictionaries with per-file edition auto-selection from `TRAN_AGS`.
  Findings carry a rule label, a group, a line and a severity. From a path or
  **from bytes** — a service that validates an upload need not give it a disk to
  sit on. The one difference is Rule 20's on-disk half, which asks whether the
  sibling `FILE/` tree really holds the attachments the file references: bytes
  have no sibling anything, so requesting it there is an error rather than a
  clean result.
- **Fix.** Mechanically repair a delivery — CRLF, BOM, embedded carriage
  returns, short rows, numeric formatting, the `TRAN` delimiter rows. The output
  is re-validated, so what comes back with it is what could *not* be fixed. The
  repairs that guess intent are withheld until you ask for them, and the result
  says how many it held back.
- **Build.** Construct AGS4 from data you hold — a query result, a spreadsheet,
  your own structs. Typed cells are formatted to their heading's declared AGS
  TYPE; strings are written as given.
- **Write.** Emit valid AGS4, deriving the `UNIT` and `TYPE` catalogue groups
  from the data. Choose whether to auto-fix, report, or refuse outright.
- **Certify.** Mint an `.ags.idx` certificate over a file's bytes, and offer one
  back so a validate can skip the rule engine or a read can slice a group
  straight out of its byte range.
- **Diff.** Compare two revisions in *AGS terms*: rows matched by their
  dictionary `KEY` headings rather than by line order, cells compared through
  their declared TYPE. A re-sorted file is not a change, and `1.0` → `1.00` is
  not a change. A line diff gets both wrong.
- **Merge.** Reconcile several deliveries of one project into one file, in
  order — a later file wins. A column two producers typed differently is
  refused unless you say how to settle it; a column they gave different *units*
  is refused outright, because nothing can settle that honestly.

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

## Scope

Read, validate, fix, build, write, certify, diff, merge, and `transport` —
compress (`pack`) or compress-and-encrypt (`lock`) any file, from a path or from
bytes in memory.

The crate is completing to **parity** with the Python and Node surfaces: per
capability, at least what the weaker of those two offers. Only Excel is still to
arrive, behind an optional feature; it is additive, so nothing here has to change
to admit it. When it reaches parity it joins the product version line.

There is no 0.2 — that milestone was retired in favour of going to parity once,
rather than stopping at a waypoint on a crate whose whole purpose is to be
stable.

## Other surfaces

The same engine ships as a Python package (`pip install laterite`), a Node
binding, a browser wasm build, and the `lat` command-line tool.

## Licence

MIT. The bundled standard dictionaries are ©AGS reference data — see
`PROVENANCE.md` in `laterite-ags4-validator`.
