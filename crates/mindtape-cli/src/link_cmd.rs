//! `mt link` — the write path for `links:`, the one task field that had none.
//!
//! Two forms over one pipeline. `mt link <task> <kind> <target…>` upserts a
//! single edge; `mt link --apply <file>` reads an adjacency map (stem to kind
//! to targets, YAML or JSON, `-` for stdin) and merges every entry in it. Both
//! desugar to the same list of [`link_input::Entry`], so the checks, the
//! rendering, the dedupe and the report are written once.
//!
//! Batch semantics mirror `mt rm` and `mt flip`: the whole plan is computed in
//! memory before a byte is written ([`link_plan::build`]), so `--dry-run`
//! reports exactly what a real run would do; then the valid entries land and a
//! per-entry failure is reported without rolling the others back.
//!
//! Merging is by `(kind, target-stem)`, so running the same command twice
//! writes nothing the second time — the property that makes an adjacency map
//! safe to re-apply as it grows, and the reason the regex one-shot this
//! command replaces was a one-shot.

use anyhow::{Result, bail};
use clap::Args;

use crate::flip_batch::{Landed, after_batch};
use crate::link_edit::{EdgeSpec, Mode};
use crate::link_input::{self, Entry};
use crate::link_plan::{self, Plan, Staged};
use crate::util::{self, Project};

/// Arguments for `mt link`.
#[derive(Debug, Args)]
pub struct LinkArgs {
    /// Linking task: handle (`1b`), slug, relative path, or absolute path.
    /// Omit only with `--apply`.
    #[arg(required_unless_present = "apply", conflicts_with = "apply")]
    pub task: Option<String>,

    /// Edge kind: a built-in (`depends-on`, `parent`, `recurred-from`,
    /// `supersedes`, `duplicates`, `related`) or a configured
    /// `[[tasks.links.kinds]]` name.
    #[arg(required_unless_present = "apply")]
    pub kind: Option<String>,

    /// Target tasks, by stem (`other`), stem with extension (`other.typ`), or
    /// a path relative to the linking task.
    #[arg(required_unless_present = "apply")]
    pub targets: Vec<String>,

    /// Edge note (the trailing `[note]` block). Required for a kind whose
    /// `note` rule is `required` — `related`, out of the box.
    ///
    /// Positional form only: a map carries a note per target (`note:` beside
    /// `target:`), so one flag could not say which of them it meant. Refused
    /// rather than ignored, the way an unknown `--set` key is.
    #[arg(
        short = 'm',
        long = "note",
        value_name = "NOTE",
        conflicts_with = "apply"
    )]
    pub note: Option<String>,

    /// Read an adjacency map instead: a YAML or JSON mapping of stem to kind
    /// to targets, or `-` for stdin.
    #[arg(long, value_name = "FILE")]
    pub apply: Option<String>,

    /// Make the map the complete edge set for each stem it names, dropping
    /// every edge it does not. Merge (the default) only ever adds.
    ///
    /// `--apply` only: the positional form upserts one edge, so a `--replace`
    /// there would silently clear every other edge on the task — a different
    /// command wearing the same name. Both bounds are declared because
    /// `requires` alone does not fire once `task` is present (its own
    /// `conflicts_with = "apply"` discharges the requirement), so the conflict
    /// is what actually rejects `mt link a depends-on b --replace`.
    #[arg(long, requires = "apply", conflicts_with = "task")]
    pub replace: bool,

    /// Print the plan without writing anything.
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
    if entries.is_empty() {
        println!("nothing to link");
        return Ok(());
    }
    let mode = if args.replace {
        Mode::Replace
    } else {
        Mode::Merge
    };
    let plan = link_plan::build(project, &entries, mode);

    if args.dry_run {
        report_plan(&plan);
        println!("dry run: nothing written");
        return finish(&plan.problems);
    }

    let mut problems = plan.problems;
    let mut landed = Landed::default();
    for staged in &plan.staged {
        match util::write_task(&project.config, &staged.abs, staged.updated.clone()) {
            Ok(()) => {
                report_written(staged);
                landed.record(&staged.task);
            }
            Err(err) => problems.push(format!("{}: {err:#}", staged.stem)),
        }
    }
    for stem in &plan.unchanged {
        println!("unchanged {stem}: every edge was already there");
    }
    if !landed.is_empty() {
        after_batch(&project.root, &landed, None, &mut problems)?;
    }
    finish(&problems)
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
        return link_input::parse_adjacency(&link_input::read_source(source)?);
    }
    let (Some(task), Some(kind)) = (args.task.as_deref(), args.kind.as_deref()) else {
        bail!("usage: mt link <task> <kind> <target>... | mt link --apply <file>");
    };
    if args.targets.is_empty() {
        bail!("no targets given; usage: mt link {task} {kind} <target>...");
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

/// Report `problems` and set the exit status, or succeed on none. Same shape
/// as `mt flip`'s tail: every failure named on stderr, one summary line, and a
/// non-zero exit that does not undo what landed.
fn finish(problems: &[String]) -> Result<()> {
    if problems.is_empty() {
        return Ok(());
    }
    for problem in problems {
        eprintln!("warning: {problem}");
    }
    bail!("{} task(s) had problems", problems.len())
}

#[cfg(test)]
#[path = "link_cmd_tests.rs"]
mod tests;
