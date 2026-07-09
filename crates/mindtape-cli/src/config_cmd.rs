//! `mt config` — inspect the effective merged configuration.
//!
//! Read-only: never writes. `show` (the default action) prints the effective
//! config assembled from every layer (embedded floor, XDG global defaults,
//! ancestor `.mindtape/defaults.toml` files, the project's own
//! `.mindtape/config.toml`); `--origin` lists the contributing layers instead;
//! `--defaults` prints just the embedded floor, independent of any project.

use std::path::Path;

use anyhow::{Context, Result};
use mindtape::{LayerSource, config_path, embedded_defaults, load_config_layered};

use crate::util;

/// Arguments for `config`.
#[derive(Debug, clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    action: Option<ConfigAction>,
}

/// `mt config` sub-actions.
#[derive(Debug, clap::Subcommand)]
enum ConfigAction {
    /// Print the effective merged config (the default action when omitted).
    Show(ShowArgs),
}

/// Arguments for `mt config show`.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct ShowArgs {
    /// List the contributing layers in precedence order instead of the config.
    #[arg(long, alias = "sources", conflicts_with = "defaults")]
    pub origin: bool,

    /// Print just the embedded default floor, verbatim; needs no project.
    #[arg(long, conflicts_with = "origin")]
    pub defaults: bool,

    /// Emit JSON instead of TOML for the effective config.
    #[arg(long, conflicts_with_all = ["origin", "defaults"])]
    pub json: bool,
}

/// Run the `config` command.
///
/// # Errors
/// Returns an error if the current directory is not inside a `MindTape`
/// project (not required for `--defaults`, which is project-independent), or
/// the layered config fails to load.
pub fn run(args: &ConfigArgs) -> Result<()> {
    let show_args = match &args.action {
        None => ShowArgs::default(),
        Some(ConfigAction::Show(show_args)) => show_args.clone(),
    };
    if show_args.defaults {
        println!("{}", embedded_defaults().trim_end());
        return Ok(());
    }
    let root = util::resolve_root()?;
    let text = if show_args.origin {
        render_origin(&root)?
    } else {
        render_effective(&root, show_args.json)?
    };
    print!("{text}");
    Ok(())
}

/// Render the effective merged config for `root` as TOML (or, with `json`,
/// pretty-printed JSON) text. Emits the merged `toml::Table` directly (the
/// same document that feeds `Config`'s typed deserialization) rather than
/// serializing `Config` itself, since the config structs derive only
/// `Deserialize`.
///
/// # Errors
/// Returns an error if any config layer fails to load or the table fails to
/// serialize.
pub(crate) fn render_effective(root: &Path, json: bool) -> Result<String> {
    let table = mindtape::merged_config_table(root)
        .with_context(|| format!("load {}", config_path(root).display()))?;
    if json {
        Ok(format!("{}\n", serde_json::to_string_pretty(&table)?))
    } else {
        Ok(toml::to_string(&table)?)
    }
}

/// Render the contributing layers for `root`, one per line, in the same
/// precedence order [`load_config_layered`] returns them.
///
/// # Errors
/// Returns an error if the layered config fails to load.
pub(crate) fn render_origin(root: &Path) -> Result<String> {
    let (_config, sources) = load_config_layered(root)
        .with_context(|| format!("load {}", config_path(root).display()))?;
    let mut out = String::new();
    for source in &sources {
        out.push_str(&describe_source(source));
        out.push('\n');
    }
    Ok(out)
}

/// One human-readable line describing a layer's origin and, for a real path,
/// whether it exists on disk.
fn describe_source(source: &LayerSource) -> String {
    match source {
        LayerSource::Embedded => "embedded: <built-in>".to_owned(),
        LayerSource::Xdg(path) => format!("xdg: {} ({})", path.display(), existence(path)),
        LayerSource::Dir(path) => format!("dir: {} ({})", path.display(), existence(path)),
        LayerSource::Project(path) => format!("project: {} ({})", path.display(), existence(path)),
    }
}

fn existence(path: &Path) -> &'static str {
    if path.is_file() { "exists" } else { "missing" }
}

#[cfg(test)]
#[path = "config_cmd_tests.rs"]
mod tests;
