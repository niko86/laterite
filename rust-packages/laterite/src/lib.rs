//! Read, validate, repair and write **AGS4** — the geotechnical data transfer
//! format.
//!
//! ```no_run
//! use laterite::ags4;
//!
//! // Read
//! let mut doc = ags4::read("delivery.ags").run()?;
//! for group in doc.groups() {
//!     println!("{} — {} rows", group.code(), group.len());
//! }
//!
//! // Validate — from a path...
//! let report = ags4::validate("delivery.ags").warnings(true).run()?;
//! for finding in report.findings() {
//!     println!("{}: {}", finding.rule(), finding.description());
//! }
//!
//! // ...or from bytes that never touch a filesystem, for a service that
//! // validates an upload without giving it a disk to sit on.
//! let upload: &[u8] = b"\"GROUP\",\"PROJ\"\r\n";
//! let report = ags4::validate_bytes(upload).run()?;
//!
//! // Modify and write
//! doc.set_cell("PROJ", 0, "PROJ_NAME", "Renamed site")?;
//! ags4::write(&doc).to_path("out.ags")?;
//!
//! // Repair a delivery mechanically — the result carries what could NOT be
//! // fixed, and the source file is untouched until you name a destination.
//! let fixed = ags4::fix("delivery.ags").run()?;
//! println!("{} repaired, {} left", fixed.fixes_applied(), fixed.findings().len());
//!
//! // Or construct AGS4 from data you hold rather than a file you read.
//! use laterite::ags4::GroupData;
//! let proj = GroupData::new("PROJ", ["PROJ_ID", "PROJ_NAME"]).row(["P1", "A site"]);
//! let built = ags4::build(vec![proj]).run()?;
//! # Ok::<(), laterite::Error>(())
//! ```
//!
//! # What this crate promises
//!
//! It is a **facade**. The work happens in a tier of `laterite-ags4-*` engine
//! crates, which move on their own version and reshape as the format work
//! demands. This crate exists so that *their* reshaping does not reach you —
//! a promise about the engine, and not the same thing as a promise that this
//! crate's own surface holds still. Both clauses below; neither is worth much
//! printed alone.
//!
//! Concretely, and these are the rules the API is built to:
//!
//! - **Everything AGS4-specific lives under [`ags4`].** The crate root stays
//!   format-neutral. AGS4 is not the last version of the format, and a root-level
//!   `laterite::read` would have to mean one of them forever. [`transport`] is at
//!   the root because it genuinely is format-neutral — zstd and age over any
//!   bytes.
//! - **Handles are opaque.** [`ags4::Document`], [`ags4::Report`] and the rest
//!   have private fields. You reach data through methods, so the engine's own
//!   structs can change shape without editing your code.
//! - **No third-party type appears in a public signature.** Not
//!   `serde_json::Value`, not `chrono`, not `encoding_rs`, not `arrow`. Encodings
//!   are WHATWG label strings, dates are ISO strings. This is the highest-leverage
//!   rule here: no dependency's major version can ever force one of ours.
//! - **One [`Error`]** with a coarse, `#[non_exhaustive]` [`ErrorKind`] and a
//!   stable [`Error::kind_str`] shared with the Python, Node and `lat` surfaces.
//!
//! **Those four rules absorb the engine. Nothing absorbs this crate.** It is
//! pre-1.0, on its own version line, and still being completed to parity with
//! the Python and Node surfaces — its API will change, and a minor release may
//! break you. Cargo will not carry you across a `0.x` minor on a caret
//! requirement, which is what `cargo add` writes, so the upgrade is yours to
//! take rather than something that happens to you. Don't force it.
//!
//! What that means for every surface is stated once, at
//! <https://docs.laterite.dev/reference/support/>.
//!
//! The `unstable-engine` feature is the only way past the facade. It is a
//! feature rather than a hidden module because it shows up in *your*
//! `Cargo.toml` — reaching past a stability boundary should be something you
//! wrote down.
//!
//! # Scope
//!
//! Read, validate, fix, build, write, certify, diff, merge, and [`transport`]
//! (compress / encrypt any file).
//!
//! The crate is completing to **parity** with the Python and Node surfaces —
//! offering, per capability, at least what the weaker of those two offers — and
//! joins the product version line when it gets there. Only Excel is still to
//! arrive, behind an optional feature; it is additive, so nothing here has to
//! change to admit it.
//!
//! There is no 0.2. This paragraph used to promise one; the milestone was
//! retired in favour of going to parity once, because a 0.2 would have been a
//! waypoint that existed for as long as the remaining work took, on a crate
//! whose purpose is to be a stable surface *over a moving engine*.

#![forbid(unsafe_code)]

pub mod ags4;
mod error;
pub mod transport;

pub use error::{Error, ErrorKind};

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
