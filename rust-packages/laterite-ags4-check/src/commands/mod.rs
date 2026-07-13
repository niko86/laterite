//! The `lat` subcommand handlers. Each `run` diverges (`-> !`) — it renders +
//! `exit`s with the contract code — so `main`'s dispatch match needs no result
//! plumbing. Shared encoding/edition/path helpers live in `common`.

pub mod cert;
pub mod certify;
pub mod common;
pub mod diff;
#[cfg(feature = "excel")]
pub mod excel;
pub mod fix;
pub mod merge;
pub mod read;
pub mod rules;
#[cfg(feature = "transport")]
pub mod transport;
pub mod validate;
