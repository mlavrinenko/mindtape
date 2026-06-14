//! Evaluate a `.typ` file and collect its `metadata()` markers.

use std::ops::ControlFlow;
use std::path::PathBuf;

use comemo::Track;
use typst::diag::SourceDiagnostic;
use typst::engine::{Route, Sink, Traced};
use typst::foundations::{Content, Module};
use typst::{ROUTINES, World};
use typst_library::introspection::MetadataElem;

use crate::HarvestError;
use crate::value::{HVal, convert};
use crate::world::HarvestWorld;

/// One harvested `metadata()` payload, projected to an [`HVal`].
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// The metadata value.
    pub value: HVal,
}

/// The result of harvesting a file: its markers and project-local dependencies.
#[derive(Debug, Clone)]
pub struct Harvest {
    /// Every `metadata()` marker found, in document order.
    pub markers: Vec<Marker>,
    /// Project-local files read during evaluation (relative to root).
    pub dependencies: Vec<PathBuf>,
}

impl Harvest {
    /// Markers whose `marker` key equals `name` (the namespacing convention).
    pub fn with_marker<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Marker> {
        self.markers
            .iter()
            .filter(move |marker| marker.value.get("marker").and_then(HVal::as_str) == Some(name))
    }
}

/// Evaluate `world`'s main file and harvest its metadata markers.
///
/// # Errors
/// Returns an error if the source cannot be read or Typst evaluation fails.
pub fn harvest(world: &HarvestWorld) -> Result<Harvest, HarvestError> {
    let world_dyn: &dyn World = world;
    let source = world
        .source(world.main())
        .map_err(|err| HarvestError::File(err.to_string()))?;

    let mut sink = Sink::new();
    let traced = Traced::default();
    let route = Route::default();

    let module: Module = typst_eval::eval(
        &ROUTINES,
        world_dyn.track(),
        traced.track(),
        sink.track_mut(),
        route.track(),
        &source,
    )
    .map_err(|diags| HarvestError::Eval(format_diagnostics(&diags)))?;

    let content: Content = module.content();
    let mut markers = Vec::new();
    collect_markers(&content, &mut markers);
    let dependencies = world.dependencies()?;
    Ok(Harvest {
        markers,
        dependencies,
    })
}

fn collect_markers(content: &Content, out: &mut Vec<Marker>) {
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(meta) = node.to_packed::<MetadataElem>() {
            out.push(Marker {
                value: convert(&meta.value),
            });
        }
        ControlFlow::Continue(())
    });
}

fn format_diagnostics(diags: &[SourceDiagnostic]) -> String {
    if diags.is_empty() {
        return "unknown evaluation error".to_owned();
    }
    diags
        .iter()
        .map(|diag| diag.message.as_str().to_owned())
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
#[path = "harvest_tests.rs"]
mod tests;
