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
        println!(
            "{}",
            serde_json::to_string_pretty(&task_to_json(&task, &mindtape::Config::default()))?
        );
    } else {
        // `mt eval` renders one harvested file against the empty config, not a
        // project: no `[ls.columns]` entries exist, so no column carries a
        // conditional rule and the environment has nothing to answer.
        let config = mindtape::Config::default();
        let readiness = mindtape::Readiness::empty();
        let env = mindtape::ConditionEnv::default();
        let ctx = RenderCtx {
            config: &config,
            readiness: &readiness,
            env: &env,
        };
        println!(
            "{}",
            render(std::slice::from_ref(&task), OutputFormat::Table, ctx, None)?
        );
    }
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
