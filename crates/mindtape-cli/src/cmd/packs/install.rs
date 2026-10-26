//! `mt packs install`: fetch what the lock pins and the cache lacks.

use std::path::Path;

use mindtape::config::pack_sources::{self, LockedSource, PackSources};

use super::{cache_root, short};
use crate::diag::{Diag, Result};
use crate::util;

/// One declared source after `install`.
struct Installed<'a> {
    source: &'a str,
    commit: &'a str,
    sha256: &'a str,
    fetched: bool,
}

/// `mt packs install`: fetch every declared source whose locked tree the
/// cache lacks, then report each source.
pub(super) fn run(json: bool) -> Result<()> {
    let root = util::resolve_root()?;
    let sources = pack_sources::read_raw(&root)?;
    sources.check_fresh()?;
    let cache = cache_root()?;
    let mut report = Vec::new();
    for (name, locked) in locked(&sources) {
        let tree = cache.join(&locked.sha256);
        let fetched = !tree.is_dir();
        if fetched {
            fetch(&root, name, locked, &cache)?;
            if !json {
                println!(
                    "fetched {name} {} into {}",
                    short(&locked.commit),
                    tree.display()
                );
            }
        }
        report.push(Installed {
            source: name,
            commit: &locked.commit,
            sha256: &locked.sha256,
            fetched,
        });
    }
    if json {
        let rows: Vec<serde_json::Value> = report.iter().map(Installed::json).collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    let cached = report.iter().filter(|source| !source.fetched).count();
    if cached > 0 || report.is_empty() {
        println!("{} up to date", mindtape::count(cached, "source"));
    }
    Ok(())
}

impl Installed<'_> {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "source": self.source,
            "commit": self.commit,
            "sha256": self.sha256,
            "fetched": self.fetched,
        })
    }
}

/// Each declared source with its lock entry, by name. A lock entry no source
/// declares is left out.
fn locked(sources: &PackSources) -> impl Iterator<Item = (&str, &LockedSource)> {
    sources.sources.keys().filter_map(|name| {
        let locked = sources.lock.entries.get(name)?;
        Some((name.as_str(), locked))
    })
}

/// Fetch `name` as `locked` pins it into `cache`, its tree checked against
/// the lock's hash.
fn fetch(root: &Path, name: &str, locked: &LockedSource, cache: &Path) -> Result<()> {
    let named = |error| Diag::from(error).value(name);
    let git = pack_sources::locate(&locked.git, root).map_err(named)?;
    pack_sources::fetch(
        &git,
        &locked.reference,
        &locked.commit,
        Some(&locked.sha256),
        cache,
    )
    .map_err(named)?;
    Ok(())
}
