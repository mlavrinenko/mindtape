//! Version-aware `[[typst.local-packages]]` pins: a pin serves the version its
//! `typst.toml` declares, and every other version falls through to the package
//! directory. [`super::with_packages`] installs the result in every world
//! Mindtape builds.
//!
//! A commit state can hand in its own package files ([`StatePackages`]) so a
//! past tree's imports resolve against that tree's `lib/`, not the working
//! tree's pin or the machine's install.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::package::{PackageManifest, PackageSpec, PackageVersion};
use typst::syntax::{FileId, VirtualRoot};
use typst_harvest::HarvestWorld;
use typst_world::{FileProvider, ProviderCtx};

use super::embedded;

/// One commit state's own `@local` package files, keyed by the versions its
/// manifests declare. The files come from the state's git tree, so a task file
/// from that tree resolves its imports against the package that tree held.
#[derive(Clone)]
pub(crate) struct StatePackages {
    pub(crate) specs: Vec<PackageSpec>,
    pub(crate) files: Arc<dyn FileProvider>,
}

/// [`super::with_packages`] with the installed-package root named, so a test
/// resolves a version under a temp directory without touching the process
/// environment.
pub(super) fn with_packages_at(
    world: HarvestWorld,
    overrides: &[(String, PathBuf)],
    package_root: Option<PathBuf>,
) -> HarvestWorld {
    with_packages_and(world, overrides, package_root, None)
}

/// [`with_packages_at`] with a commit state's own package files tried first.
pub(crate) fn with_packages_and(
    world: HarvestWorld,
    overrides: &[(String, PathBuf)],
    package_root: Option<PathBuf>,
    state: Option<StatePackages>,
) -> HarvestWorld {
    let pins = overrides
        .iter()
        .map(|(name, dir)| Pin::new(name, dir))
        .collect();
    let mut world = world.with_file_provider(Box::new(Pins {
        state,
        pins,
        fallback: embedded::package_provider(package_root),
        no_overrides: HashMap::new(),
    }));
    // The world resolves its recorded dependencies through this map, not
    // through the provider above. Keep every pin in it so a world with no
    // platform package root still records what it read.
    for (name, dir) in overrides {
        world = world.with_local_package(name, dir.clone());
    }
    world
}

/// One `[[typst.local-packages]]` entry: the package name it pins and the
/// version its directory's `typst.toml` declares.
struct Pin {
    name: String,
    /// `None` when the manifest cannot be read or parsed. The pin then serves
    /// every version, so Typst reports the pin's own manifest problem rather
    /// than a not-found for a package the project never asked the machine for.
    version: Option<PackageVersion>,
    dir: PathBuf,
}

impl Pin {
    fn new(name: &str, dir: &Path) -> Self {
        Self {
            name: name.to_owned(),
            version: manifest_version(dir),
            dir: dir.to_path_buf(),
        }
    }

    /// Whether this pin is the package `spec` names at `spec`'s version.
    fn serves(&self, spec: &PackageSpec) -> bool {
        self.name.as_str() == spec.name.as_str()
            && self
                .version
                .as_ref()
                .is_none_or(|version| version == &spec.version)
    }

    fn read(&self, id: FileId) -> FileResult<Bytes> {
        let path = id
            .vpath()
            .realize(&self.dir)
            .map_err(|_| FileError::AccessDenied)?;
        let data = std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?;
        Ok(Bytes::new(data))
    }
}

/// The version `dir`'s `typst.toml` declares, or `None` when the file cannot
/// be read or parsed.
fn manifest_version(dir: &Path) -> Option<PackageVersion> {
    let text = std::fs::read_to_string(dir.join("typst.toml")).ok()?;
    let manifest: PackageManifest = toml::from_str(&text).ok()?;
    Some(manifest.package.version)
}

/// The state's packages, then the pins, then the installed and embedded
/// packages. The fallback is handed no overrides: a pin that does not serve the
/// import's version must not capture it on the way to the package directory.
struct Pins {
    state: Option<StatePackages>,
    pins: Vec<Pin>,
    fallback: Box<dyn FileProvider>,
    no_overrides: HashMap<String, PathBuf>,
}

impl FileProvider for Pins {
    fn read(&self, id: FileId, ctx: &ProviderCtx<'_>) -> FileResult<Bytes> {
        if let VirtualRoot::Package(spec) = id.root() {
            if let Some(state) = &self.state
                && state.specs.contains(spec)
            {
                return state.files.read(id, ctx);
            }
            // Last entry wins, as the old name-keyed override map did.
            if let Some(pin) = self.pins.iter().rev().find(|pin| pin.serves(spec)) {
                return pin.read(id);
            }
        }
        self.fallback.read(
            id,
            &ProviderCtx {
                root: ctx.root,
                overrides: &self.no_overrides,
            },
        )
    }
}
