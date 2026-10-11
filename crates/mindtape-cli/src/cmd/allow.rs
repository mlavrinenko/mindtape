//! `mt allow`: let the commands the project's config runs, its
//! `[[tasks.waits]]` checkers and `[[on.flip]]` `run` actions, run in this
//! repository. See [`mindtape::allow`].

use crate::diag::Result;

use crate::util;

/// Arguments for `allow`.
#[derive(Debug, clap::Args)]
pub struct AllowArgs {}

/// Run the `allow` command: print every command the config runs, then record
/// them as allowed for this repository and each of its worktrees.
///
/// It reads the config only. Loading the tasks would ask the very checkers it
/// is about to allow.
///
/// # Errors
/// Returns an error if the project or its config cannot be read, or the
/// approval cannot be written.
pub fn run(_args: &AllowArgs) -> Result<()> {
    let root = util::resolve_root()?;
    let config = mindtape::load_config(&root)?;
    let commands = mindtape::allow::commands(&config);
    if commands.is_empty() {
        println!("the config runs no commands, so there is nothing to allow");
        return Ok(());
    }
    mindtape::allow::record(&root, &commands)?;
    println!("allowed in this repository and its worktrees:");
    for command in &commands {
        println!("  {command}");
    }
    Ok(())
}
