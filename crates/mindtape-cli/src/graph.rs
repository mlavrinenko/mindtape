//! `mt graph` — emit the task dependency graph as DOT or Mermaid.
//!
//! A read-only projection of the same typed source `mt ls` and `mt check`
//! read: the filter scopes which tasks become nodes, [`mindtape::edges`]
//! resolves their `links:` into the induced edge set (no re-implemented link
//! resolution), and [`crate::graph_render`] serialises it. Readiness styles
//! nodes ready/blocked/terminal.

use anyhow::Result;
use mindtape::{Filter, Readiness, edges, select};

use crate::graph_render::{GraphFormat, render_dot, render_mermaid};
use crate::util;

/// Arguments for `graph`.
#[derive(Debug, clap::Args)]
pub struct GraphArgs {
    /// Filter expression scoping the subgraph. Omitted graphs every task; an
    /// empty string (`''`) is the same match-all. Edges to tasks outside the
    /// scope are dropped, so the result is the induced subgraph.
    pub filter: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t)]
    pub format: GraphFormat,

    /// Drop terminal tasks (done/cancelled/failed) from the graph, keeping only
    /// the live work and its edges.
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
    let filter = match args.filter.as_deref() {
        Some(expr) => Some(Filter::parse(expr)?),
        None => None,
    };
    let mut tasks = select(project.outcome.tasks, filter.as_ref(), config, &readiness)?;
    if args.no_terminal {
        let terminal = config.tasks.status.terminal();
        tasks.retain(|task| !task.is_terminal(&terminal));
    }
    mindtape::default_order_with(&mut tasks, config, &readiness);
    let edges = edges(&tasks);
    Ok(match args.format {
        GraphFormat::Dot => render_dot(&tasks, &edges, &readiness),
        GraphFormat::Mermaid => render_mermaid(&tasks, &edges, &readiness),
    })
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
