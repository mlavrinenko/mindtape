//! `mt mv <task> <new-stem>` renames a task and rewrites its inbound links.
//!
//! A task's identity is its filename stem. A stem written before the task
//! settled specifies the wrong thing for as long as the file lives. Renaming it
//! by hand means a `git mv` plus an edit of every `links:` target that points at
//! it. The result goes unchecked until `mt check` reports a dangling edge, and
//! one missed target loses one edge from the graph.
//!
//! This command owns both halves as one move. The file changes name, and every
//! inbound edge moves onto the new stem through the walk `mt rm` already owns
//! ([`crate::cmd::rm::inbound_rewrites`]). `rm` points those edges at nothing, a
//! rename points them at the new stem, and removal is the null target of the
//! same walk.
//!
//! It deliberately leaves two things alone. The `title:` is independent of the
//! stem by design and belongs to its own edit. The file's folder stays put,
//! because moving a task between roots or folders changes its derived tags and
//! is a decision of its own. `mt mv` refuses a stem that carries a separator
//! rather than guessing at it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::diag::Result;
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
    /// The new stem, given as a file name without `.typ` and without a folder.
    pub new_stem: String,
    /// Show what would change, without changing anything.
    ///
    /// It shows the rename and every file whose links would change.

    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `mv` command.
///
/// Refuses a taken stem before anything moves, plans the inbound rewrites in
/// memory, and validates the whole set through [`crate::pre_write`]. Only then
/// does it move the file and write the rewrites. Like `mt rm`, it does not
/// commit. The user reviews the diff.
///
/// # Errors
/// An unresolvable task. A stem that is empty, carries a path separator, or
/// equals the task's own. A stem that another task or an existing file already
/// holds. A referring file that the command cannot read or splice. A rewrite
/// the project would refuse. A failed move or write.
pub fn run(args: &MvArgs) -> Result<()> {
    let project = util::open_warned()?;

    let (task, _) = util::resolve_task(&project, &args.task.task)?;
    let stem = new_stem(&args.new_stem)?;
    if stem == task.id {
        return Err(Diag::refused("task is already named")
            .value(stem)
            .hint(Hint::new("pass a different stem")));
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

/// The stem the user asked for, as a stem. A trailing `.typ` is the natural
/// slip when the argument is a filename. The function drops it rather than
/// turning it into `new.typ.typ`.
///
/// # Errors
/// An empty stem, or one carrying a path separator. `mv` renames in place.
fn new_stem(raw: &str) -> Result<&str> {
    let stem = raw.strip_suffix(".typ").unwrap_or(raw);
    if stem.is_empty() {
        return Err(
            Diag::usage("the new stem is empty").hint(Hint::new("pass a stem such as new-name"))
        );
    }
    if stem.contains('/') || stem.contains('\\') {
        return Err(Diag::usage("the new stem has a path separator")
            .value(stem)
            .hint(Hint::new("mt mv renames a task where it stands"))
            .hint(Hint::new(
                "moving a task between roots or folders changes its derived tags, so it is a decision of its own",
            )));
    }
    Ok(stem)
}

/// `file` with its stem replaced, and its extension and folder kept. This is
/// the same rename the inbound edge targets get, which keeps the file and the
/// edges that name it in agreement about the new name.
fn renamed(file: &Path, stem: &str) -> PathBuf {
    let name = file
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
    file.with_file_name(retarget_raw(&name, stem))
}

/// Refuse a stem another task already holds, or a destination file that
/// exists. Both checks run before anything moves. A refusal leaves the tree
/// exactly as it was.
///
/// The two checks cover different cases. A loaded task proves another task
/// holds the identity. The bare path catches a file the project does not load
/// (an unparseable one, or a stem outside the configured roots) that a rename
/// would silently clobber.
///
/// # Errors
/// Names the file already holding the stem.
fn refuse_taken(project: &Project, stem: &str, to: &Path) -> Result<()> {
    if let Some(taken) = project.outcome.tasks.iter().find(|task| task.id == stem) {
        return Err(Diag::refused("stem is taken")
            .value(stem)
            .note(Note::new("held by").name(mindtape::display_path(&taken.file)))
            .hint(pick_another()));
    }
    if to.exists() {
        return Err(Diag::refused("file already exists")
            .value(mindtape::display_path(to))
            .hint(pick_another()));
    }
    Ok(())
}

/// Ask the project whether the whole move may land: the task at its new path,
/// and every retargeted referrer, as one batch. A referrer's new edge only
/// resolves *given* the renamed file. The two checks go through together.
///
/// # Errors
/// Names each problem and refuses the batch. A refusal leaves every file
/// untouched.
fn verify(project: &Project, to: &Path, source: &str, rewrites: &[Rewrite]) -> Result<()> {
    let mut pending = vec![Pending { file: to, source }];
    pending.extend(rewrites.iter().map(|rewrite| Pending {
        file: &rewrite.abs,
        source: &rewrite.contents,
    }));
    pre_write::ensure(project, &pending)
}

/// Move the task file. Inside a repository it runs `git mv`, which lands the
/// rename staged and lets git carry the file's history across it. Outside one
/// it runs a plain rename, the same split `mt rm` makes between its git and
/// gitless paths.
///
/// A failed `git mv` falls through to the plain rename rather than failing the
/// run. Git refuses a path it does not track, which is every task filed since
/// the last commit. A rename it cannot stage is still a rename the user asked
/// for.
///
/// # Errors
/// A filesystem rename that fails.
fn move_file(root: &Path, from: &Path, to: &Path) -> Result<()> {
    if git::toplevel(root).is_ok() && git_mv(root, from, to) {
        return Ok(());
    }
    std::fs::rename(from, to).map_err(|err| {
        Diag::failed("can't move")
            .value(mindtape::display_path(from))
            .context("to", mindtape::display_path(to))
            .io(&err)
    })
}

fn pick_another() -> Hint {
    Hint::new("pick another stem")
}

/// `git mv` in `root`, reporting only whether git took the rename. The function
/// captures the output rather than inheriting it, because git's own wording
/// about a path it does not track is not this command's report.
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
    println!(
        "would move {} -> {}",
        mindtape::display_path(from),
        mindtape::display_path(to)
    );
    report_rewrites("would rewrite", rewrites, true);
    println!("dry run: nothing written");
}

/// Print what the run just did. Inside a repository, where there is something
/// to commit, remind the user to review it.
fn report_applied(from: &Path, to: &Path, rewrites: &[Rewrite], root: &Path) {
    println!(
        "moved {} -> {}",
        mindtape::display_path(from),
        mindtape::display_path(to)
    );
    report_rewrites("rewrote", rewrites, false);
    print_commit_hint(root);
}

/// `<verb> N links in M files`, listing the files only when previewing. A plan
/// is for deciding, and a finished report is for confirming.
fn report_rewrites(verb: &str, rewrites: &[Rewrite], list: bool) {
    if rewrites.is_empty() {
        return;
    }
    let links: usize = rewrites.iter().map(|rewrite| rewrite.links).sum();
    let tail = if list { ":" } else { "" };
    println!(
        "{verb} {} in {}{tail}",
        mindtape::count(links, "link"),
        mindtape::count(rewrites.len(), "file")
    );
    if list {
        for rewrite in rewrites {
            println!("  - {}", mindtape::display_path(&rewrite.rel));
        }
    }
}

#[cfg(test)]
#[path = "mv_tests.rs"]
mod tests;
