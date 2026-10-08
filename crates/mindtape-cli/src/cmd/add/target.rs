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
use crate::diag::{Diag, Hint, emit};
use crate::util;

/// Resolve the destination collision against `--dedup`/`[tasks.naming]
/// dedup`, and report a `skip` outcome immediately (nothing is written for
/// it). Returns `None` for `skip` (the caller's cue to stop); `Some(outcome)`
/// otherwise, carrying the exact path to write.
///
/// # Errors
/// Propagates [`cmd::add::dedup::resolve`]'s `fail`/exhausted-`add-suffix` errors.
pub(super) fn resolve_outcome(
    args: &AddArgs,
    config: &Config,
    stem: &str,
    dest: &Path,
    file: PathBuf,
) -> Result<Option<Outcome>> {
    let dedup = dedup::effective_mode(args.dedup, config);
    let label = file.display().to_string();
    let collision = dedup::resolve(dedup, stem, &label, |candidate| {
        dest.join(format!("{candidate}.typ")).exists()
    })?;
    Ok(match collision {
        dedup::Collision::None => Some(Outcome::Wrote(file)),
        dedup::Collision::Skipped => {
            output::emit(&Outcome::Skipped(file), args.format);
            None
        }
        dedup::Collision::Suffixed(suffixed_stem) => {
            Some(Outcome::Suffixed(dest.join(format!("{suffixed_stem}.typ"))))
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

pub(crate) fn dest_dir(into: &str, to: Option<&str>, project: &util::Project) -> Result<PathBuf> {
    let base = base_dir(into, project)?;
    let Some(sub) = to else {
        return Ok(base);
    };
    let sub = Path::new(sub);
    if sub.is_absolute() {
        return Err(
            Diag::usage("--to expects a subfolder, not an absolute path")
                .value(sub.display().to_string())
                .hint(Hint::new("give it relative to the destination root")),
        );
    }
    Ok(base.join(sub))
}

/// Resolve the destination base directory from `--into`: `.` (default) is the
/// first configured root; an absolute path is taken as-is; a relative path
/// resolves against the current directory.
fn base_dir(into: &str, project: &util::Project) -> Result<PathBuf> {
    if into == "." {
        let first = project
            .config
            .tasks
            .roots
            .first()
            .map_or(".", mindtape::RootSpec::path);
        return Ok(project.root.join(first));
    }
    let path = Path::new(into);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir()
        .map_err(|err| Diag::failed("can't determine the current directory").io(&err))?;
    Ok(cwd.join(path))
}

/// Warn when `dest` lands outside every configured root, since recursive loading
/// only sees files under a root.
pub(crate) fn warn_if_outside_roots(project: &util::Project, dest: &Path) {
    let Ok(dest_abs) = dest.canonicalize() else {
        return;
    };
    let inside = project.config.tasks.roots.iter().any(|root| {
        project
            .root
            .join(root.path())
            .canonicalize()
            .is_ok_and(|abs| dest_abs.starts_with(abs))
    });
    if !inside {
        emit(
            &Diag::warning("the task directory is outside the configured roots")
                .value(dest.display().to_string())
                .hint(Hint::new(
                    "add it to tasks.roots, or mt won't track the task",
                )),
        );
    }
}
