//! `mt packs update`: lock each source at the commit its `ref` points to.
//!
//! Every fetch runs before the lock is written, and the lock is written once
//! and atomically, so a failure leaves the old lock whole.

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::config::pack_sources::{self, LockedSource, PackLock, PackSource, lock_path};

use super::{cache_root, short};
use crate::diag::{Diag, Hint, Result};
use crate::util;

/// Run `mt packs update`, on the sources `names` or on every source.
pub(super) fn run(names: &[String]) -> Result<()> {
    let root = util::resolve_root()?;
    let raw = pack_sources::read_raw(&root)?;
    let picked = pick(&raw.sources, names)?;
    let cache = cache_root()?;
    let mut lock = raw.lock.clone();
    let mut changed = drop_undeclared(&mut lock, &raw.sources);
    for (name, source) in picked {
        let entry = lock.entries.get(name);
        if let Some(next) = relock(&root, name, source, entry, &cache)? {
            lock.entries.insert(name.to_owned(), next);
            changed = true;
        }
    }
    if changed {
        pack_sources::write_lock(&root, &lock)
            .map_err(|error| Diag::cannot_write(&lock_path(&root), &error))?;
        println!("wrote {}", mindtape::slash_path(&lock_path(Path::new(""))));
    }
    Ok(())
}

/// The sources `names` asks for, every source when it names none.
fn pick<'a>(
    sources: &'a BTreeMap<String, PackSource>,
    names: &[String],
) -> Result<Vec<(&'a str, &'a PackSource)>> {
    if names.is_empty() {
        return Ok(sources
            .iter()
            .map(|(name, source)| (name.as_str(), source))
            .collect());
    }
    let known: Vec<&str> = sources.keys().map(String::as_str).collect();
    names
        .iter()
        .map(|name| {
            sources
                .get_key_value(name)
                .map(|(name, source)| (name.as_str(), source))
                .ok_or_else(|| {
                    Diag::usage("unknown pack source")
                        .value(name)
                        .did_you_mean_or(
                            name,
                            &known,
                            Hint::new("declare it under [pack-sources] in .mindtape/config.toml"),
                        )
                })
        })
        .collect()
}

/// Drop each lock entry no source declares, saying so. Whether any was.
fn drop_undeclared(lock: &mut PackLock, sources: &BTreeMap<String, PackSource>) -> bool {
    let before = lock.entries.len();
    lock.entries.retain(|name, _| {
        let keep = sources.contains_key(name);
        if !keep {
            println!("dropped {name}");
        }
        keep
    });
    lock.entries.len() != before
}

/// The new lock entry of `name`, fetched into `cache`, or `None` when
/// `entry` already pins the commit its `ref` names.
fn relock(
    root: &Path,
    name: &str,
    source: &PackSource,
    entry: Option<&LockedSource>,
    cache: &Path,
) -> Result<Option<LockedSource>> {
    let git = pack_sources::locate(&source.git, root);
    let named = |error| Diag::from(error).value(name);
    let commit = pack_sources::resolve(&git, &source.reference).map_err(named)?;
    let label = format!("{name} {}", source.reference);
    let pinned = entry.filter(|old| old.git == source.git && old.reference == source.reference);
    if pinned.is_some_and(|old| old.commit == commit) {
        println!("{label}: up to date");
        return Ok(None);
    }
    let sha256 =
        pack_sources::fetch(&git, &source.reference, &commit, None, cache).map_err(named)?;
    match entry {
        Some(old) => println!("{label}: {} → {}", short(&old.commit), short(&commit)),
        None => println!("{label}: locked {}", short(&commit)),
    }
    Ok(Some(LockedSource {
        git: source.git.clone(),
        reference: source.reference.clone(),
        commit,
        sha256,
    }))
}
