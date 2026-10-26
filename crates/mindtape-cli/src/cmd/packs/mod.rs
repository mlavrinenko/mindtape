//! `mt packs`: the pack sources a project declares in `[pack-sources]` and
//! pins in `.mindtape/packs.lock`.
//!
//! Each subcommand reads the two files with `pack_sources::read_raw`, not a
//! config load, because a load refuses the very state they repair. `install`
//! writes only the cache, so it is a read command.
//!
//! # Files
//!
//! - `mod.rs`: `mt packs` and `install`.

use std::path::{Path, PathBuf};

use mindtape::config::pack_sources::{self, LockedSource, PackSources};

use crate::diag::{Diag, Hint, Result};
use crate::util;

/// Arguments for `packs`.
#[derive(Debug, clap::Args)]
pub struct PacksArgs {
    #[command(subcommand)]
    action: PacksAction,
}

/// `mt packs` sub-actions.
#[derive(Debug, clap::Subcommand)]
enum PacksAction {
    /// Fetch each source the lock pins that the cache lacks.
    ///
    /// It refuses a lock that disagrees with `[pack-sources]`, as `npm ci`
    /// does, so run it in CI or after a clone.
    Install {
        /// Print JSON: each source's name, commit, sha256 and whether it
        /// was fetched.
        #[arg(long)]
        json: bool,
    },
}

/// One declared source after `install`.
struct Installed<'a> {
    source: &'a str,
    commit: &'a str,
    sha256: &'a str,
    fetched: bool,
}

/// Run the `packs` command.
///
/// # Errors
/// Returns an error if no project is found, its sources or lock can't be
/// read or disagree, no cache directory applies, or a fetch fails.
pub fn run(args: &PacksArgs) -> Result<()> {
    match &args.action {
        PacksAction::Install { json } => install(*json),
    }
}

/// `mt packs install`: fetch every declared source whose locked tree the
/// cache lacks, then report each source.
fn install(json: bool) -> Result<()> {
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
    let git = pack_sources::locate(&locked.git, root);
    pack_sources::fetch(
        &git,
        &locked.reference,
        &locked.commit,
        Some(&locked.sha256),
        cache,
    )
    .map_err(|error| Diag::from(error).value(name))?;
    Ok(())
}

/// The directory the config load reads each source's tree from.
fn cache_root() -> Result<PathBuf> {
    pack_sources::cache_root().ok_or_else(|| {
        Diag::failed("can't find the cache directory")
            .hint(Hint::new("set the directory in MINDTAPE_CACHE_HOME"))
    })
}

/// The first seven digits of `commit`, as git abbreviates it.
fn short(commit: &str) -> &str {
    commit.get(..7).unwrap_or(commit)
}
