//! `mt fix` — run the fixers that repair files an `mt` upgrade left behind.
//!
//! Today there is one fixer. `prelude-import` rewrites an import of
//! `@local/mindtape` at an older version this binary no longer reads, changing
//! only the version bytes. `--check` reports an unsupported pin without writing
//! and exits 1 while one remains; `--dry-run` prints every edit; `--list`
//! names the fixers.
//!
//! The scan reads config with [`mindtape::load_config`] and never loads tasks:
//! a file with an unsupported import is exactly the file a task load would
//! fail on. The whole scan is a preflight, so a newer pin, a malformed spec,
//! an unreadable file or a symlinked file with an edit aborts before any file
//! changes. A successful write ends with `mt check`.
//!
//! # Files
//!
//! - `scope.rs`: the files the default scope and `--all` scan.
//! - `report.rs`: the per-import rows and the closing count.

mod report;
mod scope;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use mindtape::PRELUDE_SUPPORTED;
use mindtape::prelude_import::{
    PreludeImport, PreludeImportAction, PreludeRefusal, apply_rewrites, prelude_imports,
    project_prelude_pin_version,
};

use crate::cmd::install::hint::unsupported_prelude_diag;
use crate::diag::{Diag, Hint, Kind, Note, Result, emit};
use crate::render::widest;
use crate::util;
use report::Line;

/// Arguments for `fix`.
#[allow(clippy::struct_excessive_bools)] // independent verb, mode and scope switches
#[derive(Debug, Args)]
pub struct FixArgs {
    /// Run one fixer. Without one, mt runs every applicable fixer.
    #[arg(value_name = "FIXER", value_enum)]
    pub fixer: Option<Fixer>,

    /// List the fixers and exit.
    #[arg(long)]
    pub list: bool,

    /// Print every edit and write nothing.
    #[arg(long)]
    pub dry_run: bool,

    /// Exit 1 while an unsupported prelude import remains, writing nothing.
    #[arg(long)]
    pub check: bool,

    /// Scan the whole project tree, not just task roots and template dirs.
    #[arg(long)]
    pub all: bool,
}

/// A fixer `mt fix` can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Fixer {
    /// Rewrite an unsupported `@local/mindtape` import to the current version.
    PreludeImport,
}

impl Fixer {
    /// Every fixer, in `--list` order.
    pub(crate) const ALL: &'static [Self] = &[Self::PreludeImport];

    /// The name the command line and `--list` use.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::PreludeImport => "prelude-import",
        }
    }

    /// What this fixer rewrites, for `--list` and completion help.
    pub(crate) const fn about(self) -> &'static str {
        match self {
            Self::PreludeImport => {
                "rewrite an unsupported @local/mindtape import to the current version"
            }
        }
    }
}

/// One file the scan read.
struct ScannedFile {
    /// Absolute path on disk.
    path: PathBuf,
    /// Project-relative path for a report.
    display: PathBuf,
    /// The source as read.
    source: String,
    /// Whether the path itself is a symlink.
    symlink: bool,
    /// The prelude imports the file holds.
    imports: Vec<PreludeImport>,
}

/// The whole preflight: files read, and the files that could not be read.
struct Scan {
    files: Vec<ScannedFile>,
    errors: Vec<Diag>,
}

/// One version an unsupported import names, and the first file that names it.
struct SeenVersion {
    first: PathBuf,
    pinned: bool,
}

/// A source ready to land, built before anything is written.
struct PendingWrite {
    path: PathBuf,
    source: String,
}

/// The plan a normal or dry run prints and, when it is clean, applies.
struct Plan {
    lines: Vec<Line>,
    refusals: Vec<Diag>,
    writes: Vec<PendingWrite>,
}

/// Run `mt fix`.
///
/// # Errors
/// Returns an error when the project cannot be found or its config cannot be
/// loaded, when a file refuses the scan, or when the post-write `mt check`
/// finds a problem.
pub fn run(args: &FixArgs) -> Result<()> {
    if args.list {
        list();
        return Ok(());
    }
    let root = util::resolve_root()?;
    let config = mindtape::load_config(&root)?;
    let pin = project_prelude_pin_version(&root, &config);
    let paths = scope::collect(&root, &config, args.all)?;
    let scan = read_all(&root, &paths, pin.as_deref());
    if args.check {
        return scan.check();
    }
    if !enabled(args, Fixer::PreludeImport) {
        return Ok(());
    }
    scan.fix(args.dry_run)
}

/// Whether `args` asks for `fixer`: every fixer when none is named.
fn enabled(args: &FixArgs, fixer: Fixer) -> bool {
    args.fixer.is_none_or(|chosen| chosen == fixer)
}

/// Print the fixers, one name and its purpose per line.
fn list() {
    let width = widest(Fixer::ALL.iter().map(|fixer| fixer.name()));
    for fixer in Fixer::ALL {
        println!("{:<width$}  {}", fixer.name(), fixer.about());
    }
}

/// Read every file in `paths`, collecting the imports each holds.
fn read_all(root: &Path, paths: &[PathBuf], pin: Option<&str>) -> Scan {
    let mut scan = Scan {
        files: Vec::new(),
        errors: Vec::new(),
    };
    for path in paths {
        let display = path.strip_prefix(root).unwrap_or(path).to_path_buf();
        let symlink =
            std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink());
        match std::fs::read_to_string(path) {
            Ok(source) => {
                let imports = prelude_imports(&source, pin);
                scan.files.push(ScannedFile {
                    path: path.clone(),
                    display,
                    source,
                    symlink,
                    imports,
                });
            }
            Err(err) => scan.errors.push(Diag::cannot_read(&display, &err)),
        }
    }
    scan
}

impl Scan {
    /// `--check`: one diagnostic per unsupported version, plus every file that
    /// could not be read or whose spec is malformed. Writes nothing.
    fn check(&self) -> Result<()> {
        let mut diags = self.errors.clone();
        let mut kept = Vec::new();
        let mut versions: BTreeMap<String, SeenVersion> = BTreeMap::new();
        for file in &self.files {
            for import in &file.imports {
                let Some(version) = &import.version else {
                    diags.push(malformed_diag(&file.display, import));
                    continue;
                };
                match import.action {
                    PreludeImportAction::Current => {}
                    PreludeImportAction::Keep => kept.push(Line::kept(&file.display, version)),
                    PreludeImportAction::Rewrite => {
                        versions.entry(version.clone()).or_insert(SeenVersion {
                            first: file.display.clone(),
                            pinned: false,
                        });
                    }
                    PreludeImportAction::Refuse(reason) => {
                        let seen = versions.entry(version.clone()).or_insert(SeenVersion {
                            first: file.display.clone(),
                            pinned: reason == PreludeRefusal::Pinned,
                        });
                        seen.pinned |= reason == PreludeRefusal::Pinned;
                    }
                }
            }
        }
        report::print(&kept);
        diags.extend(
            versions
                .into_iter()
                .map(|(version, seen)| version_diag(&version, &seen)),
        );
        fail_after_emitting(diags)
    }

    /// Build the report, then write it and run `mt check`, unless `dry_run`.
    fn fix(&self, dry_run: bool) -> Result<()> {
        let plan = self.plan();
        let tally = report::print(&plan.lines);
        if !plan.refusals.is_empty() {
            return fail_after_emitting(plan.refusals);
        }
        if dry_run {
            println!("{}", report::summary("would rewrite", &tally));
            return Ok(());
        }
        if plan.writes.is_empty() {
            println!("{}", report::summary("rewrote", &tally));
            return Ok(());
        }
        write_all(&plan.writes)?;
        println!("{}", report::summary("rewrote", &tally));
        crate::cmd::check::run(&crate::cmd::check::CheckArgs::default())
    }

    /// Classify every import into a report row, a refusal or a pending edit.
    fn plan(&self) -> Plan {
        let mut plan = Plan {
            lines: Vec::new(),
            refusals: self.errors.clone(),
            writes: Vec::new(),
        };
        for file in &self.files {
            plan_file(file, &mut plan);
        }
        plan
    }
}

/// Add one file's rows, refusals and rewrite to `plan`.
fn plan_file(file: &ScannedFile, plan: &mut Plan) {
    let rewrites = file
        .imports
        .iter()
        .filter(|import| import.action == PreludeImportAction::Rewrite)
        .count();
    for import in &file.imports {
        match import.action {
            PreludeImportAction::Current => {}
            PreludeImportAction::Keep => plan.lines.push(Line::kept(
                &file.display,
                import.version.as_deref().unwrap_or(""),
            )),
            PreludeImportAction::Rewrite => {
                plan.lines.push(Line::rewrite(
                    &file.display,
                    import.version.as_deref().unwrap_or(""),
                ));
                if file.symlink {
                    plan.refusals.push(symlink_diag(file, import));
                }
            }
            PreludeImportAction::Refuse(reason) => {
                plan.refusals.push(refusal_diag(file, import, reason));
            }
        }
    }
    if rewrites == 0 || file.symlink {
        return;
    }
    match apply_rewrites(&file.source, &file.imports) {
        Some(source) => plan.writes.push(PendingWrite {
            path: file.path.clone(),
            source,
        }),
        None => plan
            .refusals
            .push(Diag::failed("can't rewrite this file's prelude import")),
    }
}

/// The diagnostic for an import the fixer refuses.
fn refusal_diag(file: &ScannedFile, import: &PreludeImport, reason: PreludeRefusal) -> Diag {
    let version = import.version.as_deref().unwrap_or("");
    match reason {
        PreludeRefusal::Newer => unsupported_prelude_diag(Kind::Refused, version, &file.display),
        PreludeRefusal::Pinned => pinned_diag(version, &file.display),
        PreludeRefusal::Malformed => malformed_diag(&file.display, import),
    }
}

/// An unsupported version a project pin owns: rewriting the import would
/// silently change which prelude the project reads.
fn pinned_diag(version: &str, file: &Path) -> Diag {
    Diag::verbatim(
        Kind::Refused,
        format!(
            "imports @local/mindtape:{version}, which this mt does not support; \
             the project pins that version, so the import is unfixable"
        ),
    )
    .at_file(file)
    .note(Note::new("supported").name(PRELUDE_SUPPORTED.join(", ")))
    .hint(Hint::new(
        "remove or upgrade the [[typst.local-packages]] entry for @local/mindtape",
    ))
}

/// A spec `mt` cannot parse, so there is no version span to rewrite.
fn malformed_diag(file: &Path, import: &PreludeImport) -> Diag {
    Diag::refused("malformed @local/mindtape import")
        .at_file(file)
        .value(&import.spec)
        .hint(Hint::new("write the spec as @local/mindtape:<version>"))
}

/// A symlinked file with an edit: the target is outside this project's tree,
/// so the fixer reports it and leaves the link alone.
fn symlink_diag(file: &ScannedFile, import: &PreludeImport) -> Diag {
    let version = import.version.as_deref().unwrap_or("");
    Diag::refused("won't rewrite a symlinked file")
        .at_file(&file.display)
        .value(format!("@local/mindtape:{version}"))
        .hint(Hint::naming(
            "rewrite its target to",
            format!("@local/mindtape:{}", mindtape::PRELUDE_VERSION),
        ))
}

/// The diagnostic for one unsupported version: the first file that names it,
/// the set of supported versions, and the rewrite or upgrade hint.
fn version_diag(version: &str, seen: &SeenVersion) -> Diag {
    if seen.pinned {
        return pinned_diag(version, &seen.first);
    }
    unsupported_prelude_diag(Kind::Refused, version, &seen.first)
}

/// Write each pending source atomically, or return the first failure.
fn write_all(writes: &[PendingWrite]) -> Result<()> {
    for write in writes {
        mindtape::write_atomic(&write.path, write.source.as_bytes())
            .map_err(|err| Diag::cannot_write(&write.path, &err))?;
    }
    Ok(())
}

/// Emit every diagnostic but the last, and return the last as the command's
/// failure so it is printed once, as the final line.
fn fail_after_emitting(mut diags: Vec<Diag>) -> Result<()> {
    match diags.pop() {
        None => Ok(()),
        Some(last) => {
            for diag in diags {
                emit(&diag);
            }
            Err(last)
        }
    }
}
