//! Labelled-metadata-marker harvesting over an eval-only Typst World.
//!
//! A thin layer on the [`typst_world`] crate, which owns the eval-only
//! [`typst::World`] and the Typst-free [`HVal`] value tree. This crate adds the
//! harvesting step: it evaluates a `.typ` file with [`typst_eval::eval`] (never
//! layout or render) and collects the `metadata()` markers it emits, projected
//! to the [`HVal`] tree.
//!
//! ```no_run
//! use typst_harvest::{HarvestWorld, harvest};
//! # fn run() -> Result<(), typst_harvest::HarvestError> {
//! let world = HarvestWorld::new(std::path::Path::new("task.typ"))?;
//! let result = harvest(&world)?;
//! for marker in result.with_marker("mindtape.task") {
//!     println!("{:?}", marker.value);
//! }
//! # Ok(())
//! # }
//! ```

mod harvest;

pub use harvest::{Harvest, Location, Marker, harvest};
pub use typst_world::{HVal, World as HarvestWorld, convert, find_project_root, format_date};

/// Errors raised while harvesting a Typst file.
#[derive(Debug, thiserror::Error)]
pub enum HarvestError {
    /// The source file could not be read.
    #[error("file error: {0}")]
    File(String),

    /// Typst evaluation produced diagnostics.
    #[error("eval error: {0}")]
    Eval(String),

    /// The world could not be constructed or queried.
    #[error("{0}")]
    World(String),
}

impl From<typst_world::WorldError> for HarvestError {
    fn from(err: typst_world::WorldError) -> Self {
        match err {
            typst_world::WorldError::File(msg) => Self::File(msg),
            typst_world::WorldError::Eval(msg) => Self::Eval(msg),
            typst_world::WorldError::World(msg) => Self::World(msg),
        }
    }
}
