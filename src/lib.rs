pub mod cli;
pub mod config;
pub mod watcher;

// Re-export sub-crate APIs for convenience and backwards compatibility.
pub use mindtape_eval as eval;
pub use mindtape_eval::world;
pub use mindtape_store as store;
