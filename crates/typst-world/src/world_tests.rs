#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use typst::World as _;
use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, RootedPath, VirtualPath, VirtualRoot};

use super::{World, find_project_root};
use crate::provider::{FileProvider, ProviderCtx};

#[test]
fn root_marker_at_start() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
    assert_eq!(find_project_root(dir.path()), dir.path());
}

#[test]
fn root_walks_up_to_git() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    let child = dir.path().join("a").join("b");
    std::fs::create_dir_all(&child).unwrap();
    assert_eq!(find_project_root(&child), dir.path());
}

#[test]
fn new_reads_main_source() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
    let file = dir.path().join("task.typ");
    std::fs::write(&file, "= Hello").unwrap();

    let world = World::new(&file).unwrap();
    let source = world.source(world.main()).unwrap();
    assert!(source.text().contains("Hello"));
}

#[test]
fn with_root_rejects_outside_file() {
    let outer = tempfile::tempdir().unwrap();
    let inner = tempfile::tempdir().unwrap();
    let file = outer.path().join("task.typ");
    std::fs::write(&file, "= x").unwrap();
    // file is not under `inner`, so construction must fail.
    assert!(World::with_root(&file, inner.path()).is_err());
}

#[test]
fn new_rejects_missing_file() {
    assert!(World::new(Path::new("/nonexistent/x.typ")).is_err());
}

/// A bytes-backed [`FileProvider`] that ignores the disk-oriented context and
/// serves an id from a map — the shape the web target uses.
struct MapProvider(HashMap<FileId, Vec<u8>>);

impl FileProvider for MapProvider {
    fn read(&self, id: FileId, _ctx: &ProviderCtx<'_>) -> FileResult<Bytes> {
        self.0
            .get(&id)
            .map(|bytes| Bytes::new(bytes.clone()))
            .ok_or(FileError::AccessDenied)
    }
}

#[test]
fn custom_provider_serves_bytes_without_disk() {
    let main_id = FileId::new(RootedPath::new(
        VirtualRoot::Project,
        VirtualPath::new("main.typ").unwrap(),
    ));
    let mut map = HashMap::new();
    map.insert(main_id, b"= Embedded".to_vec());

    let world = World::with_provider(
        main_id,
        PathBuf::from("/virtual"),
        Box::new(MapProvider(map)),
    );
    let source = world.source(world.main()).unwrap();
    assert!(source.text().contains("Embedded"));
}
