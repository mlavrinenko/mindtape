//! Where `mt add`'s new task file lands: the destination directory, the
//! minted stem, and the write outcome under `--dedup` collision policy — the
//! three concerns of resolving the write target, split out of `cmd/add/mod.rs` to
//! stay under the file-size gate. `dest_dir`/`warn_if_outside_roots` are also
//! reused by `mt import`, which shares the same destination-resolution rules
//! (see `crate::cmd::import`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::diag::Result;
use mindtape::{Config, RawValue, model::naming, today_ymd};

use super::AddArgs;
use crate::cmd::add::dedup;
use crate::cmd::add::output::{self, Outcome};
use crate::cmd::add::values::slug;
use crate::cmd::write_guard::{self, Mode};
use crate::diag::{Diag, Hint, emit};
use crate::util;

/// Resolve the destination collision against `--dedup`/`[tasks.naming]
/// dedup`, then guard the final path, and report a `skip` outcome immediately
/// (nothing is written for it). Returns `None` for `skip` (the caller's cue to
/// stop); `Some(outcome)` otherwise, carrying the exact path to write.
///
/// The guard runs on the path this resolution chose, suffix included: a
/// suffix can change which ignore rule matches, and a `skip` collision writes
/// nothing at all, so it needs no guard.
///
/// # Errors
/// Propagates [`crate::cmd::add::dedup::resolve`]'s `fail`/exhausted-`add-suffix` errors
/// and the destination guard's refusal.
pub(super) fn resolve_outcome(
    args: &AddArgs,
    project: &util::Project,
    stem: &str,
    dest: &Path,
    file: PathBuf,
) -> Result<Option<Outcome>> {
    let dedup = dedup::effective_mode(args.dedup, &project.config);
    let label = mindtape::display_path(&file);
    let collision = dedup::resolve(dedup, stem, &label, |candidate| {
        dest.join(format!("{candidate}.typ")).exists()
    })?;
    Ok(match collision {
        dedup::Collision::None => {
            write_guard::check(project, &file, Mode::Add)?;
            Some(Outcome::Wrote(file))
        }
        dedup::Collision::Skipped => {
            let outcome = Outcome::Skipped(file);
            output::emit(
                &outcome,
                &output::handle(project, &outcome),
                super::effective_format(args),
            );
            None
        }
        dedup::Collision::Suffixed(suffixed_stem) => {
            let file = dest.join(format!("{suffixed_stem}.typ"));
            write_guard::check(project, &file, Mode::Add)?;
            Some(Outcome::Suffixed(file))
        }
    })
}

/// Compute the new task's filename stem: an explicit `--name` wins outright
/// (literal, never templated — an explicit request is trusted as-is).
/// Otherwise the title's slug feeds the configured `[tasks.naming] stem`
/// template (today's date; the pure expander never reads a clock — see
/// [`mindtape::model::naming`]); unset naming keeps today's bare behaviour
/// untouched.
///
/// Pure minting only — this never checks `dest` or decides what to do about
/// a collision. Whatever produced the stem, [`resolve_outcome`] is the single
/// place that resolves it against the destination and the effective `--dedup`
/// mode; unifying that decision (rather than pre-disambiguating a templated
/// stem here, as before) is what makes `fail`/`skip` reachable under a
/// templated `stem` at all.
pub(super) fn output_stem(
    args: &AddArgs,
    values: &BTreeMap<String, RawValue>,
    config: &Config,
) -> Result<String> {
    if let Some(name) = &args.name {
        return Ok(name.clone());
    }
    minted_stem(values, config)
}

/// The stem `mt add` mints from the filled `title` when `--name` is absent: the
/// title's slug, or `[tasks.naming] stem` over it.
pub(crate) fn minted_stem(values: &BTreeMap<String, RawValue>, config: &Config) -> Result<String> {
    let slug_value = title_slug(values)?;
    let Some(template) = config.tasks.naming.stem.as_deref() else {
        return Ok(slug_value);
    };
    let ctx = naming::MintContext {
        slug: &slug_value,
        date: today_ymd(),
        occurrence: 0,
    };
    naming::mint_stem(template, &ctx).map_err(|err| {
        Diag::failed("can't mint the file stem")
            .cause(&err)
            .hint(Hint::new("check stem under [tasks.naming]"))
    })
}

/// The title's slug, or an error when there is nothing to name the file from.
fn title_slug(values: &BTreeMap<String, RawValue>) -> Result<String> {
    if let Some(RawValue::Str(title)) = values.get("title") {
        let stem = slug(title);
        if !stem.is_empty() {
            return Ok(stem);
        }
    }
    Err(Diag::usage("no title to name the task file from")
        .hint(Hint::new("pass a title, or --name")))
}

/// Resolve the one destination directory: `--dir` when given, else the first
/// configured root.
///
/// A relative `--dir` resolves from the project root, so the same command
/// writes to the same place whatever directory it runs in. An absolute one is
/// taken as-is.
pub(crate) fn dest_dir(dir: Option<&str>, project: &util::Project) -> PathBuf {
    let Some(dir) = dir else {
        let first = project
            .config
            .tasks
            .roots
            .first()
            .map_or(".", mindtape::RootSpec::path);
        return project.root.join(first);
    };
    let path = Path::new(dir);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    project.root.join(path)
}

/// The directory a hooked add writes to: an explicit `--dir` wins, then the
/// last hook's `dir`, then the first root.
///
/// # Errors
/// A hook-chosen directory outside every configured root.
pub(super) fn resolve_dir(
    args: &AddArgs,
    project: &util::Project,
    hook_dir: Option<&str>,
) -> Result<PathBuf> {
    if let Some(dir) = args.dir.as_deref() {
        return Ok(dest_dir(Some(dir), project));
    }
    let Some(dir) = hook_dir else {
        return Ok(dest_dir(None, project));
    };
    let dest = project.root.join(dir);
    if !project.holds(&dest) {
        return Err(outside_root(dir));
    }
    Ok(dest)
}

/// Refuse an explicit `--dir` outside every root before a hooked add builds
/// its draft: a hooked add writes under the roots, where the filed task has a
/// project-relative path, derived tags and a depth.
///
/// # Errors
/// A usage error naming the outside-root destination.
pub(super) fn refuse_outside_explicit(dir: Option<&str>, project: &util::Project) -> Result<()> {
    let Some(dir) = dir else {
        return Ok(());
    };
    let dest = dest_dir(Some(dir), project);
    if project.holds(&dest) {
        return Ok(());
    }
    Err(outside_root(dir))
}

/// A hook or explicit destination outside every root, which a hooked add
/// refuses: no root means no derived tags, no depth and no project-relative
/// draft path.
fn outside_root(dir: &str) -> Diag {
    Diag::refused("the task directory is outside the configured roots")
        .value(dir)
        .hint(Hint::new(
            "a hooked add needs a destination under tasks.roots",
        ))
}

/// Refuse the removed `--into`/`--to` flags by name, with the replacement in
/// the hint. Both commands keep the old spellings declared but hidden, so a
/// caller gets this instead of clap's generic unknown-argument error.
///
/// # Errors
/// A usage error naming the first old flag that was passed.
pub(crate) fn refuse_legacy(into: Option<&String>, to: Option<&String>) -> Result<()> {
    let flag = if into.is_some() {
        "--into"
    } else if to.is_some() {
        "--to"
    } else {
        return Ok(());
    };
    Err(Diag::usage("this destination flag is gone")
        .value(flag)
        .hint(Hint::new(match flag {
            "--into" => "use --dir instead of --into",
            _ => "use --dir instead of --to",
        })))
}

/// Warn when `dest` lands outside every configured root, since recursive loading
/// only sees files under a root.
pub(crate) fn warn_if_outside_roots(project: &util::Project, dest: &Path) {
    if !project.holds(dest) {
        emit(
            &Diag::warning("the task directory is outside the configured roots")
                .value(mindtape::display_path(dest))
                .hint(Hint::new(
                    "add it to tasks.roots, or mt won't track the task",
                )),
        );
    }
}
