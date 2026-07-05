//! A configurable eval-only [`typst::World`] over a file-provider seam, plus a
//! Typst-free value tree.
//!
//! The world resolves project-local imports against a root directory and
//! `@local/<name>:<version>` packages against the platform local-package
//! directory (or a caller-supplied in-repo override). All byte reads go through
//! a [`FileProvider`], so the world is target-agnostic: native reads disk via
//! [`DiskProvider`]; a web build feeds sources, fonts, and packages from
//! embedded bytes behind the same trait ([`BytesProvider`]).
//!
//! It evaluates a `.typ` file with `typst_eval` (never layout or render, unless
//! the `render` feature is on) and projects Typst values to a Typst-free [`HVal`]
//! tree. Marker harvesting on top of this substrate lives in the `typst-harvest`
//! crate, so `typst-world` stays a pure World + value substrate.
//!
//! ```no_run
//! use typst_world::World;
//! # fn run() -> Result<(), typst_world::WorldError> {
//! let world = World::new(std::path::Path::new("doc.typ"))?;
//! println!("{}", world.root().display());
//! # Ok(())
//! # }
//! ```

mod provider;
#[cfg(feature = "render")]
mod render;
mod value;
mod world;

pub use provider::{
    BytesProvider, DiskProvider, FileProvider, ProviderCtx, local_package_file_id, resolve_path,
    root_file_id,
};
#[cfg(feature = "render")]
pub use render::{Frame, render, render_svg};
pub use value::{HVal, convert, format_date};
pub use world::{World, find_project_root, format_diagnostics};

/// Errors raised while building a world, evaluating, or querying it.
#[derive(Debug, thiserror::Error)]
pub enum WorldError {
    /// The main source file could not be read.
    #[error("file error: {0}")]
    File(String),

    /// Typst evaluation produced diagnostics.
    #[error("eval error: {0}")]
    Eval(String),

    /// The world could not be constructed or queried.
    #[error("{0}")]
    World(String),
}
