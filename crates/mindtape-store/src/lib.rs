//! Store trait and `SQLite` backend for `MindTape`.
//!
//! This crate defines the `Store` trait (anti-corruption layer) and domain
//! types for persisting indexed tasks. The `SqliteStore` implements the
//! trait using `rusqlite`.

pub mod id;
mod store;

// Re-export everything at crate root.
pub use store::{
    FileBinding, FileDependencies, FileError, FileReference, FileView, IndexStats, PropertyKind,
    SortDir, SortField, SortSpec, SqliteStore, Store, StoreError, TaskFile, TaskFilter,
    TaskProperty, TaskRecord, TaskView, filter_expr::FilterExprError, hash_file, index_file,
    index_file_with_deps, to_store_records,
};
