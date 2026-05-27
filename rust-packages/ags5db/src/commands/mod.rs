//! One module per CLI subcommand. Each defines an `Args` struct (clap-derive)
//! and a `run` function that handles the command end-to-end.

pub mod agent_context;
pub mod ags4_to_db;
pub mod count;
pub mod db_to_ags4;
pub mod diff;
pub mod groups;
pub mod headings;
pub mod info;
pub mod inspect;
pub mod lock;
pub mod pack;
pub mod peek;
pub mod recipe;
pub mod sql;
pub mod sum;
pub mod unlock;
pub mod unpack;
