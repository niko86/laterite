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
//! demands. This crate exists so that reshaping does not reach you.
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
//! The `unstable-engine` feature is the only way past the facade. It is a
//! feature rather than a hidden module because it shows up in *your*
//! `Cargo.toml` — reaching past a stability boundary should be something you
//! wrote down.
//!
//! # Scope
//!
//! Read, validate, fix, build, write, certify, and [`transport`] (compress /
//! encrypt any file).
//!
//! The crate is completing to **parity** with the Python and Node surfaces —
//! offering, per capability, at least what the weaker of those two offers — and
//! joins the product version line when it gets there. Still to arrive: diff,
//! merge and Excel. Each is additive, so nothing here has to change to admit
//! them.
//!
//! There is no 0.2. This paragraph used to promise one; the milestone was
//! retired in favour of going to parity once, because a 0.2 would have been a
//! waypoint that existed for as long as the remaining work took, on a crate
//! whose whole purpose is to be a stable surface.

#![forbid(unsafe_code)]

pub mod ags4;
mod error;
pub mod transport;

pub use error::{Error, ErrorKind};
