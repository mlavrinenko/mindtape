//! `mt add` — file a new task from a Typst slot template.
//!
//! Resolves a template, fills its `slot(...)` calls from JSON/`--set` values via
//! the parse-plane engine in [`mindtape`], injects the add-owned `status`/`id`
//! into `task.with(…)`, then validates the result and — only if it passes —
//! writes it (or prints it, under `--print`). The render and the write have
//! always been separable; validation sits between them, so a rejected add
//! leaves nothing on disk for the next command to trip over.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{IsTerminal, Read, Write as _};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use clap::Args;
use mindtape::{
    Config, RawValue, RequiredSlot, SlotKind, SlotSpec, discover_slots, escape_markup, fill,
    status_value, today_ymd,
};
use typst_edit::apply;

use crate::add_dedup::DedupArg;
use crate::add_interactive;
use crate::add_output::{self, Format};
use crate::add_tokens::append_list_value;
use crate::add_values::{
    DescribeFormat, collect_values, describe, layer_env, list_templates, reject_unknown_keys,
    resolve_template, stdin_title,
};
use crate::task_edit::{filing_status, initial_status, task_with, upsert_named_arg};
use crate::util;

#[path = "add_target.rs"]
mod target;
pub(crate) use target::minted_stem;
pub(crate) use target::{dest_dir, warn_if_outside_roots};
use target::{output_stem, resolve_outcome};

#[path = "add_notes.rs"]
mod add_notes;
use add_notes::report_injected;

#[path = "add_hint.rs"]
mod add_hint;

/// Arguments for `mt add`.
// A clap flag struct: each `--flag` is an independent boolean toggle.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Task title words, joined with spaces to seed the `title` slot.
    #[arg(value_name = "TITLE")]
    pub title: Vec<String>,

    /// Template stem (default: the configured `[templates] default`).
    #[arg(long = "template", short = 't', value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// JSON or YAML values file (repeatable, merged left-to-right; `-` reads
    /// stdin). `.json`/`.yaml`/`.yml` pick the parser by extension; stdin and
    /// any other extension sniff the content.
    #[arg(long = "values", short = 'f', value_name = "PATH")]
    pub values: Vec<String>,

    /// Override a slot: `key=value` sets, `key+=value` appends to a list slot
    /// (e.g. `--set tags+=a --set tags+=b`), `key@path` reads the value out of
    /// a file. Applied after `--values`.
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,

    /// Slot assignments after `--`, in the same `key=value` / `key+=value` /
    /// `key@path` grammar as `--set` (e.g. `mt add "title" --
    /// estimate.files=2 tags+=a summary@notes.txt`). Applied after `--set`.
    #[arg(last = true, value_name = "KEY=VALUE")]
    pub tokens: Vec<String>,

    /// Destination directory; `.` (default) is the first configured root.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub into: String,

    /// Subfolder under the destination root to nest the task into (folder-derived
    /// tags). Relative to `--into`; `--to shopping` → `<root>/shopping`.
    #[arg(long = "to", value_name = "SUBDIR")]
    pub to: Option<String>,

    /// Output filename stem; default is a slug of the title value.
    #[arg(long, value_name = "STEM")]
    pub name: Option<String>,

    /// Initial status name; default is the first non-terminal legend value.
    #[arg(long, value_name = "NAME")]
    pub status: Option<String>,

    /// Open an editor to fill the template's slots (requires a terminal).
    #[arg(long)]
    pub edit: bool,

    /// Print the template's slot schema instead of filing. `--describe=json` for
    /// machine output.
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, require_equals = true, default_missing_value = "text")]
    pub describe: Option<String>,

    /// List available templates and exit.
    #[arg(long)]
    pub list: bool,

    /// Print the filled task to stdout instead of writing a file.
    #[arg(long)]
    pub print: bool,

    /// Skip post-write validation of the new task.
    #[arg(long = "no-verify")]
    pub no_verify: bool,

    /// Collision policy when the destination stem already exists: `fail`
    /// (bail), `skip` (exit 0, write nothing, report the existing path), or
    /// `add-suffix` (mint `-2`, `-3`, … via `naming::disambiguate`). Applies
    /// the same whether the stem was minted from a `[tasks.naming] stem`
    /// template or is bare. Overrides `[tasks.naming] dedup`, whose unset
    /// default is `fail` for a bare stem and `add-suffix` for a templated one
    /// (see `mindtape::model::naming::NamingConfig::effective_dedup`).
    #[arg(long, value_enum, value_name = "MODE")]
    pub dedup: Option<DedupArg>,

    /// Output format: `text` (today's `wrote <path>` prose, the default),
    /// `path`, `path0`, or `json` — the JSON object reports the outcome
    /// (`wrote`, `skipped`, `suffixed`), not only the path.
    #[arg(long, value_enum, default_value_t)]
    pub format: Format,
}

/// Run the `add` command.
///
/// # Errors
/// Returns an error if the template/values are invalid, a slot is unfilled, the
/// target identity collides, or the file cannot be written.
pub fn run(args: &AddArgs) -> Result<()> {
    let project = util::open()?;
    let config = &project.config;

    if args.list {
        for name in list_templates(config, &project.root) {
            println!("{name}");
        }
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
        add_hint::refuse_title_beside_describe(&args.title, config, &project.root)?;
        let specs = discover_slots(&source);
        println!("{}", describe(&specs, DescribeFormat::parse(format)?)?);
        return Ok(());
    }

    let mut values = collect_values(&args.values, &args.set, &args.tokens)?;
    let explicit: BTreeSet<String> = values.keys().cloned().collect();
    let specs = discover_slots(&source);
    reject_unknown_keys(&values, &specs)?;
    fill_title_from_args(&mut values, args, &specs);
    let before_env: BTreeSet<String> = values.keys().cloned().collect();
    layer_env(&mut values, &specs, |key| std::env::var(key).ok());
    let env_keys: BTreeSet<String> = values
        .keys()
        .filter(|key| !before_env.contains(*key))
        .cloned()
        .collect();
    fill_title_from_stdin(&mut values, args)?;
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let interactive = add_interactive::should_open(args.edit, &config.add, &specs, &values, tty)?;
    if interactive {
        values = add_interactive::run(&config.add, &specs, &values)?;
    }
    let filled =
        fill(&source, &markup_safe(&values, &specs)).map_err(|err| anyhow!(err.to_string()))?;
    let filled = inject_header(&filled, config, status.as_deref())?;
    let filled = util::canonical_source(config, filled);
    report_injected(&filled, &injected, &explicit, &env_keys)?;

    // Resolve the path the write would land at before the `--print` return:
    // it is the candidate the pre-write check validates, and a preview must
    // clear the same check, or a dry run stops being a preflight.
    let dest = dest_dir(&args.into, args.to.as_deref(), &project)?;
    let stem = output_stem(args, &values, config)?;
    let file = dest.join(format!("{stem}.typ"));

    let written = if args.print {
        print_preview(args, &project, &file, &filled, interactive)
    } else {
        write_task(args, &project, (&stem, &dest, file), &filled, interactive)
    };
    written.map_err(|err| {
        let Some(refused) = add_hint::refused_fields(&err) else {
            return err;
        };
        match add_hint::template_hint(
            config,
            &project.root,
            &template_name,
            (&source, &filled),
            (&explicit, refused.as_ref()),
        ) {
            Some(hint) => anyhow!("{err:#}\n{hint}"),
            None => err,
        }
    })
}

/// Resolve the dedup outcome, then [`commit`] the task.
fn write_task(
    args: &AddArgs,
    project: &util::Project,
    (stem, dest, file): (&str, &Path, std::path::PathBuf),
    filled: &str,
    interactive: bool,
) -> Result<()> {
    let outcome = resolve_outcome(args, &project.config, stem, dest, file)?;
    let Some(outcome) = outcome else {
        // `--dedup skip` already reported the existing path; nothing to write.
        return Ok(());
    };
    commit(args, project, &outcome, filled, interactive)
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
    interactive: bool,
) -> Result<()> {
    if !args.no_verify {
        pre_verify(project, file, filled, interactive)?;
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
/// at, which `mt add --into /somewhere/else` does not have. That case alone
/// falls back to the old order — write, then [`verify_written`] — and its error
/// still carries [`kept_note`], because there the file really is on disk.
fn commit(
    args: &AddArgs,
    project: &util::Project,
    outcome: &add_output::Outcome,
    filled: &str,
    interactive: bool,
) -> Result<()> {
    let file = outcome.path();
    let checked = args.no_verify || pre_verify(project, file, filled, interactive)?;

    let dest = file.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dest).with_context(|| format!("create {}", dest.display()))?;
    // `create_new`, not `write_atomic`: the dedup check above only proves no
    // file was here a moment ago, and a plain create-or-truncate write would
    // silently clobber one that appeared since. `create_new` fails instead.
    File::create_new(file)
        .and_then(|mut handle| handle.write_all(filled.as_bytes()))
        .with_context(|| format!("write {}", file.display()))?;
    warn_if_outside_roots(project, dest);
    add_output::emit(outcome, args.format);

    if !checked {
        crate::task_edit::verify_written(&project.root, file).with_context(|| kept_note(file))?;
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
fn pre_verify(
    project: &util::Project,
    file: &Path,
    source: &str,
    interactive: bool,
) -> Result<bool> {
    let result = crate::task_edit::verify_candidate(project, file, source);
    if result.is_err()
        && interactive
        && let Some(parked) = park(source)
    {
        eprintln!("note: the filled task was parked at {}", parked.display());
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
/// The escaped copy feeds [`fill`] only; the caller keeps the authored values so
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
fn kept_note(file: &Path) -> String {
    format!(
        "{} was kept on disk exactly as written; fix or delete it before the next command",
        file.display()
    )
}

/// Splice slot-bearing values for config-required core fields the template
/// omits, so their slots surface in the fill scaffold, `--describe`, and the
/// written task. A field already set by the template (literal or slot) is left
/// untouched, except that a slot which is its whole value inherits the field's
/// config `values` as choices ([`mindtape::inherit_field_choices`]). Runs
/// before discovery, so a spliced value may itself nest slots — a
/// strict-framework priority splices `framework("ice", impact: slot(…), …)`.
///
/// Returns the fields actually spliced (the template omitted them), so the
/// caller can report the values it filled that the template never mentioned.
pub(crate) fn splice_config_slots(
    source: &str,
    config: &Config,
    status: Option<&str>,
) -> Result<(String, Vec<RequiredSlot>)> {
    let source =
        mindtape::inherit_field_choices(source, config).map_err(|err| anyhow!(err.to_string()))?;
    let call = task_with(&source)?;
    let mut edits = Vec::new();
    let mut injected = Vec::new();
    for slot in mindtape::required_core_slots(config, status) {
        if call.named(&slot.field).is_none() {
            edits.push(upsert_named_arg(&source, &call, &slot.field, &slot.call)?);
            injected.push(slot);
        }
    }
    let spliced = apply(&source, edits).map_err(|err| anyhow!(err.to_string()))?;
    Ok((spliced, injected))
}

/// Inject the add-owned `status` (and `id` when the config generates one) into
/// `task.with(…)`.
pub(crate) fn inject_header(
    source: &str,
    config: &Config,
    status_override: Option<&str>,
) -> Result<String> {
    let call = task_with(source)?;
    let mut edits = Vec::new();

    let status_name = status_override
        .map(ToOwned::to_owned)
        .or_else(|| initial_status(config));
    if let Some(status_name) = status_name {
        let value = status_value(&status_name, today_ymd(), None);
        edits.push(upsert_named_arg(source, &call, "status", &value)?);
    }

    if let Some(id) = config.tasks.id.format.generate() {
        edits.push(upsert_named_arg(source, &call, "id", &format!("\"{id}\""))?);
    }

    apply(source, edits).map_err(|err| anyhow!(err.to_string()))
}

/// Seed the `title` slot from the positional `TITLE` words, joined with spaces.
/// A higher-precedence `title` (`-f`/`--set`) wins; blank input sets nothing.
///
/// When the template declares a `tags` slot, a leading `+word` token (the
/// Taskwarrior/todo.txt tag sigil) is peeled out of the title into `tags`
/// instead — `mt add fix login +backend` sets `title: "fix login"`,
/// `tags: ("backend",)`. Without a `tags` slot the words are left untouched, so
/// a `+`-prefixed word never silently vanishes.
fn fill_title_from_args(
    values: &mut BTreeMap<String, RawValue>,
    args: &AddArgs,
    specs: &[SlotSpec],
) {
    let has_tags_slot = specs.iter().any(|spec| spec.name == "tags");
    let (title_words, tags) = if has_tags_slot {
        split_tag_sugar(&args.title)
    } else {
        (args.title.clone(), Vec::new())
    };
    for tag in tags {
        append_list_value(values, "tags", tag);
    }

    if values.contains_key("title") || title_words.is_empty() {
        return;
    }
    let title = title_words.join(" ");
    if !title.trim().is_empty() {
        values.insert("title".to_owned(), RawValue::Str(title));
    }
}

/// Split `+tag` words out of the positional title, in order. A bare `+` (no
/// tag text) is kept as a title word rather than treated as an empty tag.
fn split_tag_sugar(words: &[String]) -> (Vec<String>, Vec<String>) {
    let mut title_words = Vec::new();
    let mut tags = Vec::new();
    for word in words {
        match word.strip_prefix('+') {
            Some(tag) if !tag.is_empty() => tags.push(tag.to_owned()),
            _ => title_words.push(word.clone()),
        }
    }
    (title_words, tags)
}

/// Read a piped title from stdin into the `title` slot. Gated to the
/// non-interactive pipe case: a TTY (no pipe) would block, and `-f -` already
/// claims stdin as JSON. A higher-precedence `title` (`--set`/`-f`/env) wins.
fn fill_title_from_stdin(values: &mut BTreeMap<String, RawValue>, args: &AddArgs) -> Result<()> {
    if values.contains_key("title")
        || std::io::stdin().is_terminal()
        || args.values.iter().any(|source| source == "-")
    {
        return Ok(());
    }
    let mut raw = String::new();
    std::io::stdin()
        .read_to_string(&mut raw)
        .context("read title from stdin")?;
    if let Some(title) = stdin_title(&raw) {
        values.insert("title".to_owned(), RawValue::Str(title));
    }
    Ok(())
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;
