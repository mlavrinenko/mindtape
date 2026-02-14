#![warn(
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::cloned_instead_of_copied,
    clippy::flat_map_option,
    clippy::semicolon_if_nothing_returned,
    clippy::uninlined_format_args
)]

pub mod cli;
pub mod config;
pub mod eval;
pub mod store;
pub mod watcher;
pub mod world;
