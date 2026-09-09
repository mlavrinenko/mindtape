//! `mt eval` — evaluate a single task file and print it.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mindtape::Task;
use typst_harvest::{HarvestWorld, harvest};

use crate::output::{OutputFormat, render, task_to_json};
use crate::render::RenderCtx;

/// Arguments for `eval`.
#[derive(Debug, clap::Args)]
pub struct EvalArgs {
    /// Task file to evaluate.
    pub file: PathBuf,

    /// Emit JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// Run the `eval` command.
///
/// # Errors
/// Returns an error if the file cannot be evaluated or holds no task marker.
pub fn run(args: &EvalArgs) -> Result<()> {
    let world = build_world(&args.file)?;
    let result = harvest(&world)?;
    let marker = result
        .with_marker("mindtape.task")
        .next()
        .with_context(|| format!("no mindtape.task marker in {}", args.file.display()))?;

    let id = args
        .file
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let task = Task::from_marker(id, args.file.clone(), &marker.value);

    if args.json {
        print_json(&task)
    } else {
        print_table(&task)
    }
}

/// Print one harvested task as a bare JSON object — no envelope, since there
/// is no listing here for `applied` or `truncated` to describe.
///
/// # Errors
/// Returns an error if the task will not serialize.
fn print_json(task: &Task) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&task_to_json(task, &mindtape::Config::default()))?
    );
    Ok(())
}

/// Print one harvested task as a one-row `ls` table.
///
/// It renders against the embedded floor, not [`mindtape::Config::default`]:
/// every column's `when` expression lives in `defaults.toml`, so a config that
/// never went through `parse_config` describes no column at all and would print
/// an empty table. There is no project here to layer over it. The environment
/// stays default too — one file is not a listing, so nothing is asking
/// `nonempty()` or `tty()` for a real answer.
///
/// # Errors
/// Returns an error if the embedded floor stops parsing, or if a column's
/// `when` fails to evaluate.
fn print_table(task: &Task) -> Result<()> {
    let config = mindtape::parse_config("")?;
    let readiness = mindtape::Readiness::empty();
    let env = mindtape::ConditionEnv::default();
    let ctx = RenderCtx {
        config: &config,
        readiness: &readiness,
        env: &env,
    };
    println!(
        "{}",
        render(std::slice::from_ref(task), OutputFormat::Table, ctx, None)?
    );
    Ok(())
}

pub(crate) fn build_world(file: &Path) -> Result<HarvestWorld> {
    if !file.is_file() {
        bail!("no such file: {}", file.display());
    }
    let world = HarvestWorld::new(file).map_err(anyhow::Error::from)?;
    let lib = world.root().join("lib");
    Ok(if lib.join("typst.toml").is_file() {
        world.with_local_package("mindtape", lib)
    } else {
        world
    })
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
