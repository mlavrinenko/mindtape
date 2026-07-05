//! Compile a world's main file and rasterise or vectorise its first page.
//!
//! The render counterpart to the eval/harvest path: where harvesting evaluates
//! (fonts off, no layout), this runs the full `typst::compile` pipeline and hands
//! the caller a ready-to-present frame. The world must carry fonts (see
//! [`World::with_fonts`]); inject per-frame state with [`World::with_inputs`].

use typst::utils::Scalar;
use typst_layout::{Page, PagedDocument};
use typst_render::RenderOptions;
use typst_svg::SvgOptions;

use crate::WorldError;
use crate::world::{World, format_diagnostics};

/// A rendered raster frame: premultiplied RGBA8 pixels plus dimensions, the
/// channel order the shell presents directly (it owns any channel swap at the
/// present boundary). `rgba.len() == width * height * 4`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Premultiplied RGBA8 bytes, row-major, top-left origin.
    pub rgba: Vec<u8>,
}

/// Compile `world`'s main file and rasterise its first page at
/// `pixels_per_point`.
///
/// Repeated calls with only `sys.inputs` changed stay cheap: comemo memoises the
/// layout work that the unchanged source shares across frames.
///
/// # Errors
/// Returns [`WorldError::Eval`] if compilation produces fatal diagnostics, or
/// [`WorldError::World`] if the document has no pages.
pub fn render(world: &World, pixels_per_point: f32) -> Result<Frame, WorldError> {
    let page = first_page(world)?;
    let opts = RenderOptions {
        pixel_per_pt: Scalar::new(f64::from(pixels_per_point)),
        render_bleed: false,
    };
    let pixmap = typst_render::render(&page, &opts);
    Ok(Frame {
        width: pixmap.width(),
        height: pixmap.height(),
        rgba: pixmap.take(),
    })
}

/// Compile `world`'s main file and vectorise its first page to an SVG string —
/// the vector analog of [`render`], resolution-free so there is no
/// `pixels_per_point`.
///
/// # Errors
/// As [`render`].
pub fn render_svg(world: &World) -> Result<String, WorldError> {
    let page = first_page(world)?;
    Ok(typst_svg::svg(&page, &SvgOptions::default()))
}

/// How many [`comemo::evict`] passes a memoized result may go unused before
/// eviction. comemo's cache is process-global and every `typst::compile` call
/// feeds it, so a session's thousands of re-renders (redraw on every MIDI
/// note/key/resize tick) grow it unbounded unless something evicts. Called once
/// per compile below, so `EVICT_MAX_AGE` frames of the same unreferenced result
/// survive before it is dropped — enough to keep memoized layout warm across a
/// short burst of frames without letting the cache grow without bound.
const EVICT_MAX_AGE: usize = 10;

/// Compile `world`'s main file and hand back its first page, evicting comemo's
/// global memoization cache afterwards (see [`EVICT_MAX_AGE`]) so the cache
/// stays bounded across the many compiles one render loop issues.
fn first_page(world: &World) -> Result<Page, WorldError> {
    let world_dyn: &dyn typst::World = world;
    let output = typst::compile::<PagedDocument>(world_dyn).output;
    comemo::evict(EVICT_MAX_AGE);
    let document = output.map_err(|diags| WorldError::Eval(format_diagnostics(&diags)))?;
    document
        .pages()
        .first()
        .cloned()
        .ok_or_else(|| WorldError::World("document produced no pages".to_owned()))
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
