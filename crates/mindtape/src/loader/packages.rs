//! The `@local` package resolution every world Mindtape builds shares.

use std::path::{Path, PathBuf};

use typst_harvest::HarvestWorld;

use crate::config::Config;

use super::{StatePackages, package_root, pins};

/// `world` resolving `@local` packages the way every world Mindtape builds
/// does: under [`package_root`], with `overrides` (from [`package_overrides`])
/// pinning a package to an in-tree directory for the version its manifest
/// declares, and the installed or embedded copy where the pin does not hold
/// that version.
///
/// A pin is authoritative only for its own version. A task file importing
/// another version falls through to the package directory, so a commit range
/// across a prelude bump reads each tree's imports against the version that
/// tree names while the working tree keeps its pin.
#[must_use]
pub fn with_packages(world: HarvestWorld, overrides: &[(String, PathBuf)]) -> HarvestWorld {
    pins::with_packages_at(world, overrides, package_root())
}

/// [`with_packages`] with a commit state's own package files tried first, so
/// a past tree's task files resolve against that tree's `lib/` rather than the
/// working tree's pin or the machine's install.
#[must_use]
pub(crate) fn with_packages_and(
    world: HarvestWorld,
    overrides: &[(String, PathBuf)],
    state: StatePackages,
) -> HarvestWorld {
    pins::with_packages_and(world, overrides, package_root(), Some(state))
}

/// The `@local/<name>` → in-tree overrides for this project: its
/// `[[typst.local-packages]]` entries, and nothing else. The core names no
/// directory of its own, so Mindtape's own repository pins its `lib/` with an
/// entry like any other project. A pin serves only the version its
/// `typst.toml` declares; [`with_packages`] applies that rule.
///
/// Public because it is the project's single answer to "which `@local`
/// packages resolve here", and every world Mindtape builds, `mt eval`'s
/// included, has to give the same one.
pub fn package_overrides(root: &Path, config: &Config) -> Vec<(String, PathBuf)> {
    config
        .typst
        .local_packages
        .iter()
        .map(|pkg| (pkg.name.clone(), root.join(&pkg.path)))
        .collect()
}
