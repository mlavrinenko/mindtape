//! `mt link` — the write path for `links:`, the one task field that had none.
//!
//! Two forms over one pipeline. `mt link <task> <kind> <target…>` upserts a
//! single edge; `mt link --apply <file>` reads an adjacency map (stem to kind
//! to targets, YAML or JSON, `-` for stdin) and merges every entry in it. Both
//! desugar to the same list of [`input::Entry`], so the checks, the
//! rendering, the dedupe and the report are written once.
//!
//! Batch semantics mirror `mt rm` and `mt flip`: the whole plan is computed in
//! memory before a byte is written ([`cmd::link::plan::build`]), so `--dry-run`
//! reports exactly what a real run would do; then the valid entries land and a
//! per-entry failure is reported without rolling the others back.
//!
//! Merging is by `(kind, target-stem)`, so running the same command twice
//! writes nothing the second time — the property that makes an adjacency map
//! safe to re-apply as it grows, and the reason the regex one-shot this
//! command replaces was a one-shot.
//!
//! # Files
//!
//! - `edit.rs` — renders edges as prelude helper calls and reconciles them.
//! - `input.rs` — the `--apply` adjacency map, read as YAML.
//! - `plan.rs` — resolves every entry and checks every edge before a write.

pub(crate) mod edit;
pub(crate) mod input;
pub(crate) mod plan;

use anyhow::Result;
use clap::Args;

use self::input::Entry;
use self::plan::{Plan, Staged};
use crate::cmd::flip::batch::{Landed, after_batch_diags};
use crate::cmd::link::edit::{EdgeSpec, Mode};
use crate::diag::{Command, Diag, Hint, emit};
use crate::pre_write::{self, Pending};
use crate::util::{self, Project};

/// Arguments for `mt link`.
#[derive(Debug, Args)]
pub struct LinkArgs {
    #[arg(help = crate::cli::task_ref::HELP, required_unless_present = "apply", conflicts_with = "apply")]
    pub task: Option<String>,

    /// The kind of link: `depends-on`, `parent`, `recurred-from`, `supersedes`,
    /// `duplicates`, `related`, or one your project adds.
    #[arg(required_unless_present = "apply")]
    pub kind: Option<String>,

    /// The tasks to link to, each by stem or by a path relative to the task. A
    /// stem that tasks in two folders share is refused, so give a path then.
    #[arg(required_unless_present = "apply")]
    pub targets: Vec<String>,

    /// A note on the link. A `related` link needs one, unless your project
    /// changes that. With `--apply`, put notes in the file instead.
    //
    // Refused with `--apply` rather than ignored, the way an unknown `--set`
    // key is: a map carries a note per target (`note:` beside `target:`), so
    // one flag could not say which of them it meant.
    #[arg(
        short = 'm',
        long = "note",
        value_name = "NOTE",
        conflicts_with = "apply"
    )]
    pub note: Option<String>,

    /// Read many links from a YAML or JSON file, or `-` for stdin, instead of
    /// naming one task. The file maps each task's stem to kinds, and each kind
    /// to targets.
    #[arg(long, value_name = "FILE")]
    pub apply: Option<String>,

    /// With `--apply`, make the file the full set of links for each task it
    /// names, removing links it leaves out. Without `--replace`, mt only adds.
    //
    // `--apply` only: the positional form upserts one edge, so a `--replace`
    // there would silently clear every other edge on the task, a different
    // command wearing the same name. Both bounds are declared because
    // `requires` alone does not fire once `task` is present (its own
    // `conflicts_with = "apply"` discharges the requirement), so the conflict
    // is what actually rejects `mt link a depends-on b --replace`.
    #[arg(long, requires = "apply", conflicts_with = "task")]
    pub replace: bool,

    /// Show what would change without writing anything.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `link` command.
///
/// # Errors
/// Returns an error if the project cannot be opened, the adjacency map cannot
/// be read or parsed, or any entry failed to resolve, check, write or
/// revalidate — the last of which still leaves the entries that did land.
pub fn run(args: &LinkArgs) -> Result<()> {
    let project = util::open()?;
    util::warn_errors(&project.outcome);
    run_with(&project, args)
}

/// Plan, write and report one run against an already-loaded `project`.
pub(crate) fn run_with(project: &Project, args: &LinkArgs) -> Result<()> {
    let entries = entries(args)?;
    let total = entries.len();
    if entries.is_empty() {
        println!("nothing to link");
        return Ok(());
    }
    let mode = if args.replace {
        Mode::Replace
    } else {
        Mode::Merge
    };
    let plan = plan::build(project, &entries, mode);

    if args.dry_run {
        report_plan(&plan);
        println!("dry run: nothing written");
        return finish(plan.problems, plan.failed, total);
    }

    let mut problems = plan.problems;
    let mut failed = plan.failed;
    let mut landed = Landed::default();
    // The batch goes to the pre-write check as one set: `--apply` routinely
    // adds edges in both directions at once, and an edge whose other end is
    // staged in the same run only resolves when both candidates are present.
    let pending: Vec<Pending<'_>> = plan
        .staged
        .iter()
        .map(|staged| Pending {
            file: &staged.abs,
            source: &staged.updated,
        })
        .collect();
    let verdicts = pre_write::check(project, &project.outcome.tasks, &pending)?;

    for (staged, verdict) in plan.staged.iter().zip(&verdicts) {
        // Prints this entry's warnings and breaches, and returns its refusal.
        if let Some(refusal) = pre_write::settle(
            std::slice::from_ref(&staged.stem),
            std::slice::from_ref(verdict),
        ) {
            problems.push(refusal);
            failed += 1;
            continue;
        }
        match util::write_task(&project.config, &staged.abs, staged.updated.clone()) {
            Ok(()) => {
                report_written(staged);
                landed.record(&staged.task);
            }
            Err(err) => {
                problems.push(err.into());
                failed += 1;
            }
        }
    }
    for stem in &plan.unchanged {
        println!("unchanged {stem}: every edge was already there");
    }
    if !landed.is_empty() {
        after_batch_diags(&project.root, &landed, &mut problems)?;
    }
    finish(problems, failed, total)
}

/// The entries one invocation asks for: the whole adjacency map under
/// `--apply`, or the single edge set the positional form spells.
///
/// # Errors
/// Returns an error if the map cannot be read or parsed, or — on the
/// positional form — a required argument is missing (clap guarantees these,
/// so reaching one of these bails means the attributes and this function have
/// drifted apart).
fn entries(args: &LinkArgs) -> Result<Vec<Entry>> {
    if let Some(source) = &args.apply {
        return Ok(input::parse_adjacency(&input::read_source(source)?)?);
    }
    let (Some(task), Some(kind)) = (args.task.as_deref(), args.kind.as_deref()) else {
        return Err(
            Diag::usage("mt link needs a task, a kind and targets, or --apply")
                .hint(Hint::new(
                    "usage: mt link <task> <kind> <target>... | mt link --apply <file>",
                ))
                .into(),
        );
    };
    if args.targets.is_empty() {
        return Err(Diag::usage("no link targets given")
            .hint(Hint::command(
                "name at least one",
                Command::mt()
                    .word("link")
                    .arg(task)
                    .arg(kind)
                    .arg("<target>"),
            ))
            .into());
    }
    let edges = args
        .targets
        .iter()
        .map(|target| EdgeSpec {
            kind: kind.to_owned(),
            target: target.clone(),
            note: args.note.clone(),
        })
        .collect();
    Ok(vec![Entry {
        stem: task.to_owned(),
        edges,
    }])
}

/// Print what the run would do, one line per entry (`--dry-run`).
fn report_plan(plan: &Plan) {
    println!("would rewrite {} task(s):", plan.staged.len());
    for staged in &plan.staged {
        println!("  - {}: {}", staged.stem, change_summary(staged));
    }
    for stem in &plan.unchanged {
        println!("  - {stem}: unchanged");
    }
}

/// Print what one entry just did.
fn report_written(staged: &Staged) {
    println!("linked {}: {}", staged.stem, change_summary(staged));
}

/// One entry's change, worded the same in the plan and in the applied report so
/// a `--dry-run` diff against a real run is a diff of the verbs alone.
fn change_summary(staged: &Staged) -> String {
    let added: Vec<String> = staged
        .added
        .iter()
        .map(|edge| format!("{} -> {}", edge.kind, edge.target))
        .collect();
    let dropped = if staged.removed == 0 {
        String::new()
    } else {
        format!(" (dropping {} edge(s))", staged.removed)
    };
    if added.is_empty() {
        return format!("rewrote links:{dropped}");
    }
    format!("+{}{dropped}", added.join(", +"))
}

/// Report a batch's `problems` and set the exit status, or succeed on none.
/// Same shape as `mt flip`'s tail: a single target's last problem is the
/// command's error and gets no summary; several targets end with one. The
/// exit status does not undo what landed.
fn finish(mut problems: Vec<Diag>, failed: usize, total: usize) -> Result<()> {
    let last = match problems.pop() {
        None => return Ok(()),
        Some(last) if total <= 1 => last,
        Some(last) => {
            problems.push(last);
            Diag::batch("linked", total.saturating_sub(failed), total)
        }
    };
    for problem in &problems {
        emit(problem);
    }
    Err(last.into())
}

#[cfg(test)]
mod tests;
