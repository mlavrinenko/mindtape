//! `mt eval` — evaluate a single task file and print it.

use std::path::{Path, PathBuf};

use crate::diag::Result;
use mindtape::{Config, StemHashes, Task, find_root, load_config, load_project};
use typst_harvest::{HarvestWorld, harvest};

use crate::diag::{Diag, Hint};
use crate::render::RenderCtx;
use crate::render::output::{OutputFormat, render, task_to_json};
use crate::util::Project;

/// Arguments for `eval`.
#[derive(Debug, clap::Args)]
pub struct EvalArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,

    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// Run the `eval` command.
///
/// # Errors
/// Returns an error if the file cannot be evaluated or holds no task marker.
pub fn run(args: &EvalArgs) -> Result<()> {
    let (file, project) = locate(&args.task.task)?;
    let origin = project
        .as_ref()
        .map(|project| (project.root.clone(), project.hyperlinks));
    let world = build_world(&file)?;
    let result =
        harvest(&world).map_err(|err| Diag::failed("can't evaluate").at_file(&file).cause(&err))?;
    let marker = result.with_marker("mindtape.task").next().ok_or_else(|| {
        Diag::failed("not a task file: it holds no task.with(…) call")
            .at_file(&file)
            .hint(Hint::new("start the file with #show: task.with(...)"))
    })?;

    let id = file
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut task = Task::from_marker(id, file.clone(), &marker.value);
    resolve_counts(&mut task, &file, project.as_ref());

    if args.json {
        print_json(&task)
    } else {
        print_table(&task, &hashes_for(&file, &task, project), origin)?;
        // The table already ends in a blank line.
        print!("{}", breakdown(&task));
        Ok(())
    }
}

/// The file `reference` names, and the project that resolved it. Anything
/// spelled as a path stays a path, so a `.typ` under no project still
/// evaluates; a stem or stem hash resolves against the enclosing project the
/// way `flip`, `set` and `link` do, and that project is handed back so nothing
/// loads it again.
///
/// # Errors
/// Returns an error if a path does not exist, no project encloses the working
/// directory, or no task matches the reference.
fn locate(reference: &str) -> Result<(PathBuf, Option<Project>)> {
    let path = Path::new(reference);
    if path.is_file() {
        return Ok((path.to_path_buf(), None));
    }
    if mindtape::is_path_ref(reference) {
        return Err(Diag::failed("no such file").value(reference));
    }
    let project = crate::util::open()?;
    let (_, abs) = crate::util::resolve_task(&project, reference)?;
    Ok((abs, Some(project)))
}

/// Resolve `task`'s `count(...)` units against the project enclosing `file`:
/// `project` when the reference resolved one, else the one above the file.
/// Under no project, the default config names no unit, so each count records
/// why it can't resolve.
fn resolve_counts(task: &mut Task, file: &Path, project: Option<&Project>) {
    if task.estimate_counts.is_empty() {
        return;
    }
    let (root, config) = project
        .map(|project| (project.root.clone(), project.config.clone()))
        .or_else(|| {
            let root = find_root(file.canonicalize().ok()?.parent()?)?;
            let config = load_config(&root).ok()?;
            Some((root, config))
        })
        .unwrap_or_else(|| {
            let dir = file.parent().unwrap_or_else(|| Path::new("."));
            (dir.to_path_buf(), Config::default())
        });
    mindtape::estimate_queries::annotate(std::slice::from_mut(task), &config, &root);
}

/// Where each `count(...)` unit's number comes from: the resolved total, the
/// manual part, and each query with the files it matched, aligned in two
/// columns. A unit that didn't resolve prints why instead. Empty for a task
/// with no count.
fn breakdown(task: &Task) -> String {
    let mut out = String::new();
    for (unit, count) in &task.estimate_counts {
        let Some(total) = task.estimate.get(unit).filter(|_| count.problem.is_none()) else {
            let problem = count.problem.as_deref().unwrap_or("not resolved");
            out.push_str(&format!("estimate.{unit}: {problem}\n"));
            continue;
        };
        let mut rows = vec![(mindtape::format_num(count.manual), "manual".to_owned())];
        rows.extend(
            count
                .queries
                .iter()
                .zip(&count.matches)
                .map(|(query, files)| (mindtape::count(*files, "file"), query.label())),
        );
        let width = rows
            .iter()
            .map(|(left, _)| left.chars().count())
            .max()
            .unwrap_or(0);
        out.push_str(&format!(
            "estimate.{unit} = {}\n",
            mindtape::format_num(*total)
        ));
        for (left, label) in rows {
            out.push_str(&format!("  {left:<width$}  {label}\n"));
        }
    }
    out
}

/// Print one harvested task as a bare JSON object — no envelope, since there
/// is no listing here for `applied` or `truncated` to describe.
///
/// # Errors
/// Returns an error if the task will not serialize.
fn print_json(task: &Task) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&task_to_json(task, &Config::default()))?
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
fn print_table(task: &Task, hashes: &StemHashes, origin: Option<(PathBuf, bool)>) -> Result<()> {
    let (root, hyperlinks) = origin.unwrap_or_default();
    let config = mindtape::parse_config("")?;
    let readiness = mindtape::Readiness::empty();
    let env = mindtape::ConditionEnv::default();
    let ctx = RenderCtx {
        config: &config,
        readiness: &readiness,
        env: &env,
        hashes,
        root: &root,
        hyperlinks,
        project: None,
    };
    println!(
        "{}",
        render(std::slice::from_ref(task), OutputFormat::Table, ctx, None)?
    );
    Ok(())
}

/// The stem hashes of the project enclosing `file`, so the `slug#` cell is
/// the one every listing prints. `resolved` is the project the reference
/// already loaded; without one the enclosing project loads here. A file under
/// no project, or one its project does not load, is hashed alone.
fn hashes_for(file: &Path, task: &Task, resolved: Option<Project>) -> StemHashes {
    let project = resolved.or_else(|| {
        let root = find_root(file.canonicalize().ok()?.parent()?)?;
        let config = load_config(&root).ok()?;
        let outcome = load_project(&root, &config);
        Some(Project {
            root,
            config,
            outcome,
            hyperlinks: false,
        })
    });
    match project {
        Some(project) if project.outcome.tasks.iter().any(|t| t.id == task.id) => {
            crate::util::stem_hashes(&project.outcome.tasks, &project.config)
        }
        _ => StemHashes::new([task.id.as_str()]),
    }
}

/// Build the world one task file evaluates in.
///
/// The world's own root stays the one `HarvestWorld::new` detects, since that
/// is what the file's virtual path is relative to. Which `@local` packages
/// resolve is a different question, and not one this command gets to answer:
/// it asks [`mindtape::package_overrides`], the same function the batch loader
/// asks, against the enclosing project, so `mt eval` and `mt ls` resolve one
/// import to one package.
///
/// # Errors
/// Returns an error if the file does not exist, the world cannot be built, or
/// the project's config does not parse.
pub(crate) fn build_world(file: &Path) -> Result<HarvestWorld> {
    if !file.is_file() {
        return Err(Diag::failed("no such file").value(file.display().to_string()));
    }
    let world = HarvestWorld::new(file)
        .map_err(|err| Diag::failed("can't set up the Typst world").cause(&err))?;
    // `find_root` looks for `.mindtape`, which the Typst root markers
    // (`Cargo.toml`, `lib/`, `.git`) need not agree with; a file under no
    // project at all falls back to the world's root, where the lookup finds no
    // config and so no `[[typst.local-packages]]` entry.
    let root = find_root(world.root()).unwrap_or_else(|| world.root().to_path_buf());
    let config = load_config(&root)?;
    let mut world = world.with_package_root(mindtape::package_root());
    for (name, dir) in mindtape::package_overrides(&root, &config) {
        world = world.with_local_package(&name, dir);
    }
    Ok(world)
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
