//! `mt graph` — emit the task dependency graph as DOT or Mermaid.
//!
//! A read-only projection of the same typed source `mt ls` and `mt check`
//! read: the filter scopes which tasks become nodes, [`mindtape::edges`]
//! resolves their `links:` into the induced edge set (no re-implemented link
//! resolution), and [`crate::cmd::graph::render`] serialises it. Readiness styles
//! nodes ready/blocked/terminal.
//!
//! Edges of a kind that points outside the backlog (`[[tasks.links.kinds]]
//! validate` other than `local-task-path`) are split off before rendering: they
//! never join the traversed task graph, even when a target string happens to
//! spell a loaded task's stem. They render as their own leaf nodes instead —
//! see [`crate::cmd::graph::render::ExternalTarget`].
//!
//! # Files
//!
//! - `render.rs` — DOT and Mermaid rendering.

pub(crate) mod render;

use crate::diag::Result;
use mindtape::link::kind_targets_tasks;
use mindtape::{Config, Filter, Readiness, Task, edges, select};

use crate::cmd::graph::render::{ExternalTarget, GraphFormat, render_dot, render_mermaid};
use crate::util;

/// Arguments for `graph`.
#[derive(Debug, clap::Args)]
pub struct GraphArgs {
    /// Which tasks to draw, as a filter expression. Without it, mt draws every
    /// task. Links to tasks the filter leaves out aren't drawn.
    pub filter: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t)]
    pub format: GraphFormat,

    /// Leave out closed tasks, such as done or cancelled ones.
    #[arg(long)]
    pub no_terminal: bool,
}

/// Run the `graph` command.
///
/// # Errors
/// Returns an error if the project cannot be opened or the filter is invalid.
pub fn run(args: &GraphArgs) -> Result<()> {
    run_with(args, util::open()?)
}

pub(crate) fn run_with(args: &GraphArgs, project: util::Project) -> Result<()> {
    util::warn_errors(&project.outcome);
    println!("{}", build(args, project)?);
    Ok(())
}

/// Scope, style, and serialise the graph into its output string.
///
/// Readiness is computed over the *full* loaded set before the filter narrows
/// it, so a terminal dependency hidden by the filter still counts as satisfied
/// (a leaf under `!terminal` reads ready, not blocked). The filter then selects
/// the node set, `--no-terminal` optionally prunes settled tasks, and
/// [`edges`] induces the edges among whatever nodes remain.
///
/// # Errors
/// Returns an error if the filter expression is invalid.
fn build(args: &GraphArgs, project: util::Project) -> Result<String> {
    let config = &project.config;
    let readiness = Readiness::compute(&project.outcome.tasks, config);
    let typed = |err| crate::diag::filter(err, args.filter.as_deref());
    let filter = match args.filter.as_deref() {
        Some(expr) => Some(Filter::parse(expr).map_err(typed)?),
        None => None,
    };
    let mut tasks =
        select(project.outcome.tasks, filter.as_ref(), config, &readiness).map_err(typed)?;
    if args.no_terminal {
        let terminal = config.tasks.status.terminal();
        tasks.retain(|task| !task.is_terminal(&terminal));
    }
    mindtape::default_order_with(&mut tasks, config, &readiness);
    let mut graph_edges = edges(&tasks);
    graph_edges.retain(|edge| kind_targets_tasks(edge.kind, &config.tasks.links));
    let external = external_targets(&tasks, config);
    Ok(match args.format {
        GraphFormat::Dot => render_dot(&tasks, &graph_edges, &external, &readiness),
        GraphFormat::Mermaid => render_mermaid(&tasks, &graph_edges, &external, &readiness),
    })
}

/// Every `links:` edge in `tasks` whose kind points outside the backlog, in
/// authored order per task. These are never resolved against the task set — the
/// target is carried through verbatim for the renderer to show as a leaf.
fn external_targets<'a>(tasks: &'a [Task], config: &Config) -> Vec<ExternalTarget<'a>> {
    let mut out = Vec::new();
    for task in tasks {
        for link in &task.links {
            if !kind_targets_tasks(&link.kind, &config.tasks.links) {
                out.push(ExternalTarget {
                    from: task,
                    kind: &link.kind,
                    target: &link.task,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
