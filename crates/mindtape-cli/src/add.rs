//! `mt add` — file a new task from a Typst slot template.
//!
//! Resolves a template, fills its `slot(...)` calls from JSON/`--set` values via
//! the parse-plane engine in [`mindtape`], injects the add-owned `status`/`id`
//! into `task.with(…)`, then writes (or prints) the result and validates it.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::Args;
use mindtape::{
    Config, RawValue, SlotSpec, discover_slots, fill, load_config, load_project, status_value,
    today_ymd, validate,
};
use typst_edit::apply;

use crate::add_interactive;
use crate::add_values::{
    DescribeFormat, append_list_value, collect_values, describe, layer_env, list_templates,
    resolve_template, slug, stdin_title,
};
use crate::task_edit::{task_with, upsert_named_arg};
use crate::util;

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

    /// JSON values file (repeatable, merged left-to-right; `-` reads stdin).
    #[arg(long = "values", short = 'f', value_name = "PATH")]
    pub values: Vec<String>,

    /// Override a slot: `key=value` (string; applied after `--values`). Repeat
    /// `key[]=value` to append to a list slot (e.g. `tags[]=a --set tags[]=b`).
    #[arg(long = "set", value_name = "KEY=VALUE")]
    pub set: Vec<String>,

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

    let mut values = collect_values(&args.values, &args.set)?;
    let specs = discover_slots(&source);
    fill_title_from_args(&mut values, args, &specs);
    layer_env(&mut values, &specs, |key| std::env::var(key).ok());
    fill_title_from_stdin(&mut values, args)?;
    if add_interactive::should_open(args.edit, &config.add, &specs, &values)? {
        values = add_interactive::run(&config.add, &specs, &values)?;
    }
    let filled = fill(&source, &values).map_err(|err| anyhow!(err.to_string()))?;
    let filled = inject_header(&filled, config, args.status.as_deref())?;

    if args.print {
        print!("{filled}");
        return Ok(());
    }

    let stem = output_stem(args, &values)?;
    let dest = dest_dir(&args.into, args.to.as_deref(), &project)?;
    let file = dest.join(format!("{stem}.typ"));
    if file.exists() {
        bail!("task {stem:?} already exists at {}", file.display());
    }
    std::fs::create_dir_all(&dest).with_context(|| format!("create {}", dest.display()))?;
    std::fs::write(&file, &filled).with_context(|| format!("write {}", file.display()))?;
    warn_if_outside_roots(&project, &dest);
    println!("wrote {}", file.display());

    if !args.no_verify {
        verify(&project.root, &file)?;
    }
    Ok(())
}

/// Splice `slot(...)` calls for config-required core fields the template omits,
/// so they surface in the fill scaffold, `--describe`, and the written task. A
/// field already set by the template (literal or slot) is left untouched.
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

/// The status name to stamp on a freshly filed task: the legend's own initial
/// (first non-terminal) value, falling back to the legend's first entry when
/// every entry happens to be terminal. `None` only when the legend is
/// genuinely empty (a project that cleared it with `status = []`) — the
/// caller must then leave the status slot unfilled and let validation report
/// the real problem, rather than inventing a value.
fn initial_status(config: &Config) -> Option<String> {
    let legend = &config.tasks.status;
    legend
        .initial()
        .or_else(|| legend.entries.first())
        .map(|entry| entry.value.clone())
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

fn output_stem(args: &AddArgs, values: &BTreeMap<String, RawValue>) -> Result<String> {
    if let Some(name) = &args.name {
        return Ok(name.clone());
    }
    if let Some(RawValue::Str(title)) = values.get("title") {
        let stem = slug(title);
        if !stem.is_empty() {
            return Ok(stem);
        }
    }
    bail!("need --name or a 'title' value to name the task file")
}

fn dest_dir(into: &str, to: Option<&str>, project: &util::Project) -> Result<PathBuf> {
    let base = base_dir(into, project)?;
    let Some(sub) = to else {
        return Ok(base);
    };
    let sub = Path::new(sub);
    if sub.is_absolute() {
        bail!("--to expects a subfolder relative to the destination root, got absolute {sub:?}");
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
    Ok(std::env::current_dir()
        .context("determine current directory")?
        .join(path))
}

/// Warn when `dest` lands outside every configured root, since recursive loading
/// only sees files under a root.
fn warn_if_outside_roots(project: &util::Project, dest: &Path) {
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
        eprintln!(
            "warning: {} is outside the configured roots; the task will not be tracked",
            dest.display()
        );
    }
}

/// Reload the project and report any load error or validation violation that
/// touches the newly written file.
fn verify(root: &Path, file: &Path) -> Result<()> {
    let config = load_config(root)?;
    let outcome = load_project(root, &config);
    let rel = file.strip_prefix(root).unwrap_or(file);

    if let Some(error) = outcome.errors.iter().find(|err| err.file == rel) {
        bail!("written task failed to load: {}", error.message);
    }
    let Some(task) = outcome.tasks.iter().find(|task| task.file == rel) else {
        bail!(
            "written task at {} did not harvest a task marker",
            rel.display()
        );
    };

    let issues: Vec<_> = validate(&outcome.tasks, &config)
        .into_iter()
        .filter(|violation| violation.task_id == task.id)
        .collect();
    if issues.is_empty() {
        return Ok(());
    }
    for violation in &issues {
        eprintln!(
            "warning: {}: {}: {}",
            task.id, violation.field, violation.message
        );
    }
    bail!("{} validation issue(s) in {}", issues.len(), rel.display())
}

#[cfg(test)]
#[path = "add_tests.rs"]
mod tests;
