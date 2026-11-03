//! The `@local` package files a commit state's pins resolve, and the
//! fingerprint that lets evaluations be reused when two states resolve imports
//! identically.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use sha1::{Digest, Sha1};
use typst::syntax::package::{PackageManifest, PackageSpec};
use typst_world::{BytesProvider, local_package_file_id};

use crate::config::Config;
use crate::loader::{StatePackages, package_overrides};

/// One package file: its path within the package and its bytes.
pub(crate) type PackageBlob = (String, Vec<u8>);

/// One package's spec and files, when a pin tree holds a readable manifest.
type CachedPackage = Option<(PackageSpec, Vec<PackageBlob>)>;

/// The last package files read for each pin name. A range walks commits in
/// order, so a pin's previous tree is the only other one it can need; keeping
/// every historical tree would hold a long prelude history in memory.
#[derive(Default)]
pub(crate) struct PackageCache {
    entries: HashMap<String, (String, CachedPackage)>,
}

impl PackageCache {
    /// The package at `name` and pin `tree`, when this cache holds that tree.
    pub(crate) fn get(&self, name: &str, tree: &str) -> Option<CachedPackage> {
        self.entries
            .get(name)
            .filter(|(found_tree, _)| found_tree == tree)
            .map(|(_, found)| found.clone())
    }

    /// Record the package at `name` and pin `tree`, replacing that pin's
    /// previous tree.
    pub(crate) fn insert(&mut self, name: &str, tree: &str, found: CachedPackage) {
        self.entries
            .insert(name.to_owned(), (tree.to_owned(), found));
    }
}

/// Package files gathered from one state, with the fingerprint that lets a
/// caller reuse evaluations when two states resolve imports identically.
#[derive(Default)]
pub(crate) struct PackageSet {
    specs: Vec<PackageSpec>,
    files: BytesProvider,
    hash: Sha1,
}

impl PackageSet {
    /// Add one package's spec and files, hashing the bytes a harvest would
    /// read.
    pub(crate) fn add(&mut self, spec: PackageSpec, blobs: Vec<PackageBlob>) {
        let version = spec.version.to_string();
        self.hash.update(b"\0package\0");
        self.hash.update(spec.namespace.as_str().as_bytes());
        self.hash.update([0]);
        self.hash.update(spec.name.as_str().as_bytes());
        self.hash.update([0]);
        self.hash.update(version.as_bytes());
        for (vpath, bytes) in blobs {
            let Ok(id) = local_package_file_id(spec.name.as_str(), &version, &vpath) else {
                continue;
            };
            self.hash.update([0, b'f', 0]);
            self.hash.update(vpath.as_bytes());
            self.hash.update((bytes.len() as u64).to_le_bytes());
            self.hash.update(&bytes);
            let files = std::mem::replace(&mut self.files, BytesProvider::new());
            self.files = files.with_file(id, bytes);
        }
        self.specs.push(spec);
    }

    /// The state's packages, or `None` when no pin contributed one.
    pub(crate) fn finish(self) -> Option<StatePackages> {
        if self.specs.is_empty() {
            return None;
        }
        let fingerprint = crate::loader::cache::hex(&self.hash.finalize());
        Some(StatePackages {
            specs: self.specs,
            files: Arc::new(self.files),
            fingerprint,
        })
    }
}

/// The package files the working tree's pins hold, fingerprinted by the same
/// recipe as a state's so the two compare. The bytes come from the working
/// tree, not the state's git tree. `None` when a pin cannot be read, which
/// disables the reuse path rather than guessing.
pub(crate) fn working_packages(root: &Path, config: &Config) -> Option<StatePackages> {
    let mut set = PackageSet::default();
    for (name, dir) in package_overrides(root, config) {
        let manifest = std::fs::read_to_string(dir.join("typst.toml")).ok()?;
        let manifest: PackageManifest = toml::from_str(&manifest).ok()?;
        let spec = PackageSpec {
            namespace: "local".into(),
            name: name.as_str().into(),
            version: manifest.package.version,
        };
        let mut paths = Vec::new();
        crate::loader::cache::collect_files(&dir, Path::new(""), &mut paths);
        paths.sort();
        let mut blobs = Vec::new();
        for (rel, path) in paths {
            let bytes = std::fs::read(path).ok()?;
            blobs.push((crate::slash_path(&rel), bytes));
        }
        set.add(spec, blobs);
    }
    set.finish()
}

#[cfg(test)]
#[path = "packages_tests.rs"]
mod tests;
