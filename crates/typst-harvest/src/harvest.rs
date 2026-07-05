//! Evaluate a `.typ` file and collect its `metadata()` markers.

use std::ops::ControlFlow;
use std::path::PathBuf;

use comemo::Track;
use typst::engine::{Route, Sink, Traced};
use typst::foundations::{Content, Module};
use typst::syntax::{Span, VirtualRoot};
use typst::{World as _, WorldExt as _};
use typst_library::introspection::MetadataElem;
use typst_world::{HVal, World, convert, format_diagnostics};

use crate::HarvestError;

/// Where a marker was emitted: a tree-relative (or package-qualified) path and a
/// 1-based line number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Source path. Tree-relative for a project file, or `{pkg-spec}/{vpath}`
    /// for a file imported from an `@local/<name>:<version>` package, so the
    /// identity is stable across consuming trees.
    pub path: String,
    /// 1-based line of the marker's emitting call.
    pub line: usize,
}

/// One harvested `metadata()` payload, projected to an [`HVal`].
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    /// The metadata value.
    pub value: HVal,
    /// The label attached to the marker (`#metadata(..) <name>`), if any. Tools
    /// that namespace by label (rather than by a dict key) filter on this.
    pub label: Option<String>,
    /// Source location of the emitting call, when resolvable.
    pub location: Option<Location>,
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
    /// Markers whose `marker` key equals `name` (the dict-key namespacing
    /// convention).
    pub fn with_marker<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Marker> {
        self.markers
            .iter()
            .filter(move |marker| marker.value.get("marker").and_then(HVal::as_str) == Some(name))
    }

    /// Markers carrying the label `name` (the `<name>` namespacing convention).
    pub fn with_label<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Marker> {
        self.markers
            .iter()
            .filter(move |marker| marker.label.as_deref() == Some(name))
    }
}

/// Evaluate `world`'s main file and harvest its metadata markers.
///
/// # Errors
/// Returns an error if the source cannot be read or Typst evaluation fails.
pub fn harvest(world: &World) -> Result<Harvest, HarvestError> {
    let world_dyn: &dyn typst::World = world;
    let source = world
        .source(world.main())
        .map_err(|err| HarvestError::File(err.to_string()))?;

    let mut sink = Sink::new();
    let traced = Traced::default();
    let route = Route::default();

    let module: Module = typst_eval::eval(
        world_dyn.track(),
        world.library(),
        traced.track(),
        sink.track_mut(),
        route.track(),
        &source,
    )
    .map_err(|diags| HarvestError::Eval(format_diagnostics(&diags)))?;

    let content: Content = module.content();
    let mut markers = Vec::new();
    collect_markers(world, &content, &mut markers);
    let dependencies = world.dependencies()?;
    Ok(Harvest {
        markers,
        dependencies,
    })
}

fn collect_markers(world: &World, content: &Content, out: &mut Vec<Marker>) {
    let _ = content.traverse(&mut |node: Content| -> ControlFlow<()> {
        if let Some(meta) = node.to_packed::<MetadataElem>() {
            out.push(Marker {
                value: convert(&meta.value),
                label: node
                    .label()
                    .map(|label| label.resolve().as_str().to_owned()),
                location: span_to_location(node.span(), world),
            });
        }
        ControlFlow::Continue(())
    });
}

/// Resolve a Typst [`Span`] to a [`Location`]: path plus 1-based line. Files
/// imported from an `@local/<name>:<version>` package are qualified with the
/// package spec so their identity is stable across consuming trees.
fn span_to_location(span: Span, world: &World) -> Option<Location> {
    let file_id = span.id()?;
    let source = world.source(file_id).ok()?;
    let range = world.range(span)?;
    let line = source.lines().byte_to_line(range.start)? + 1;
    let vpath = file_id.vpath().get_with_slash();
    let vpath = vpath.strip_prefix('/').unwrap_or(vpath);
    let path = match file_id.root() {
        VirtualRoot::Package(spec) => format!("{spec}/{vpath}"),
        VirtualRoot::Project => vpath.to_owned(),
    };
    Some(Location { path, line })
}

#[cfg(test)]
#[path = "harvest_tests.rs"]
mod tests;
