//! `mt packs`: the pack sources a project declares in `[pack-sources]` and
//! pins in `.mindtape/packs.lock`.
//!
//! Each subcommand reads the two files with `pack_sources::read_raw`, not a
//! config load, because a load refuses the very state they repair. `install`
//! writes only the cache, so it is a read command. `update` writes the lock.
//!
//! # Files
//!
//! - `mod.rs`: `mt packs` and what both subcommands share.
//! - `install.rs`: `mt packs install`.
//! - `update.rs`: `mt packs update`.

mod install;
mod update;

use std::path::PathBuf;

use mindtape::config::pack_sources;

use crate::diag::{Diag, Hint, Result};

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
    /// Lock each source at the commit its `ref` points to.
    ///
    /// It fetches each source whose commit moved, rewrites its lock entry,
    /// and drops the entries no source declares.
    Update {
        /// The sources to update. Without one, it updates them all.
        sources: Vec<String>,
    },
}

/// Run the `packs` command.
///
/// # Errors
/// Returns an error if no project is found, its sources or lock can't be
/// read or disagree, no cache directory applies, a `ref` can't be resolved,
/// a fetch fails, or the lock can't be written.
pub fn run(args: &PacksArgs) -> Result<()> {
    match &args.action {
        PacksAction::Install { json } => install::run(*json),
        PacksAction::Update { sources } => update::run(sources),
    }
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
