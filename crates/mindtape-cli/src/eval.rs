//! `mt eval` — evaluate a single task file and print it.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use mindtape::{Task, config_path, find_root, load_config};
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

/// Build the world one task file evaluates in.
///
/// The world's own root stays the one `HarvestWorld::new` detects, since that
/// is what the file's virtual path is relative to. Which `@local` packages
/// resolve is a different question, and not one this command gets to answer:
/// it asks [`mindtape::package_overrides`], the same function the batch loader
/// asks, against the enclosing project. Deciding here instead is what let
/// `mt eval` register any `<root>/lib` holding a `typst.toml` as `mindtape` —
/// so in a worktree whose `lib/` is some other package it resolved the import
/// there and died on the manifest's declared name, while `mt ls` on the same
/// tree was fine.
///
/// # Errors
/// Returns an error if the file does not exist, the world cannot be built, or
/// the project's config does not parse.
pub(crate) fn build_world(file: &Path) -> Result<HarvestWorld> {
    if !file.is_file() {
        bail!("no such file: {}", file.display());
    }
    let world = HarvestWorld::new(file).map_err(anyhow::Error::from)?;
    // `find_root` looks for `.mindtape`, which the Typst root markers
    // (`Cargo.toml`, `lib/`, `.git`) need not agree with; a file under no
    // project at all falls back to the world's root, where the lookup finds no
    // config and no in-tree lib either.
    let root = find_root(world.root()).unwrap_or_else(|| world.root().to_path_buf());
    let config =
        load_config(&root).with_context(|| format!("load {}", config_path(&root).display()))?;
    let mut world = world.with_package_root(mindtape::package_root());
    for (name, dir) in mindtape::package_overrides(&root, &config) {
        world = world.with_local_package(&name, dir);
    }
    Ok(world)
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
