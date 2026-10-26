//! `mt packs update`: lock each source at the commit its `ref` points to.
//!
//! Every fetch runs before the lock is written, and the lock is written once
//! and atomically, so a failure leaves the old lock whole. The report and
//! its warnings wait for that write too, so a failed update prints only its
//! error, not progress on a lock it never wrote.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mindtape::config::pack_sources::{self, LockedSource, PackLock, PackSource, lock_path};

use super::report::{self, Report};
use super::{cache_root, short};
use crate::diag::{Diag, Hint, Result};
use crate::util;

/// Run `mt packs update`, on the sources `names` or on every source.
pub(super) fn run(names: &[String]) -> Result<()> {
    let root = util::resolve_root()?;
    let raw = pack_sources::read_raw(&root)?;
    let picked = pick(&raw.sources, names)?;
    let mut update = Update {
        root: &root,
        cache: cache_root()?,
        report: Report::default(),
    };
    let mut lock = raw.lock.clone();
    let mut changed = update.drop_undeclared(&mut lock, &raw.sources);
    for (name, source) in picked {
        let entry = lock.entries.get(name);
        if let Some(next) = update.relock(name, source, entry)? {
            lock.entries.insert(name.to_owned(), next);
            changed = true;
        }
    }
    if changed {
        pack_sources::write_lock(&root, &lock)
            .map_err(|error| Diag::cannot_write(&lock_path(&root), &error))?;
        let shown = mindtape::slash_path(&lock_path(Path::new("")));
        update.report.line(&format!("wrote {shown}"));
    }
    update.report.print();
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

/// One `mt packs update` run: where it reads and writes, and the report it
/// prints once the lock is written.
struct Update<'a> {
    root: &'a Path,
    cache: PathBuf,
    report: Report,
}

impl Update<'_> {
    /// Drop each lock entry no source declares, saying so. Whether any was.
    fn drop_undeclared(
        &mut self,
        lock: &mut PackLock,
        sources: &BTreeMap<String, PackSource>,
    ) -> bool {
        let before = lock.entries.len();
        let report = &mut self.report;
        lock.entries.retain(|name, _| {
            let keep = sources.contains_key(name);
            if !keep {
                report.line(&format!("dropped {name}"));
            }
            keep
        });
        lock.entries.len() != before
    }

    /// The new lock entry of `name`, fetched into the cache, or `None` when
    /// `entry` already pins the commit its `ref` names.
    fn relock(
        &mut self,
        name: &str,
        source: &PackSource,
        entry: Option<&LockedSource>,
    ) -> Result<Option<LockedSource>> {
        let named = |error| Diag::from(error).value(name);
        let git = pack_sources::locate(&source.git, self.root).map_err(named)?;
        let commit = pack_sources::resolve(&git, &source.reference, &self.cache).map_err(named)?;
        let label = format!("{name} {}", source.reference);
        let pinned = entry.filter(|old| old.git == source.git && old.reference == source.reference);
        if pinned.is_some_and(|old| old.commit == commit) {
            self.report.line(&format!("{label}: up to date"));
            return Ok(None);
        }
        let sha256 = pack_sources::fetch(&git, &source.reference, &commit, None, &self.cache)
            .map_err(named)?;
        self.report.line(&match entry {
            Some(old) => format!("{label}: {} → {}", short(&old.commit), short(&commit)),
            None => format!("{label}: locked {}", short(&commit)),
        });
        let moved = report::Moved {
            name,
            old: entry,
            git: &git,
            tree: &self.cache.join(&sha256),
        };
        moved.report(self.root, &self.cache, &mut self.report);
        Ok(Some(LockedSource {
            git: source.git.clone(),
            reference: source.reference.clone(),
            commit,
            sha256,
        }))
    }
}
