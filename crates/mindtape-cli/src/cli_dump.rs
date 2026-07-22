//! `mt _cli` — dump the command tree as JSON for the docs generator.
//!
//! Hidden. Reflects `Cli::command()` (the clap tree the parser already builds
//! from the `#[derive(Parser)]` structs) into a stable JSON schema consumed by
//! the docs site's rendered CLI reference (`www/content/cli.typ`). The derive
//! structs are the single source of truth; this only reads them back, so the
//! docs can never restate — or drift from — a flag fact.
//!
//! Refs: `truth/tasks/structured-cli-reference-from-command-dump.typ`.

use anyhow::Result;
use clap::{Arg, ArgAction, Command, CommandFactory};
use serde_json::{Value, json};

use crate::cli::Cli;

/// Arguments for the hidden `_cli` dump command.
#[derive(Debug, clap::Args)]
pub struct CliDumpArgs {
    /// Emit the command tree as JSON (the only supported format).
    #[arg(long)]
    pub json: bool,
}

/// Run `mt _cli`: print the reflected command tree as pretty JSON.
///
/// `--json` is accepted for an explicit invocation; JSON is the only format, so
/// it is emitted regardless.
///
/// # Errors
/// Propagates a JSON serialization failure (not expected for this fixed shape).
pub fn run(_args: &CliDumpArgs) -> Result<()> {
    println!("{}", dump_json()?);
    Ok(())
}

/// Build the pretty-printed JSON command tree. Split out from [`run`] so it is
/// testable without printing to stdout.
fn dump_json() -> Result<String> {
    serde_json::to_string_pretty(&dump(&Cli::command())).map_err(Into::into)
}

/// The top-level document: about, version, the once-captured globals, and the
/// recursive command tree.
fn dump(root: &Command) -> Value {
    let globals: Vec<Value> = root
        .get_arguments()
        .filter(|a| a.is_global_set())
        .map(arg_json)
        .collect();
    json!({
        "schema": 1,
        "bin": root.get_name(),
        "version": root.get_version(),
        "about": opt_str(root.get_about()),
        "globals": globals,
        "commands": subcommands(root),
    })
}

/// Visible, non-`help` child commands, each reflected recursively.
fn subcommands(cmd: &Command) -> Vec<Value> {
    cmd.get_subcommands()
        // clap's synthetic `help` pseudo-command and any `hide = true` command
        // (InstallLib, `_cli` itself) never appear in the reference.
        .filter(|c| c.get_name() != "help" && !c.is_hide_set())
        .map(command_json)
        .collect()
}

/// One command node: name, both about levels, visible aliases, local args, and
/// nested subcommands. Global args are dropped here — they live once at the top.
fn command_json(cmd: &Command) -> Value {
    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|a| !a.is_global_set())
        // clap's auto `-h/--help` and `-V/--version` are formatter artifacts,
        // not real args; the reference synthesizes its own affordances.
        .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
        .map(arg_json)
        .collect();
    json!({
        "name": cmd.get_name(),
        "about": opt_str(cmd.get_about()),
        "long_about": opt_str(cmd.get_long_about()),
        "aliases": cmd.get_visible_aliases().collect::<Vec<_>>(),
        "args": args,
        "subcommands": subcommands(cmd),
    })
}

/// One argument (positional, flag, or option), discriminated by `kind`.
///
/// Boolean/count flags carry a clap-synthesized value placeholder and
/// `true`/`false` possible-values; both are formatter noise, so they are
/// nulled for `flag` kinds — a flag takes no value.
fn arg_json(arg: &Arg) -> Value {
    let is_flag = kind_of(arg) == "flag";
    json!({
        "id": arg.get_id().as_str(),
        "kind": kind_of(arg),
        "short": arg.get_short().map(|c| c.to_string()),
        "long": arg.get_long(),
        "value_name": if is_flag { None } else { value_name(arg) },
        "about": opt_str(arg.get_help()),
        "long_about": opt_str(arg.get_long_help()),
        "required": arg.is_required_set(),
        "multiple": matches!(arg.get_action(), ArgAction::Append | ArgAction::Count),
        "default": default_value(arg),
        "possible_values": if is_flag { None } else { possible_values(arg) },
    })
}

/// Classify an arg: positional, boolean/count `flag`, or value-taking `option`.
fn kind_of(arg: &Arg) -> &'static str {
    if arg.is_positional() {
        "positional"
    } else if matches!(
        arg.get_action(),
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count
    ) {
        "flag"
    } else {
        "option"
    }
}

/// The first value placeholder (`FILTER`, `LIMIT`, …), if the arg takes one.
fn value_name(arg: &Arg) -> Option<String> {
    arg.get_value_names()
        .and_then(<[clap::builder::Str]>::first)
        .map(ToString::to_string)
}

/// The default value rendered as a string, if one is declared.
fn default_value(arg: &Arg) -> Option<String> {
    arg.get_default_values()
        .first()
        .map(|v| v.to_string_lossy().into_owned())
}

/// Enum-valued args' accepted values, each with its own help.
fn possible_values(arg: &Arg) -> Option<Vec<Value>> {
    let values = arg.get_possible_values();
    if values.is_empty() {
        return None;
    }
    Some(
        values
            .iter()
            .map(|pv| json!({ "name": pv.get_name(), "about": opt_str(pv.get_help()) }))
            .collect(),
    )
}

/// Flatten an optional `StyledStr` (or any displayable) to plain text.
fn opt_str<T: ToString>(styled: Option<&T>) -> Option<String> {
    styled.map(ToString::to_string)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn tree() -> Value {
        dump(&Cli::command())
    }

    // `Value` indexing (`v["k"]`) trips clippy's `indexing_slicing`; go through
    // `.get(...)` accessors instead.
    fn arr<'a>(value: &'a Value, key: &str) -> &'a [Value] {
        value.get(key).unwrap().as_array().unwrap()
    }

    fn field<'a>(value: &'a Value, key: &str) -> &'a str {
        value.get(key).and_then(Value::as_str).unwrap_or_default()
    }

    fn command<'a>(tree: &'a Value, name: &str) -> &'a Value {
        arr(tree, "commands")
            .iter()
            .find(|c| field(c, "name") == name)
            .unwrap()
    }

    fn longs(args: &[Value]) -> Vec<&str> {
        args.iter().map(|a| field(a, "long")).collect()
    }

    #[test]
    fn globals_captured_once_at_top() {
        let t = tree();
        let g = longs(arr(&t, "globals"));
        assert!(g.contains(&"worktree"));
        assert!(g.contains(&"color"));
        assert!(g.contains(&"verbose"));
    }

    #[test]
    fn subcommands_exclude_globals_hidden_and_help() {
        let t = tree();
        let names: Vec<_> = arr(&t, "commands")
            .iter()
            .map(|c| field(c, "name"))
            .collect();
        // hidden (`install-lib`, `_cli`) and clap's `help` pseudo-command absent
        assert!(!names.contains(&"install-lib"));
        assert!(!names.contains(&"_cli"));
        assert!(!names.contains(&"help"));
        // a real command is present
        assert!(names.contains(&"ls"));
        // globals are not repeated onto a subcommand
        assert!(!longs(arr(command(&t, "ls"), "args")).contains(&"worktree"));
    }

    #[test]
    fn ls_filter_positional_and_format_enum() {
        let t = tree();
        let ls = command(&t, "ls");
        let args = arr(ls, "args");
        let filter = args.iter().find(|a| field(a, "id") == "filter").unwrap();
        assert_eq!(field(filter, "kind"), "positional");
        let format = args.iter().find(|a| field(a, "id") == "format").unwrap();
        assert_eq!(field(format, "kind"), "option");
        assert_eq!(field(format, "default"), "table");
        let values: Vec<_> = arr(format, "possible_values")
            .iter()
            .map(|p| field(p, "name"))
            .collect();
        assert_eq!(values, vec!["table", "json", "csv"]);
    }

    #[test]
    fn flags_drop_synthetic_value_and_possible_values() {
        let t = tree();
        let args = arr(command(&t, "ls"), "args");
        let json_flag = args.iter().find(|a| field(a, "id") == "json").unwrap();
        assert_eq!(field(json_flag, "kind"), "flag");
        // clap invents a value name + true/false values for a bool flag; nulled.
        assert!(json_flag.get("value_name").unwrap().is_null());
        assert!(json_flag.get("possible_values").unwrap().is_null());
    }

    #[test]
    fn verbose_flag_is_repeatable() {
        let t = tree();
        let verbose = arr(&t, "globals")
            .iter()
            .find(|g| field(g, "long") == "verbose")
            .unwrap();
        assert_eq!(field(verbose, "kind"), "flag");
        assert_eq!(verbose.get("multiple").and_then(Value::as_bool), Some(true));
    }

    #[test]
    fn config_show_nested() {
        let t = tree();
        let subs = arr(command(&t, "config"), "subcommands");
        assert!(subs.iter().any(|c| field(c, "name") == "show"));
    }

    #[test]
    fn value_name_returns_some_for_positional() {
        let cmd = Cli::command();
        let ls = cmd.find_subcommand("ls").unwrap();
        let filter = ls.get_arguments().find(|a| a.get_id() == "filter").unwrap();
        assert_eq!(value_name(filter), Some("FILTER".to_owned()));
    }

    #[test]
    fn opt_str_maps_to_string() {
        let styled = clap::builder::StyledStr::from("hello");
        assert_eq!(opt_str(Some(&styled)), Some("hello".to_owned()));
    }

    #[test]
    fn opt_str_none_when_input_none() {
        let none: Option<&clap::builder::StyledStr> = None;
        assert_eq!(opt_str(none), None);
    }

    #[test]
    fn dump_json_is_valid() {
        let json = dump_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("schema").is_some());
    }

    #[test]
    fn dump_json_contains_bin_name() {
        let json = dump_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.get("bin").and_then(Value::as_str), Some("mt"));
    }
}
