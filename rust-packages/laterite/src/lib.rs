//! Read, validate and write **AGS4** — the geotechnical data transfer format.
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
//! // Validate
//! let report = ags4::validate("delivery.ags").warnings(true).run()?;
//! for finding in report.findings() {
//!     println!("{}: {}", finding.rule(), finding.description());
//! }
//!
//! // Modify and write
//! doc.set_cell("PROJ", 0, "PROJ_NAME", "Renamed site")?;
//! ags4::write(&doc).to_path("out.ags")?;
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
//!   `laterite::read` would have to mean one of them forever.
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
//! # Scope of 0.1
//!
//! Read, validate, write. Diff, merge, typed cell access and the indexed
//! `scan()` path are the engine's already and will surface here in 0.2 —
//! additively, so nothing here has to change to accommodate them.

#![forbid(unsafe_code)]

pub mod ags4;
mod error;

pub use error::{Error, ErrorKind};
