pub mod cli;
pub mod config;
pub mod watcher;

// Re-export sub-crate APIs for convenience and backwards compatibility.
pub use mindtape_eval as eval;
pub use mindtape_eval::world;
pub use mindtape_store as store;

/// Typst package version for `@local/mindtape:VERSION`.
///
/// Read from `lib/typst.toml` at build time by `build.rs`.
pub const TYPST_PACKAGE_VERSION: &str = env!("TYPST_PACKAGE_VERSION");
