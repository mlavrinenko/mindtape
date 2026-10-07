//! `mt init` — project scaffold, and the library install `mt install` shares.
//!
//! `init` writes one of the named presets in [`crate::cmd::init::preset`]. Two axes, kept
//! independent: `--preset` picks what the config says, `--annotate` picks
//! whether it says it with the reasoning attached. They were conflated before —
//! a `--full` flag meant both "the elaborate shape" and "the commented
//! rendering" — which left no way to get a documented copy of a simple shape.
//!
//! The scaffold's first line is a `#:schema` comment directive so a
//! schema-aware TOML editor (tombi, Taplo) assists filling `.mindtape/
//! config.toml`. It points, by default, at the schema `mt install` writes under
//! the XDG config dir; `--schema URL|PATH` overrides it (e.g. the published
//! URL for a portable, checked-in config), `--no-schema` drops it.
//!
//! # Files
//!
//! - `compose.rs` — folds the wizard's axis fragments into one config body.
//! - `preset.rs` — the named starter shapes.
//! - `wizard/` — the questionnaire over the configuration axes.
//! - `presets/` — the built-in preset directories, one folder each.
//! - `axes/` — the axis fragments the wizard composes.
//! - `cmd/ls/interactive/tests.rs`, `*_tests.rs` — unit tests.

pub(crate) mod compose;
pub(crate) mod preset;
pub(crate) mod wizard;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use self::preset::Preset;
use self::wizard::{Estimate, Headers, Organisation, Priority, Recurrence, Required, Workflow};

/// The Typst package version `mt install` installs. Mirrors `lib/typst.toml`.
const TYPST_PKG_VERSION: &str = "0.2.0";
const PRELUDE: &str = include_str!("../../../../../lib/prelude.typ");

/// The `#:schema` directive controls, grouped so they travel together — the
/// two are one decision, and splitting them across the parent's flag list
/// makes it read like four unrelated toggles.
#[derive(Debug, clap::Args)]
pub struct SchemaArgs {
    /// Value for the top `#:schema` directive: a URL, a path relative to the
    /// config file, or `local` for a `file://` URL to the schema `mt install`
    /// writes under the XDG config dir. Defaults to the published URL, which is
    /// the only value that survives someone else cloning the repo.
    #[arg(long, value_name = "URL|PATH|local")]
    pub schema: Option<String>,

    /// Omit the `#:schema` comment directive from the scaffold.
    #[arg(long)]
    pub no_schema: bool,
}

/// Arguments for `init`.
#[derive(Debug, clap::Args)]
#[allow(clippy::struct_excessive_bools)] // independent, orthogonal switches
pub struct InitArgs {
    /// Starter shape to write: a built-in name, a user preset under
    /// `mindtape/presets` in the config directory, or a path to a config or preset directory.
    #[arg(long, short = 'p', value_name = "NAME|PATH")]
    pub preset: Option<String>,

    /// Keep the preset's explanatory comments in the written config.
    #[arg(long)]
    pub annotate: bool,

    /// List the built-in presets and exit.
    #[arg(long, conflicts_with_all = ["preset", "annotate", "force"])]
    pub list_presets: bool,

    /// Overwrite an existing `.mindtape/config.toml`.
    #[arg(long)]
    pub force: bool,

    /// Never prompt: fall through to the default preset (or compose from the
    /// axis flags below) without asking anything.
    #[arg(long)]
    pub no_input: bool,

    /// Workflow axis: pre-answer the status-legend question (suppresses it).
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub workflow: Option<Workflow>,

    /// Priority axis: pre-answer the ranking question (suppresses it).
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub priority: Option<Priority>,

    /// Organisation axis: pre-answer the folder-derived-tags question.
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub organisation: Option<Organisation>,

    /// Estimate axis: pre-answer the units question (suppresses it).
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub estimate: Option<Estimate>,

    /// Required-fields axis: pre-answer the required-fields question.
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub required: Option<Required>,

    /// Body-headers axis: pre-answer the required-sections question.
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub headers: Option<Headers>,

    /// Recurrence axis: pre-answer the recurrence-syntax question.
    #[arg(long, value_enum, conflicts_with = "preset")]
    pub recurrence: Option<Recurrence>,

    /// `#:schema` directive controls.
    #[command(flatten)]
    pub schema: SchemaArgs,
}

/// Run the `init` command — scaffold `.mindtape/config.toml` in the current directory.
///
/// # Errors
/// Returns an error if the preset cannot be resolved or the config cannot be
/// written.
pub fn run(args: &InitArgs) -> Result<()> {
    if args.list_presets {
        print_preset_list();
        return Ok(());
    }
    if let Some(name) = args.preset.clone() {
        // An explicit preset never prompts and ignores the axis flags.
        return write_preset(&name, args, true);
    }
    let flags = wizard_flags(args);
    let tty = both_terminals(
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    );
    let interactive = run_interactively(args.no_input, tty);
    if interactive || flags.any() {
        // A terminal walks the axes; a non-terminal run with any flag composes
        // from the flags plus defaults, without prompting.
        return run_wizard(args, &flags, interactive);
    }
    // No preset, no flags, and no terminal (or `--no-input`): write the default
    // preset without asking anything — the non-interactive baseline.
    write_preset(preset::DEFAULT_PRESET, args, false)
}

/// Whether both standard streams are terminals. A prompt needs a tty on each
/// side: a terminal on stdin with stdout piped still cannot hold a
/// conversation, and neither can the reverse.
fn both_terminals(stdin_tty: bool, stdout_tty: bool) -> bool {
    stdin_tty && stdout_tty
}

/// Whether to walk the wizard: a terminal, unless `--no-input` forbids it.
fn run_interactively(no_input: bool, tty: bool) -> bool {
    !no_input && tty
}

/// Collect the per-axis flag values off `InitArgs`.
fn wizard_flags(args: &InitArgs) -> wizard::Flags {
    wizard::Flags {
        workflow: args.workflow,
        priority: args.priority,
        organisation: args.organisation,
        estimate: args.estimate,
        required: args.required,
        headers: args.headers,
        recurrence: args.recurrence,
    }
}

/// Resolve, render, and write a named preset — the shared path for `--preset X`
/// and for the non-interactive default fall-through. `explicit` suppresses the
/// "other starter shapes" hint that only helps someone who did not pick one.
fn write_preset(name: &str, args: &InitArgs, explicit: bool) -> Result<()> {
    let preset = preset::resolve(name, preset::user_preset_dir().as_deref())?;
    let text = with_schema_directive(&render_preset(&preset, args.annotate)?, &args.schema)?;
    let root = Path::new(".");
    if !scaffold_config(root, args.force, &text)? {
        return Ok(());
    }
    write_templates(root, &preset)?;
    println!("preset: {}", preset.name);
    if !explicit {
        println!("run `mt init --list-presets` to see the other starter shapes");
    }
    Ok(())
}

/// Render a preset's config body for writing.
///
/// An ordinary preset renders as before: annotated verbatim, or comment-stripped.
/// A preset carrying a top-level `[overlay]` is composed instead — its base and
/// overlay run through [`crate::cmd::init::compose::compose`], which deep-merges the
/// overlay last and flattens the result to one file the loader can read (no
/// literal `[overlay]` survives). That composed form is already comment-free, so
/// `--annotate` has nothing left to keep.
fn render_preset(preset: &Preset, annotate: bool) -> Result<String> {
    match preset::split_overlay(&preset.config)? {
        Some((base, overlay)) => crate::cmd::init::compose::compose(&[
            ("preset", base),
            (crate::cmd::init::compose::OVERLAY_AXIS, overlay),
        ]),
        None => Ok(preset.render(annotate)),
    }
}

/// Walk the axes (or compose from flags when non-interactive), then scaffold the
/// composed config and any templates the answers require.
fn run_wizard(args: &InitArgs, flags: &wizard::Flags, interactive: bool) -> Result<()> {
    let answers = {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut writer = std::io::stdout();
        wizard::resolve(flags, interactive, &mut reader, &mut writer)?
    };
    let outcome = wizard::assemble(answers)?;
    let text = with_schema_directive(&outcome.config, &args.schema)?;
    let root = Path::new(".");
    if !scaffold_config(root, args.force, &text)? {
        return Ok(());
    }
    // Reuse the preset template writer for any synthesised templates.
    let bundle = Preset {
        name: "custom".to_owned(),
        config: String::new(),
        templates: outcome.templates,
    };
    write_templates(root, &bundle)?;
    println!("scaffolded a custom configuration");
    Ok(())
}

/// Print one line per built-in preset, marking the default.
fn print_preset_list() {
    println!("built-in presets (`mt init --preset <name>`):");
    for (name, description) in preset::list_builtins() {
        let marker = if name == preset::DEFAULT_PRESET {
            " (default)"
        } else {
            ""
        };
        println!("  {name:<10} {description}{marker}");
    }
    if let Some(dir) = preset::user_preset_dir() {
        println!("\nuser presets live in {} and shadow these.", dir.display());
    }
}

/// Prepend the `#:schema` comment directive to the scaffold, unless
/// `--no-schema`.
///
/// Defaults to the published [`SCHEMA_URL`]. `.mindtape/config.toml` is a
/// checked-in file, and the previous default — a `file://` URL into the running
/// user's XDG dir — was meaningless to anyone who cloned the repo, so every
/// scaffold started life needing a hand edit. An editor fetches and caches the
/// URL, so this needs no local install either.
///
/// `--schema local` opts back into the XDG path for offline use or a project
/// that is never shared; that file may not exist yet, so a hint nudges the user
/// to run `mt install`.
fn with_schema_directive(template: &str, args: &SchemaArgs) -> Result<String> {
    if args.no_schema {
        return Ok(template.to_owned());
    }
    let value = match args.schema.as_deref() {
        Some("local") => {
            let path = crate::cmd::install::xdg_schema_path()?;
            if !path.exists() {
                println!(
                    "note: schema not installed at {} yet — run `mt install` (the \
                     `#:schema` directive points there)",
                    path.display()
                );
            }
            format!("file://{}", path.display())
        }
        Some(value) => value.to_owned(),
        None => mindtape::SCHEMA_URL.to_owned(),
    };
    Ok(format!("#:schema {value}\n{template}"))
}

pub(crate) fn install_lib() -> Result<()> {
    let dir = package_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let manifest = format!(
        "[package]\nname = \"mindtape\"\nversion = \"{TYPST_PKG_VERSION}\"\nentrypoint = \"prelude.typ\"\n"
    );
    mindtape::write_atomic(&dir.join("typst.toml"), manifest.as_bytes())
        .context("write typst.toml")?;
    mindtape::write_atomic(&dir.join("prelude.typ"), PRELUDE.as_bytes())
        .context("write prelude.typ")?;
    println!(
        "installed @local/mindtape:{TYPST_PKG_VERSION} -> {}",
        dir.display()
    );
    Ok(())
}

pub(crate) fn package_dir() -> Result<PathBuf> {
    let mut path = mindtape::data_home().context("determine local data directory")?;
    path.push("typst");
    path.push("packages");
    path.push("local");
    path.push("mindtape");
    path.push(TYPST_PKG_VERSION);
    Ok(path)
}

/// Write the scaffold, returning whether it landed. An existing config without
/// `--force` is left alone and reported, and the caller writes no templates
/// either — a half-applied preset is worse than none.
pub(crate) fn scaffold_config(root: &Path, force: bool, template: &str) -> Result<bool> {
    let dir = root.join(".mindtape");
    let config = dir.join("config.toml");
    if config.exists() && !force {
        println!(
            "{} already exists (use --force to overwrite)",
            config.display()
        );
        return Ok(false);
    }
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    mindtape::write_atomic(&config, template.as_bytes())
        .with_context(|| format!("write {}", config.display()))?;
    println!("wrote {}", config.display());
    Ok(true)
}

/// Drop the preset's `mt add` templates into `.mindtape/templates`.
///
/// A preset that constrains what a task must contain ships the template that
/// produces it; without this the scaffolded project's first `mt add` would
/// write a task its own config rejects.
pub(crate) fn write_templates(root: &Path, preset: &Preset) -> Result<()> {
    if preset.templates.is_empty() {
        return Ok(());
    }
    let dir = root.join(".mindtape").join("templates");
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    for (file, body) in &preset.templates {
        let path = dir.join(file);
        mindtape::write_atomic(&path, body.as_bytes())
            .with_context(|| format!("write {}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
