#![allow(clippy::unwrap_used)]

use typst::foundations::{Dict, Str, Value};

use super::render;
use crate::World;

/// A small fixed-size page, kept tiny so rasterisation stays fast in tests.
const PAGE: &str = "#set page(width: 120pt, height: 80pt, margin: 8pt)\n";

fn world_for(body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
    let file = dir.path().join("main.typ");
    std::fs::write(&file, format!("{PAGE}{body}")).unwrap();
    (dir, file)
}

#[test]
fn renders_rgba_buffer() {
    let (_dir, file) = world_for("Hello, world.");
    let world = World::new(&file).unwrap().with_fonts();

    let frame = render(&world, 2.0).unwrap();
    assert!(frame.width > 0 && frame.height > 0);
    assert_eq!(frame.rgba.len(), (frame.width * frame.height * 4) as usize);
    // Text was actually drawn: not every pixel is the white page fill.
    assert!(frame.rgba.iter().any(|&byte| byte != 255));
}

#[test]
fn pixels_per_point_scales_dimensions() {
    let (_dir, file) = world_for("scale me");
    let world = World::new(&file).unwrap().with_fonts();

    let small = render(&world, 1.0).unwrap();
    let large = render(&world, 2.0).unwrap();
    assert!(large.width > small.width && large.height > small.height);
}

#[test]
fn inputs_reach_sys_inputs() {
    // Body interpolates a value only present if `sys.inputs.state` resolves.
    let (_dir, file) = world_for("#sys.inputs.state");
    let mut inputs = Dict::new();
    inputs.insert(Str::from("state"), Value::Str(Str::from("ready")));
    let world = World::new(&file).unwrap().with_fonts().with_inputs(inputs);

    assert!(render(&world, 1.0).is_ok());
}

#[test]
fn with_input_sets_a_single_string_entry() {
    let (_dir, file) = world_for("#json(bytes(sys.inputs.state)).at(\"k\")");
    let world = World::new(&file)
        .unwrap()
        .with_fonts()
        .with_input("state", "{\"k\": 1}");

    assert!(render(&world, 1.0).is_ok());
}

#[test]
fn set_inputs_batches_multiple_entries_in_one_rebuild() {
    let (_dir, file) = world_for("#json(bytes(sys.inputs.state)).at(\"k\") #sys.inputs.viewport");
    let mut world = World::new(&file).unwrap().with_fonts();
    world.set_inputs([
        ("state", "{\"k\": 1}".to_owned()),
        ("viewport", "{\"w\":1}".to_owned()),
    ]);

    assert!(render(&world, 1.0).is_ok());
}

#[test]
fn missing_input_is_a_render_error() {
    // Same body, but no inputs set: the dict-key access is a fatal diagnostic.
    let (_dir, file) = world_for("#sys.inputs.state");
    let world = World::new(&file).unwrap().with_fonts();

    assert!(render(&world, 1.0).is_err());
}
