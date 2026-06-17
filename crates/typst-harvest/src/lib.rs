//! Eval-only Typst World plus labelled-metadata-marker harvesting.
//!
//! A package-agnostic substrate for file-based tools that use Typst documents
//! as a structured data medium. It evaluates a `.typ` file with
//! [`typst_eval::eval`] (never layout or render) and collects the `metadata()`
//! markers it emits, projected to a Typst-free [`HVal`] tree.
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
pub mod value;
pub mod world;

pub use harvest::{Harvest, Location, Marker, harvest};
pub use value::{HVal, convert, format_date};
pub use world::{HarvestWorld, find_project_root};

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
