//! `mindtape cleanup` — retire terminal-status tasks and rewrite inbound links
//! into SWHID tombstones, atomically, in a single git commit.
//!
//! Thin orchestration over two consumer-blind crates: [`swhid_mint`] mints a
//! `swh:1:rev:<commit>;path=<path>` history pointer for each removed file (and
//! enforces the working-blob-is-recoverable invariant), and [`typst_edit`]
//! locates and splices the `link("…")` targets in surviving files.
//!
//! Atomic by construction: every mint and every file rewrite is computed in
//! memory first ([`plan`]); not a byte is written until the whole plan succeeds.
//! A wrong-shaped file or an unresolvable commit aborts the run with no changes.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;
use mindtape::{Task, is_history_pointer, normalize_link};

use crate::{git, util};

/// Arguments for `mindtape cleanup`.
#[derive(Args)]
pub struct CleanupArgs {
    /// Skip the interactive confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

/// A file the plan will delete: its absolute path and root-relative display path.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Deletion {
    abs: PathBuf,
    rel: PathBuf,
}

/// A surviving file the plan will rewrite, with its already-spliced contents.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rewrite {
    abs: PathBuf,
    rel: PathBuf,
    contents: String,
    /// How many links were retargeted in this file.
    links: usize,
}

/// The full, computed-in-memory cleanup. Empty when nothing is terminal.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Plan {
    deletions: Vec<Deletion>,
    rewrites: Vec<Rewrite>,
}

impl Plan {
    fn is_empty(&self) -> bool {
        self.deletions.is_empty()
    }
}

/// Run the `cleanup` command.
///
/// # Errors
/// Returns an error (non-zero exit) on a dirty working tree, a task file that
/// is not recoverable from git, a wrong-shaped surviving file, a declined
/// prompt, or any git failure.
pub fn run(args: &CleanupArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let root = project
        .root
        .canonicalize()
        .with_context(|| format!("resolve {}", project.root.display()))?;
    let git_root = git::toplevel(&root)?
        .canonicalize()
        .context("resolve git toplevel")?;

    if !git::is_clean(&root)? {
        bail!(
            "working tree has uncommitted changes; cleanup is destructive — commit or stash first"
        );
    }

    let plan = plan(
        &project.outcome.tasks,
        &project.config.tasks.status.terminal,
        &root,
        &git_root,
    )?;

    if plan.is_empty() {
        println!("nothing to clean up: no terminal-status tasks");
        return Ok(());
    }

    if !args.yes && !confirm(&plan)? {
        bail!("aborted by user");
    }

    apply(&plan)?;
    let commit = git::commit_all(&root, &commit_message(&plan))?;

    report(&plan, &commit);
    Ok(())
}

/// Compute the whole cleanup without touching disk.
///
/// Mints a tombstone for every terminal task (failing the run if any file is
/// untracked or its working bytes diverge from git) and splices every surviving
/// file's inbound links onto those tombstones. Returns an empty plan when no
/// task is terminal.
///
/// # Errors
/// Propagates mint failures, file reads, and [`typst_edit`] rejections.
fn plan(tasks: &[Task], terminal: &[String], root: &Path, git_root: &Path) -> Result<Plan> {
    let mut tombstones: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut deletions = Vec::new();

    let removed: Vec<&Task> = tasks.iter().filter(|t| t.is_terminal(terminal)).collect();
    if removed.is_empty() {
        return Ok(Plan::default());
    }

    for task in &removed {
        let abs = root.join(&task.file);
        let git_rel = abs
            .strip_prefix(git_root)
            .with_context(|| format!("{} is outside the git repository", abs.display()))?;
        let git_rel = git_rel.to_string_lossy();
        let id = swhid_mint::git::mint_rev_path(git_root, &git_rel)
            .with_context(|| format!("mint tombstone for {}", task.file.display()))?;
        tombstones.insert(task.id.clone(), format!("\"{}\"", id.render()));
        deletions.push(Deletion {
            abs,
            rel: task.file.clone(),
        });
    }

    let surviving = tasks.iter().filter(|t| !t.is_terminal(terminal));
    let mut rewrites = Vec::new();
    for task in surviving {
        if let Some(rewrite) = plan_rewrite(task, root, &tombstones)? {
            rewrites.push(rewrite);
        }
    }

    deletions.sort_by(|a, b| a.rel.cmp(&b.rel));
    rewrites.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(Plan {
        deletions,
        rewrites,
    })
}

/// Build the rewrite for one surviving file, or `None` if it links to no removed
/// task. Reads the source, locates every `link("…")`, and retargets those whose
/// stem names a removed task — leaving existing history pointers untouched.
fn plan_rewrite(
    task: &Task,
    root: &Path,
    tombstones: &std::collections::HashMap<String, String>,
) -> Result<Option<Rewrite>> {
    let abs = root.join(&task.file);
    let source =
        std::fs::read_to_string(&abs).with_context(|| format!("read {}", abs.display()))?;

    let mut edits = Vec::new();
    for target in typst_edit::find_link_targets(&source) {
        if is_history_pointer(&target.url) {
            continue;
        }
        if let Some(tombstone) = tombstones.get(&normalize_link(&target.url)) {
            edits.push(typst_edit::Edit::new(
                target.range.clone(),
                tombstone.clone(),
            ));
        }
    }
    if edits.is_empty() {
        return Ok(None);
    }

    let links = edits.len();
    let contents = typst_edit::apply(&source, edits)
        .with_context(|| format!("rewrite links in {}", task.file.display()))?;
    Ok(Some(Rewrite {
        abs,
        rel: task.file.clone(),
        contents,
        links,
    }))
}

/// Write every rewrite, then delete every removed file. Called only after a
/// complete, validated [`Plan`].
fn apply(plan: &Plan) -> Result<()> {
    for rewrite in &plan.rewrites {
        std::fs::write(&rewrite.abs, &rewrite.contents)
            .with_context(|| format!("write {}", rewrite.rel.display()))?;
    }
    for deletion in &plan.deletions {
        std::fs::remove_file(&deletion.abs)
            .with_context(|| format!("delete {}", deletion.rel.display()))?;
    }
    Ok(())
}

fn commit_message(plan: &Plan) -> String {
    let mut body = String::from("chore(tasks): cleanup retire terminal tasks\n\nDeletes:\n");
    for deletion in &plan.deletions {
        body.push_str(&format!("  - {}\n", deletion.rel.display()));
    }
    if !plan.rewrites.is_empty() {
        body.push_str("\nRewrote inbound links in:\n");
        for rewrite in &plan.rewrites {
            body.push_str(&format!("  - {}\n", rewrite.rel.display()));
        }
    }
    body
}

fn confirm(plan: &Plan) -> Result<bool> {
    use std::io::Write;
    let mut stderr = std::io::stderr();
    writeln!(
        stderr,
        "cleanup will delete {} task(s) and rewrite links in {} file(s):",
        plan.deletions.len(),
        plan.rewrites.len()
    )?;
    for deletion in &plan.deletions {
        writeln!(stderr, "  - {}", deletion.rel.display())?;
    }
    write!(stderr, "Continue? [y/N] ")?;
    stderr.flush()?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    let answer = line.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn report(plan: &Plan, commit: &str) {
    println!("deleted {} task(s):", plan.deletions.len());
    for deletion in &plan.deletions {
        println!("  - {}", deletion.rel.display());
    }
    if !plan.rewrites.is_empty() {
        let links: usize = plan.rewrites.iter().map(|r| r.links).sum();
        println!("rewrote {links} link(s) in {} file(s)", plan.rewrites.len());
    }
    println!("committed {commit}");
}

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;
