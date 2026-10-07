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

use anyhow::{Context, Result, bail};

use crate::cmd::init::preset;
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
    /// Save the current project's `.mindtape/config.toml` and templates as a
    /// named user preset.
    Save(SaveArgs),
    /// List built-in and user presets together, marking shadows and the
    /// default.
    Ls,
}

/// Arguments for `mt preset save`.
#[derive(Debug, clap::Args)]
pub struct SaveArgs {
    /// Name to save the preset under.
    pub name: String,

    /// Overwrite an existing user preset with this name.
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
            let user_dir = preset::user_preset_dir()
                .context("could not determine the user config directory")?;
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
        .with_context(|| format!("read {}", config_path.display()))?;
    let templates = preset::read_templates(&root.join(".mindtape").join("templates"))?;

    if let Some(existing) = preset::existing_user_preset(user_dir, name) {
        if !force {
            bail!(
                "user preset {name:?} already exists at {} (use --force to overwrite)",
                existing.display()
            );
        }
        if existing.is_file() {
            std::fs::remove_file(&existing)
                .with_context(|| format!("remove {}", existing.display()))?;
        }
    }

    let dest = user_dir.join(name);
    std::fs::create_dir_all(&dest).with_context(|| format!("create {}", dest.display()))?;
    let dest_config = dest.join("config.toml");
    mindtape::write_atomic(&dest_config, config.as_bytes())
        .with_context(|| format!("write {}", dest_config.display()))?;
    if !templates.is_empty() {
        let templates_dir = dest.join("templates");
        std::fs::create_dir_all(&templates_dir)
            .with_context(|| format!("create {}", templates_dir.display()))?;
        for (file, body) in &templates {
            let path = templates_dir.join(file);
            mindtape::write_atomic(&path, body.as_bytes())
                .with_context(|| format!("write {}", path.display()))?;
        }
    }
    Ok(dest)
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
