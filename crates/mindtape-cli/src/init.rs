//! `mt init` / `mt install-lib` — project scaffold and library install.
//!
//! `init` writes one of the named presets in [`crate::preset`]. Two axes, kept
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

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::init_wizard::{
    self, Estimate, Headers, Organisation, Priority, Recurrence, Required, Workflow,
};
use crate::preset::{self, Preset};

/// The Typst package version installed by `install-lib`. Mirrors `lib/typst.toml`.
const TYPST_PKG_VERSION: &str = "0.2.0";
const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

/// The `#:schema` directive controls, grouped so they travel together — the
/// two are one decision, and splitting them across the parent's flag list
/// makes it read like four unrelated toggles.
#[derive(Debug, clap::Args)]
pub struct SchemaArgs {
    /// Value for the top `#:schema` directive: a URL, or a path relative to the
    /// config file. Defaults to a `file://` URL for the schema `mt install`
    /// writes under the XDG config dir.
    #[arg(long, value_name = "URL|PATH")]
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
    /// `~/.config/mindtape/presets`, or a path to a config or preset directory.
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
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let interactive = !args.no_input && tty;
    if interactive || flags.any() {
        // A terminal walks the axes; a non-terminal run with any flag composes
        // from the flags plus defaults, without prompting.
        return run_wizard(args, &flags, interactive);
    }
    // No preset, no flags, and no terminal (or `--no-input`): write the default
    // preset without asking anything — the non-interactive baseline.
    write_preset(preset::DEFAULT_PRESET, args, false)
}

/// Collect the per-axis flag values off `InitArgs`.
fn wizard_flags(args: &InitArgs) -> init_wizard::Flags {
    init_wizard::Flags {
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
    let text = with_schema_directive(&preset.render(args.annotate), &args.schema)?;
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

/// Walk the axes (or compose from flags when non-interactive), then scaffold the
/// composed config and any templates the answers require.
fn run_wizard(args: &InitArgs, flags: &init_wizard::Flags, interactive: bool) -> Result<()> {
    let answers = {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut writer = std::io::stdout();
        init_wizard::resolve(flags, interactive, &mut reader, &mut writer)?
    };
    let outcome = init_wizard::assemble(answers)?;
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
    println!("\nuser presets live in ~/.config/mindtape/presets and shadow these.");
}

/// Prepend the `#:schema` comment directive to the scaffold, unless
/// `--no-schema`. The value is `--schema` when given, else a `file://` URL to
/// the schema `mt install` writes under the XDG config dir; that file may not
/// exist yet, so a hint nudges the user to run `mt install`.
fn with_schema_directive(template: &str, args: &SchemaArgs) -> Result<String> {
    if args.no_schema {
        return Ok(template.to_owned());
    }
    let value = if let Some(value) = &args.schema {
        value.clone()
    } else {
        let path = crate::install::xdg_schema_path()?;
        if !path.exists() {
            println!(
                "note: schema not installed at {} yet — run `mt install` (the \
                 `#:schema` directive points there)",
                path.display()
            );
        }
        format!("file://{}", path.display())
    };
    Ok(format!("#:schema {value}\n{template}"))
}

/// Run the `install-lib` command — install `@local/mindtape` into the Typst packages dir.
///
/// # Errors
/// Returns an error if the library cannot be written.
pub fn run_install_lib() -> Result<()> {
    install_lib()
}

pub(crate) fn install_lib() -> Result<()> {
    let dir = package_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let manifest = format!(
        "[package]\nname = \"mindtape\"\nversion = \"{TYPST_PKG_VERSION}\"\nentrypoint = \"prelude.typ\"\n"
    );
    std::fs::write(dir.join("typst.toml"), manifest).context("write typst.toml")?;
    std::fs::write(dir.join("prelude.typ"), PRELUDE).context("write prelude.typ")?;
    println!(
        "installed @local/mindtape:{TYPST_PKG_VERSION} -> {}",
        dir.display()
    );
    Ok(())
}

pub(crate) fn package_dir() -> Result<PathBuf> {
    let mut path = dirs::data_local_dir().context("determine local data directory")?;
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
    std::fs::write(&config, template).with_context(|| format!("write {}", config.display()))?;
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
        std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
#[path = "init_tests.rs"]
mod tests;
