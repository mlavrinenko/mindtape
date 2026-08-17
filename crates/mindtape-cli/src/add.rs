//! `mt add` — file a new task from a Typst slot template.
//!
//! Resolves a template, fills its `slot(...)` calls from JSON/`--set` values via
//! the parse-plane engine in [`mindtape`], injects the add-owned `status`/`id`
//! into `task.with(…)`, then writes (or prints) the result and validates it.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use clap::Args;
use mindtape::{
    Config, RawValue, SlotKind, SlotSpec, discover_slots, escape_markup, fill, status_value,
    today_ymd,
};
use typst_edit::apply;

use crate::add_dedup::DedupArg;
use crate::add_interactive;
use crate::add_output::{self, Format};
use crate::add_values::{
    DescribeFormat, append_list_value, collect_values, describe, layer_env, list_templates,
    reject_unknown_keys, resolve_template, stdin_title,
};
use crate::task_edit::{initial_status, task_with, upsert_named_arg};
use crate::util;

#[path = "add_target.rs"]
mod target;
pub(crate) use target::{dest_dir, warn_if_outside_roots};
use target::{output_stem, resolve_outcome};

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
    /// (e.g. `--set tags+=a --set tags+=b`). Applied after `--values`.
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,

    /// Slot assignments after `--`, in the same `key=value` / `key+=value`
    /// grammar as `--set` (e.g. `mt add "title" -- estimate.files=2 tags+=a`).
    /// Applied after `--set`.
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
    /// (see `mindtape::naming::NamingConfig::effective_dedup`).
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
    let source = inject_required_slots(&source, config)?;

    if let Some(format) = &args.describe {
        let specs = discover_slots(&source);
        println!("{}", describe(&specs, DescribeFormat::parse(format)?)?);
        return Ok(());
    }

    let mut values = collect_values(&args.values, &args.set, &args.tokens)?;
    let specs = discover_slots(&source);
    reject_unknown_keys(&values, &specs)?;
    fill_title_from_args(&mut values, args, &specs);
    layer_env(&mut values, &specs, |key| std::env::var(key).ok());
    fill_title_from_stdin(&mut values, args)?;
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    if add_interactive::should_open(args.edit, &config.add, &specs, &values, tty)? {
        values = add_interactive::run(&config.add, &specs, &values)?;
    }
    let filled =
        fill(&source, &markup_safe(&values, &specs)).map_err(|err| anyhow!(err.to_string()))?;
    let filled = inject_header(&filled, config, args.status.as_deref())?;
    let filled = util::canonical_source(config, filled);

    if args.print {
        print!("{filled}");
        return Ok(());
    }

    let dest = dest_dir(&args.into, args.to.as_deref(), &project)?;
    let stem = output_stem(args, &values, config)?;
    let file = dest.join(format!("{stem}.typ"));
    let outcome = resolve_outcome(args, config, &stem, &dest, file)?;
    let Some(outcome) = outcome else {
        // `--dedup skip` already reported the existing path; nothing to write.
        return Ok(());
    };
    let file = outcome.path().to_path_buf();

    std::fs::create_dir_all(&dest).with_context(|| format!("create {}", dest.display()))?;
    std::fs::write(&file, &filled).with_context(|| format!("write {}", file.display()))?;
    warn_if_outside_roots(&project, &dest);
    add_output::emit(&outcome, args.format);

    if !args.no_verify {
        crate::task_edit::verify_written(&project.root, &file).with_context(|| kept_note(&file))?;
    }
    Ok(())
}

/// Escape every fill value that lands in Typst markup, leaving the rest alone.
///
/// Only a `text` slot is escaped (see [`mindtape::SlotKind::Text`]); every
/// other kind renders as a self-delimiting literal — a quoted string, a
/// number, a `("a",)` list, an `on(y, m, d)` call — where the escaping
/// backslashes would be data, not syntax, and would read back as themselves.
///
/// `typst` ([`mindtape::SlotKind::Typst`]) also splices its value verbatim
/// into markup, like `text`, but is deliberately left out of this escape
/// pass. The choice belongs to the *template*, not the *caller*: a template
/// is checked-in repo content, so `kind: "typst"` is the template author
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
/// `add` writes before it verifies, so a rejected task is a real file by the
/// time the error surfaces. It is kept rather than removed — a `--edit` buffer
/// can hold minutes of typing, and silently deleting it to tidy up an error
/// message is the worse trade — but "wrote X" followed by a bare error read as
/// if `add` had cleaned up after itself. It never did; now it says so.
fn kept_note(file: &Path) -> String {
    format!(
        "{} was kept on disk exactly as written; fix or delete it before the next command",
        file.display()
    )
}

/// Splice slot-bearing values for config-required core fields the template
/// omits, so their slots surface in the fill scaffold, `--describe`, and the
/// written task. A field already set by the template (literal or slot) is left
/// untouched. Runs before discovery, so a spliced value may itself nest slots —
/// a strict-framework priority splices `framework("ice", impact: slot(…), …)`.
fn inject_required_slots(source: &str, config: &Config) -> Result<String> {
    let needed = mindtape::required_core_slots(config);
    if needed.is_empty() {
        return Ok(source.to_owned());
    }
    let call = task_with(source)?;
    let mut edits = Vec::new();
    for slot in &needed {
        if call.named(slot.field).is_none() {
            edits.push(upsert_named_arg(source, &call, slot.field, &slot.call)?);
        }
    }
    if edits.is_empty() {
        return Ok(source.to_owned());
    }
    apply(source, edits).map_err(|err| anyhow!(err.to_string()))
}

/// Inject the add-owned `status` (and `id` when the config generates one) into
/// `task.with(…)`.
fn inject_header(source: &str, config: &Config, status_override: Option<&str>) -> Result<String> {
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
