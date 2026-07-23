//! Output dispatch — now the shared `laterite_cliutil::report` module.
//!
//! The `Ctx`/`Report`/`emit`/`Plan`/`note`/`without_keys` scaffold was
//! lifted verbatim into `laterite-cliutil` (the one shared UX crate) so
//! this harness and `laterite-ags4-forge` share a single report contract
//! instead of a documented copy — the same extract-over-duplicate
//! stance that created `laterite-cliutil` itself. Kept as a thin re-export
//! so every existing `use crate::output::…` / `output::emit` /
//! `output::note` site is unchanged: behaviour byte-identical.

pub use laterite_cliutil::report::{Ctx, Plan, Report, emit, note, without_keys};
