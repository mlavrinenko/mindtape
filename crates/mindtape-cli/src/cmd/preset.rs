//! `mt preset save` / `mt preset ls` — capture and inspect user presets.
//!
//! A preset is a `.mindtape/config.toml` body plus its `templates/` (see
//! [`crate::cmd::init::preset`]); `mt init --preset` reads one, this module writes and
//! lists them. `save` captures the *current project's* config verbatim — no
//! reverse-engineering of the wizard answers that would have produced it,
//! since a wrong guess would silently change what the preset means — and
//! `ls` surfaces the one thing that is otherwise invisible until it
//! surprises someone: a user preset shadowing a built-in of the same name.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cmd::init::preset;
use crate::diag::{Command, Diag, Hint, Note};
use crate::util;

/// Arguments for `preset`.
#[derive(Debug, clap::Args)]
pub struct PresetArgs {
    #[command(subcommand)]
    action: PresetAction,
}

/// `mt preset` sub-actions.
#[derive(Debug, clap::Subcommand)]
enum PresetAction {
    /// Save this project's config and templates as a preset you can pass to
    /// `mt init -p`.
    Save(SaveArgs),
    /// List the built-in presets and yours, marking the default and any of yours
    /// that hides a built-in one.
    Ls,
}

/// Arguments for `mt preset save`.
#[derive(Debug, clap::Args)]
pub struct SaveArgs {
    /// The name to save the preset under.
    pub name: String,

    /// Overwrite your preset if one has this name.
    #[arg(long)]
    pub force: bool,
}

/// Run the `preset` command.
///
/// # Errors
/// Returns an error if the current directory is not inside a `MindTape`
/// project, the user config directory cannot be determined, or the write (for
/// `save`) or read (for `ls`) fails.
pub fn run(args: &PresetArgs) -> Result<()> {
    match &args.action {
        PresetAction::Save(save_args) => {
            let root = util::resolve_root()?;
            let user_dir = preset::user_preset_dir().ok_or_else(|| {
                Diag::failed("can't find the user config directory")
                    .hint(Hint::new("set the directory in MINDTAPE_CONFIG_HOME"))
            })?;
            let dest = save(&root, &user_dir, &save_args.name, save_args.force)?;
            println!("saved preset {:?} -> {}", save_args.name, dest.display());
            Ok(())
        }
        PresetAction::Ls => {
            print!("{}", render_ls(preset::user_preset_dir().as_deref())?);
            Ok(())
        }
    }
}

/// Capture `root`'s `.mindtape/config.toml` and `.mindtape/templates/*.typ`
/// into `<user_dir>/<name>/{config.toml,templates/}`, one directory per
/// preset so a broken one breaks only itself. Returns the destination
/// directory.
///
/// Refuses to overwrite an existing user preset of the same name unless
/// `force`; a `force`d overwrite also clears a same-named flat `<name>.toml`
/// preset, so the freshly written nested layout is not silently shadowed by
/// it on the next resolve.
///
/// # Errors
/// Returns an error if the project's config cannot be read, an existing
/// preset blocks the write without `force`, or the destination cannot be
/// written.
pub(crate) fn save(root: &Path, user_dir: &Path, name: &str, force: bool) -> Result<PathBuf> {
    let config_path = root.join(".mindtape").join("config.toml");
    let config = std::fs::read_to_string(&config_path)
        .map_err(|err| Diag::cannot_read(&config_path, &err))?;
    let templates = preset::read_templates(&root.join(".mindtape").join("templates"))?;

    if let Some(existing) = preset::existing_user_preset(user_dir, name) {
        if !force {
            return Err(Diag::refused("user preset already exists")
                .value(name)
                .note(Note::new("it is at").value(existing.display().to_string()))
                .hint(Hint::command(
                    "overwrite it",
                    Command::mt()
                        .word("preset")
                        .word("save")
                        .arg(name)
                        .word("--force"),
                ))
                .into());
        }
        if existing.is_file() {
            std::fs::remove_file(&existing).map_err(|err| {
                Diag::failed("can't remove")
                    .value(existing.display().to_string())
                    .io(&err)
            })?;
        }
    }

    let dest = user_dir.join(name);
    create_dir(&dest)?;
    let dest_config = dest.join("config.toml");
    mindtape::write_atomic(&dest_config, config.as_bytes())
        .map_err(|err| Diag::cannot_write(&dest_config, &err))?;
    if !templates.is_empty() {
        let templates_dir = dest.join("templates");
        create_dir(&templates_dir)?;
        for (file, body) in &templates {
            let path = templates_dir.join(file);
            mindtape::write_atomic(&path, body.as_bytes())
                .map_err(|err| Diag::cannot_write(&path, &err))?;
        }
    }
    Ok(dest)
}

/// Create `dir` and its parents, or say which directory could not be made.
fn create_dir(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|err| {
        Diag::failed("can't create")
            .value(dir.display().to_string())
            .io(&err)
            .into()
    })
}

/// Render `mt preset ls`: every built-in, then every user preset (if the
/// user preset directory resolves and holds any), each annotated with
/// whichever of "default" / "shadowed by user preset" / "shadows built-in"
/// applies — shadowing is otherwise invisible until it surprises someone.
///
/// # Errors
/// Returns an error if the user preset directory exists but cannot be read.
pub(crate) fn render_ls(user_dir: Option<&Path>) -> Result<String> {
    let builtins = preset::list_builtins();
    let user_names = match user_dir {
        Some(dir) => preset::list_user_presets(dir)?,
        None => Vec::new(),
    };

    let mut out = String::new();
    out.push_str("built-in presets:\n");
    for (name, description) in &builtins {
        let mut markers = Vec::new();
        if *name == preset::DEFAULT_PRESET {
            markers.push("default");
        }
        if user_names.iter().any(|user| user == name) {
            markers.push("shadowed by user preset");
        }
        out.push_str(&format!(
            "  {name:<10} {description}{}\n",
            marker_suffix(&markers)
        ));
    }

    if user_names.is_empty() {
        return Ok(out);
    }
    out.push('\n');
    out.push_str("user presets");
    if let Some(dir) = user_dir {
        out.push_str(&format!(" ({})", dir.display()));
    }
    out.push_str(":\n");
    for name in &user_names {
        let mut markers = Vec::new();
        if builtins.iter().any(|(builtin, _)| builtin == name) {
            markers.push("shadows built-in");
        }
        if name == preset::DEFAULT_PRESET {
            markers.push("default");
        }
        out.push_str(&format!("  {name}{}\n", marker_suffix(&markers)));
    }
    Ok(out)
}

/// `" (a, b)"` for non-empty markers, else `""`.
fn marker_suffix(markers: &[&str]) -> String {
    if markers.is_empty() {
        String::new()
    } else {
        format!(" ({})", markers.join(", "))
    }
}

#[cfg(test)]
#[path = "preset_tests.rs"]
mod tests;
