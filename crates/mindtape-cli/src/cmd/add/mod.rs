//! `mt add` — file a new task from a Typst slot template: fill its slots,
//! validate, then write (or print under `--print`). A rejected add writes nothing.
//!
//! # Files
//!
//! - `dedup.rs`, `target.rs`, `output.rs` — `--dedup`, destination and stem,
//!   `--format`/`--json`.
//! - `values.rs`, `values_ops.rs`, `tokens.rs`, `field_values.rs`, `yaml_values.rs`, `templates/` — value sources, ordered `--set` operations, non-slot field writes, YAML.
//! - `draft.rs`, `hooks.rs`, `pipeline.rs` — the partially filled task, `[[on.add]]` execution and the render order.
//! - `interactive.rs`, `hint.rs`, `notes.rs`, `title.rs` — editor fill, refusal hint, fill note,
//!   title from words or stdin.

pub(crate) mod dedup;
mod draft;
mod hooks;
pub(crate) mod interactive;
pub(crate) mod output;
mod pipeline;
pub(crate) mod tokens;
pub(crate) mod values;
mod values_ops;
pub(crate) mod yaml_values;

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write as _;
use std::path::Path;

use crate::diag::Result;
use clap::Args;
use mindtape::{Config, FieldOp, RawValue, SlotKind, SlotSpec, discover_slots, escape_markup};

use self::output::Format;
use crate::cmd::add::dedup::DedupArg;
use crate::cmd::add::values::{DescribeFormat, describe_for, list_templates, resolve_template};
use crate::diag::{Diag, Hint, Note};
use crate::task_edit::filing_status;
use crate::util;

mod hint;
use hint::Rescue;
mod field_values;
mod header;
pub(crate) use header::{inject_header, splice_config_slots};
mod notes;
mod target;
mod title;
use notes::report_injected;
pub(crate) use target::minted_stem;
pub(crate) use target::{dest_dir, refuse_legacy, warn_if_outside_roots};
use target::{output_stem, resolve_dir, resolve_outcome};

/// Arguments for `mt add`.
// A clap flag struct: each `--flag` is an independent boolean toggle.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub struct AddArgs {
    /// The task's title.
    ///
    /// mt joins the words with spaces.
    #[arg(value_name = "TITLE")]
    pub title: Vec<String>,

    /// The template to fill.
    ///
    /// Defaults to your project's default template.
    #[arg(long = "template", short = 't', value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// Read slot values from a JSON or YAML file, or `-` for stdin.
    ///
    /// Repeat it to merge several files, and a later file wins on a shared
    /// key.
    #[arg(long = "values", short = 'f', value_name = "PATH")]
    pub values: Vec<String>,

    /// Set a slot or a core field, as `key=value`, `key+=value` or `key@path`.
    ///
    /// `key+=value` adds to a list, and `key@path` reads the value from a
    /// file. It wins over `--values`.
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,

    /// Slots to set after `--`, written as for `--set`.
    ///
    /// For example `mt add "Fix login" -- tags+=auth summary@notes.txt`. They
    /// win over `--set`.
    #[arg(last = true, value_name = "KEY=VALUE")]
    pub tokens: Vec<String>,

    /// The directory to write the task in.
    ///
    /// A relative path resolves from the project root; an absolute one is
    /// taken as-is. Without it, an `[[on.add]]` hook may choose the directory,
    /// else mt writes to the first configured root.
    #[arg(long, value_name = "DIR")]
    pub dir: Option<String>,

    /// Removed. Use `--dir`.
    #[arg(long, hide = true, value_name = "DIR")]
    pub into: Option<String>,

    /// Removed. Use `--dir`.
    #[arg(long = "to", hide = true, value_name = "SUBDIR")]
    pub to: Option<String>,

    /// The priority, as a number.
    #[arg(long, value_name = "VALUE")]
    pub priority: Option<String>,

    /// The difficulty, as a number.
    #[arg(long, value_name = "VALUE")]
    pub difficulty: Option<String>,

    /// The kind of work, such as `feat` or `fix`.
    #[arg(long, value_name = "VALUE")]
    pub kind: Option<String>,

    /// Who is working on the task.
    #[arg(long, value_name = "NAME")]
    pub assignee: Option<String>,

    /// The due date, as YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    pub due: Option<String>,

    /// The start date, as YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    pub start: Option<String>,

    /// The file name, without `.typ`.
    ///
    /// Defaults to a slug of the title.
    #[arg(long, value_name = "STEM")]
    pub name: Option<String>,

    /// The status to file the task with.
    ///
    /// Defaults to the template's status, else your project's first status
    /// that doesn't close a task.
    #[arg(long, value_name = "NAME")]
    pub status: Option<String>,

    /// Fill the template's slots in your editor.
    ///
    /// Needs a terminal.

    #[arg(long)]
    pub edit: bool,

    /// Print the template's slots and the project's add hooks instead of
    /// filing a task.
    ///
    /// `--describe=json` prints the same list, hooks included, as JSON.
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, require_equals = true, default_missing_value = "text")]
    pub describe: Option<String>,

    /// List the templates and exit.
    #[arg(long)]
    pub list: bool,

    /// Print the filled task instead of writing it.
    #[arg(long)]
    pub print: bool,

    /// Don't check the new task against your project's rules.
    #[arg(long = "no-verify")]
    pub no_verify: bool,

    /// Don't run the project's `[[on.add]]` hooks.
    #[arg(long = "no-hooks")]
    pub no_hooks: bool,

    /// What to do when a task with that file name exists.
    ///
    /// Without it, mt follows your project's `[tasks.naming]` settings.

    #[arg(long, value_enum, value_name = "MODE")]
    pub dedup: Option<DedupArg>,

    /// How to report what `mt add` did.
    #[arg(long, value_enum, default_value_t)]
    pub format: Format,

    /// Same as `--format json`.
    #[arg(long)]
    pub json: bool,
}

/// The format this run reports in: `--json` is shorthand for `--format json`
/// and wins, otherwise `--format` (which has its own default).
fn effective_format(args: &AddArgs) -> Format {
    if args.json { Format::Json } else { args.format }
}

/// The fields with an explicit `=` from the caller: an append does not count,
/// since a later hook may still set the field.
fn explicit_sets(ops: &[(String, FieldOp)]) -> BTreeSet<String> {
    ops.iter()
        .filter(|(_, op)| matches!(op, FieldOp::Set(_)))
        .map(|(name, _)| name.clone())
        .collect()
}

/// `mt add --list`: every template stem, one per line.
fn print_templates(config: &Config, root: &Path) {
    for name in list_templates(config, root) {
        println!("{name}");
    }
}

/// `mt add --describe`: the template's slots and the project's add hooks.
fn print_describe(
    args: &AddArgs,
    config: &Config,
    root: &Path,
    source: &str,
    format: &str,
) -> Result<()> {
    hint::refuse_title_beside_describe(&args.title, config, root)?;
    let specs = discover_slots(source);
    println!(
        "{}",
        describe_for(config, &specs, DescribeFormat::parse(format)?)?
    );
    Ok(())
}

/// Run the `add` command.
///
/// # Errors
/// Returns an error if the template/values are invalid, a slot is unfilled, the
/// target identity collides, or the file cannot be written.
pub fn run(args: &AddArgs) -> Result<()> {
    refuse_legacy(args.into.as_ref(), args.to.as_ref())?;
    let project = util::open()?;
    let config = &project.config;

    if args.list {
        print_templates(config, &project.root);
        return Ok(());
    }

    let template_name = args
        .template
        .clone()
        .unwrap_or_else(|| config.templates.default.clone());
    let source = resolve_template(config, &project.root, &template_name)?;
    let status = filing_status(config, &source, args.status.as_deref());
    let (source, injected) = splice_config_slots(&source, config, status.as_deref())?;

    if let Some(format) = &args.describe {
        print_describe(args, config, &project.root, &source, format)?;
        return Ok(());
    }

    let ready =
        pipeline::collect_and_render(args, &project, &template_name, &source, status.as_deref())?;
    let filled = util::canonical_source(config, ready.filled);
    report_injected(
        &filled,
        &injected,
        &ready.explicit,
        &ready.env_keys,
        &ready.touched,
    );

    // Resolve the path the write would land at before the `--print` return:
    // it is the candidate the pre-write check validates, and a preview must
    // clear the same check, or a dry run stops being a preflight.
    let dest = resolve_dir(args, &project, ready.dir.as_deref())?;
    let stem = output_stem(args, &ready.values, config)?;
    let file = dest.join(format!("{stem}.typ"));

    let rescue = Rescue::new(&template_name, ready.interactive);
    let written = if args.print {
        print_preview(args, &project, &file, &filled, &rescue)
    } else {
        write_task(args, &project, (&stem, &dest, file), &filled, &rescue)
    };
    let context = hint::RefusalContext {
        config,
        root: &project.root,
        template: &template_name,
        sources: (&source, &filled),
        explicit: &ready.explicit,
    };
    written.map_err(|err| context.decorate(err, rescue.parked.take()))
}

/// Resolve the dedup outcome, then [`commit`] the task.
fn write_task(
    args: &AddArgs,
    project: &util::Project,
    (stem, dest, file): (&str, &Path, std::path::PathBuf),
    filled: &str,
    rescue: &Rescue,
) -> Result<()> {
    let outcome = resolve_outcome(args, project, stem, dest, file)?;
    let Some(outcome) = outcome else {
        // `--dedup skip` already reported the existing path; nothing to write.
        return Ok(());
    };
    commit(args, project, &outcome, filled, rescue)
}

/// Print the preview — but only after the same pre-write check a write runs.
/// A preview that exits 0 while the write it previews is refused is how the
/// sweep shipped a retired estimate unit: the dry run green, `--write` dead.
/// `--no-verify` opens both doors.
fn print_preview(
    args: &AddArgs,
    project: &util::Project,
    file: &Path,
    filled: &str,
    rescue: &Rescue,
) -> Result<()> {
    if !args.no_verify {
        pre_verify(project, file, filled, rescue)?;
    }
    print!("{filled}");
    Ok(())
}

/// Validate the rendered task, then write it — in that order, so a rejected add
/// creates nothing and the failure is about the input rather than about a file
/// to repair. The write is the last irreversible step: everything that can
/// still say no has already said it.
///
/// The pre-write check needs a project-relative path to evaluate the candidate
/// at, which `mt add --dir /somewhere/else` does not have. That case alone
/// falls back to the old order — write, then [`crate::task_edit::verify_written`] — and its error
/// still carries [`kept`], because there the file really is on disk.
fn commit(
    args: &AddArgs,
    project: &util::Project,
    outcome: &output::Outcome,
    filled: &str,
    rescue: &Rescue,
) -> Result<()> {
    let file = outcome.path();
    let checked = args.no_verify || pre_verify(project, file, filled, rescue)?;

    let dest = file.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dest).map_err(|err| Diag::cannot_write(dest, &err))?;
    // `create_new`, not `write_atomic`: the dedup check above only proves no
    // file was here a moment ago, and a plain create-or-truncate write would
    // silently clobber one that appeared since. `create_new` fails instead.
    File::create_new(file)
        .and_then(|mut handle| handle.write_all(filled.as_bytes()))
        .map_err(|err| Diag::cannot_write(file, &err))?;
    warn_if_outside_roots(project, dest);
    output::emit(
        outcome,
        &output::handle(project, outcome),
        effective_format(args),
    );

    if !checked {
        crate::task_edit::verify_written(&project.root, file).map_err(|err| kept(err, file))?;
    }
    Ok(())
}

/// [`crate::task_edit::verify_candidate`], with one thing added: when the
/// values came from an interactive `--edit` buffer, a rejection parks the
/// rendered task in a scratch file and names it.
///
/// The buffer is minutes of typing that the editor's own temp file no longer
/// holds, and refusing the write is exactly what makes it unrecoverable —
/// which is why the file used to be kept in the tasks root instead. Parking it
/// outside every root keeps the recovery without the cost: no later `mt`
/// command ever loads it. A `-f`/`--set` rejection parks nothing; the values
/// are still in the shell history that produced them.
fn pre_verify(project: &util::Project, file: &Path, source: &str, rescue: &Rescue) -> Result<bool> {
    let result = crate::task_edit::verify_candidate(project, file, source, &rescue.template);
    if result.is_err() && rescue.interactive {
        rescue.parked.set(park(source));
    }
    result
}

/// Write `source` to a kept scratch file, returning its path. `None` when the
/// scratch file cannot be created — a failed rescue must not replace the
/// validation error that prompted it.
fn park(source: &str) -> Option<std::path::PathBuf> {
    let mut file = tempfile::Builder::new()
        .prefix("mindtape-rejected-")
        .suffix(".typ")
        .tempfile()
        .ok()?;
    file.write_all(source.as_bytes()).ok()?;
    file.keep().ok().map(|(_, path)| path)
}

/// Escape every fill value that lands in Typst markup, leaving the rest alone.
///
/// Only a `text` slot is escaped (see [`mindtape::SlotKind::Text`]); every
/// other kind renders as a self-delimiting literal — a quoted string, a
/// number, a `("a",)` list, an `on(y, m, d)` call — where the escaping
/// backslashes would be data, not syntax, and would read back as themselves.
///
/// `raw` ([`mindtape::SlotKind::Raw`]) also splices its value verbatim
/// into markup, like `text`, but is deliberately left out of this escape
/// pass. The choice belongs to the *template*, not the *caller*: a template
/// is checked-in repo content, so `kind: "raw"` is the template author
/// asserting that whatever fills this slot — `--set`, `-f`, the environment,
/// none of which this function can tell apart — is trusted to be markup, not
/// prose that merely contains markup characters. Escaping is applied here,
/// downstream of every value source: `-f`/`--set`, the positional title, the
/// `MINDTAPE_ADD_*` environment, and the `--edit` buffer all converge on this
/// map, so the trust decision is enforced in exactly one place.
///
/// The escaped copy feeds `mindtape::fill` only; the caller keeps the authored
/// values so
/// the filename slug is minted from the text the user actually typed.
fn markup_safe(
    values: &BTreeMap<String, RawValue>,
    specs: &[SlotSpec],
) -> BTreeMap<String, RawValue> {
    let mut out = values.clone();
    for spec in specs.iter().filter(|spec| spec.kind == SlotKind::Text) {
        if let Some(RawValue::Str(text)) = out.get_mut(&spec.name) {
            *text = escape_markup(text);
        }
    }
    out
}

/// What a failed post-write verification says about the file already on disk.
///
/// Only [`commit`]'s fallback arm reaches this: a destination outside the
/// project root, where the candidate cannot be evaluated before the write and
/// a rejected task is therefore a real file by the time the error surfaces. It
/// is kept rather than removed — silently deleting a file to tidy up an error
/// message is the worse trade — but "wrote X" followed by a bare error read as
/// if `add` had cleaned up after itself. It never did; now it says so.
///
/// Under a root the order is the other way round ([`pre_verify`]) and no such
/// file exists, so this note is never printed there.
fn kept(err: Diag, file: &Path) -> Diag {
    let diag = err.note(
        Note::new("the task was kept on disk exactly as written")
            .value(mindtape::display_path(file)),
    );
    if diag.has_hint_room() {
        diag.hint(Hint::new("fix or delete it before the next command"))
    } else {
        diag
    }
}

#[cfg(test)]
mod tests;
