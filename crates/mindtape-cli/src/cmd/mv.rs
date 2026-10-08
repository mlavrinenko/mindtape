//! `mt mv <task> <new-stem>` — rename a task and rewrite its inbound links.
//!
//! A task's identity is its filename stem, so a stem written before the task
//! settled names the wrong thing for as long as the file lives. Renaming it by
//! hand means a `git mv` plus an edit of every `links:` target that points at
//! it, with nothing checking the result until `mt check` reports a dangling
//! edge — one missed target and the graph quietly loses one.
//!
//! So this command owns both halves as one move: the file changes name, and
//! every inbound edge is retargeted onto the new stem through the walk `mt rm`
//! already owns ([`crate::cmd::rm::inbound_rewrites`]) — `rm` points those edges at
//! nothing, a rename points them at the new stem, and removal is the null
//! target of the same walk.
//!
//! What it deliberately does not touch: the `title:`, which is independent of
//! the stem by design and belongs to its own edit, and the file's folder, since
//! moving a task between roots or folders changes its derived tags and is a
//! decision of its own — a stem carrying a separator is refused rather than
//! guessed at.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use clap::Args;
use typst_cleanup::Rewrite;

use crate::cmd::rm::{Retarget, inbound_rewrites, print_commit_hint, retarget_raw};
use crate::diag::{Diag, Hint, Note};
use crate::pre_write::{self, Pending};
use crate::util::Project;
use crate::{git, util};

/// Arguments for `mt mv`.
#[derive(Args)]
pub struct MvArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,
    /// The new stem: a file name without `.typ` and without a folder.
    pub new_stem: String,
    /// Show the rename and every file whose links would change, without
    /// changing anything.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `mv` command.
///
/// Refuses a taken stem before anything moves, plans the inbound rewrites in
/// memory, validates the whole set through [`crate::pre_write`], and only then
/// moves the file and writes the rewrites. Like `mt rm`, it does not commit —
/// the user reviews the diff.
///
/// # Errors
/// An unresolvable task, a stem that is empty, carries a path separator, is
/// already the task's own, or is taken by another task or an existing file; a
/// referring file that cannot be read or spliced; a rewrite the project would
/// refuse; or a failed move or write.
pub fn run(args: &MvArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);

    let (task, _) = util::resolve_task(&project, &args.task.task)?;
    let stem = new_stem(&args.new_stem)?;
    if stem == task.id {
        return Err(Diag::refused("task is already named")
            .value(stem)
            .hint(Hint::new("pass a different stem"))
            .into());
    }

    // Root-relative throughout: `resolve_task` canonicalizes a path argument,
    // which may not share a prefix with the project root through a symlink, and
    // a destination outside that root is a write `pre_write` cannot check.
    let from_rel = task.file.clone();
    let to_rel = renamed(&from_rel, stem);
    let from = project.root.join(&from_rel);
    let to = project.root.join(&to_rel);
    refuse_taken(&project, stem, &to)?;

    let moved: HashSet<String> = std::iter::once(task.id.clone()).collect();
    let rewrites = inbound_rewrites(
        &project.outcome.tasks,
        &moved,
        &project.root,
        Retarget::Stem(stem),
    )?;

    if args.dry_run {
        report_plan(&from_rel, &to_rel, &rewrites);
        return Ok(());
    }

    let source =
        std::fs::read_to_string(&from).map_err(|err| Diag::cannot_read(&from_rel, &err))?;
    verify(&project, &to, &source, &rewrites)?;

    move_file(&project.root, &from, &to)?;
    for rewrite in &rewrites {
        util::write_task(&project.config, &rewrite.abs, rewrite.contents.clone())?;
    }
    report_applied(&from_rel, &to_rel, &rewrites, &project.root);
    Ok(())
}

/// The stem the user asked for, as a stem: a trailing `.typ` is the natural
/// slip when the argument is a filename and is dropped rather than turned into
/// `new.typ.typ`.
///
/// # Errors
/// An empty stem, or one carrying a path separator — `mv` renames in place.
fn new_stem(raw: &str) -> Result<&str> {
    let stem = raw.strip_suffix(".typ").unwrap_or(raw);
    if stem.is_empty() {
        return Err(Diag::usage("the new stem is empty")
            .hint(Hint::new("pass a stem such as new-name"))
            .into());
    }
    if stem.contains('/') || stem.contains('\\') {
        return Err(Diag::usage("the new stem has a path separator")
            .value(stem)
            .hint(Hint::new("mt mv renames a task where it stands"))
            .hint(Hint::new(
                "moving a task between roots or folders changes its derived tags, so it is a decision of its own",
            ))
            .into());
    }
    Ok(stem)
}

/// `file` with its stem replaced, extension and folder kept — the same rename
/// the inbound edge targets get, so the file and the edges that name it cannot
/// disagree about what the new name is.
fn renamed(file: &Path, stem: &str) -> PathBuf {
    let name = file
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    file.with_file_name(retarget_raw(&name, stem))
}

/// Refuse a stem another task already answers to, or a destination file that
/// exists — before anything has moved, so a refused rename leaves the tree
/// exactly as it was.
///
/// Both checks, not one: a loaded task proves the identity is taken, and the
/// bare path catches a file the project does not load (an unparseable one, or a
/// stem outside the configured roots) that a rename would silently clobber.
///
/// # Errors
/// Names the file already holding the stem.
fn refuse_taken(project: &Project, stem: &str, to: &Path) -> Result<()> {
    if let Some(taken) = project.outcome.tasks.iter().find(|task| task.id == stem) {
        return Err(Diag::refused("stem is taken")
            .value(stem)
            .note(Note::new("held by").name(taken.file.display().to_string()))
            .hint(pick_another())
            .into());
    }
    if to.exists() {
        return Err(Diag::refused("file already exists")
            .value(to.display().to_string())
            .hint(pick_another())
            .into());
    }
    Ok(())
}

/// Ask the project whether the whole move may land: the task at its new path,
/// and every retargeted referrer, as one batch — a referrer's new edge only
/// resolves *given* the renamed file, so they cannot be checked apart.
///
/// # Errors
/// Names each problem and refuses the batch; nothing has been written when it
/// does.
fn verify(project: &Project, to: &Path, source: &str, rewrites: &[Rewrite]) -> Result<()> {
    let mut pending = vec![Pending { file: to, source }];
    pending.extend(rewrites.iter().map(|rewrite| Pending {
        file: &rewrite.abs,
        source: &rewrite.contents,
    }));
    pre_write::ensure(project, &pending)
}

/// Move the task file: `git mv` inside a repository, so the rename lands staged
/// and git carries the file's history across it, and a plain rename outside one
/// — the same split `mt rm` makes between its git and gitless paths.
///
/// A failed `git mv` falls through to the plain rename rather than failing the
/// run: git refuses a path it does not track, which is every task filed since
/// the last commit, and a rename it cannot stage is still a rename the user
/// asked for.
///
/// # Errors
/// A filesystem rename that fails.
fn move_file(root: &Path, from: &Path, to: &Path) -> Result<()> {
    if git::toplevel(root).is_ok() && git_mv(root, from, to) {
        return Ok(());
    }
    std::fs::rename(from, to).map_err(|err| {
        Diag::failed("can't move")
            .value(from.display().to_string())
            .context("to", to.display().to_string())
            .io(&err)
            .into()
    })
}

fn pick_another() -> Hint {
    Hint::new("pick another stem")
}

/// `git mv` in `root`, reporting only whether git took the rename. Output is
/// captured rather than inherited: git's own wording about a path it does not
/// track is not this command's report.
fn git_mv(root: &Path, from: &Path, to: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("mv")
        .arg(from)
        .arg(to)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Print what the run would do (`--dry-run`): the move, and every file whose
/// edges change.
fn report_plan(from: &Path, to: &Path, rewrites: &[Rewrite]) {
    println!("would move {} -> {}", from.display(), to.display());
    report_rewrites("would rewrite", rewrites, true);
    println!("dry run: nothing written");
}

/// Print what the run just did, and — inside a repository, where there is
/// something to commit — remind the user to review it.
fn report_applied(from: &Path, to: &Path, rewrites: &[Rewrite], root: &Path) {
    println!("moved {} -> {}", from.display(), to.display());
    report_rewrites("rewrote", rewrites, false);
    print_commit_hint(root);
}

/// `<verb> N link(s) in M file(s)`, the files listed only when previewing:
/// a plan is read to decide, and a finished report to confirm.
fn report_rewrites(verb: &str, rewrites: &[Rewrite], list: bool) {
    if rewrites.is_empty() {
        return;
    }
    let links: usize = rewrites.iter().map(|rewrite| rewrite.links).sum();
    let tail = if list { ":" } else { "" };
    println!("{verb} {links} link(s) in {} file(s){tail}", rewrites.len());
    if list {
        for rewrite in rewrites {
            println!("  - {}", rewrite.rel.display());
        }
    }
}

#[cfg(test)]
#[path = "mv_tests.rs"]
mod tests;
