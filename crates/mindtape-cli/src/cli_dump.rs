//! `mt _cli` — dump the command tree as JSON for the docs generator.
//!
//! Hidden. Reflects `Cli::command()` (the clap tree the parser already builds
//! from the `#[derive(Parser)]` structs) into a stable JSON schema consumed by
//! the docs site's rendered CLI reference (`www/content/cli.typ`). The derive
//! structs are the single source of truth; this only reads them back, so the
//! docs can never restate — or drift from — a flag fact.
//!
//! Every leaf command node also carries a `class` field — its read/write
//! classification from [`crate::command_class`] — so `www/generated/cli.json`
//! carries the CQRS split for `docs/src/contributing.typ` and
//! `tests/read_no_write_e2e.rs` to read, instead of either retyping it. A
//! node with subcommands (`config`, `preset`) carries `class: null`: it is
//! not a leaf, only its children are classified.
//!
//! Refs: `truth/tasks/structured-cli-reference-from-command-dump.typ`,
//! `truth/tasks/gate-the-cqrs-split-against-the-command-tree.typ`.

use anyhow::Result;
use clap::{Arg, ArgAction, Command, CommandFactory};
use serde_json::{Value, json};

use crate::cli::Cli;
use crate::command_class::{self, Class};

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
        "commands": subcommands(root, ""),
    })
}

/// Visible, non-`help` child commands, each reflected recursively. `prefix` is
/// the flattened path of `cmd` itself (empty at the root), threaded down so
/// each child can compute its own [`flattened_name`].
fn subcommands(cmd: &Command, prefix: &str) -> Vec<Value> {
    cmd.get_subcommands()
        // clap's synthetic `help` pseudo-command and any `hide = true` command
        // (`_cli` itself) never appear in the reference.
        .filter(|c| c.get_name() != "help" && !c.is_hide_set())
        .map(|c| command_json(c, prefix))
        .collect()
}

/// One command node: name, both about levels, visible aliases, local args,
/// read/write `class`, and nested subcommands. Global args are dropped here —
/// they live once at the top.
fn command_json(cmd: &Command, prefix: &str) -> Value {
    let name = cmd.get_name();
    let path = flattened_name(prefix, name);
    let args: Vec<Value> = cmd
        .get_arguments()
        .filter(|a| !a.is_global_set())
        // clap's auto `-h/--help` and `-V/--version` are formatter artifacts,
        // not real args; the reference synthesizes its own affordances.
        .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
        .map(arg_json)
        .collect();
    let subs = subcommands(cmd, &path);
    // Only a leaf is classified — `config`/`preset` are containers, not
    // commands in their own right; their children carry the class instead.
    let class = if subs.is_empty() {
        class_json(&path)
    } else {
        Value::Null
    };
    json!({
        "name": name,
        "about": opt_str(cmd.get_about()),
        "long_about": opt_str(cmd.get_long_about()),
        "aliases": cmd.get_visible_aliases().collect::<Vec<_>>(),
        "args": args,
        "class": class,
        "subcommands": subs,
    })
}

/// A node's flattened path: its own name if `prefix` is empty, else `"<prefix>
/// <name>"` — the same joining [`crate::command_class::CLASSIFICATION`]'s keys
/// use for a nested leaf like `"config show"`.
fn flattened_name(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix} {name}")
    }
}

/// A leaf's classification as JSON: `"read"`/`"write"`, or `{"kind":
/// "exempt", "reason": "..."}`. `null` for a name with no
/// [`crate::command_class`] entry — never expected to ship (the
/// exhaustiveness tests below catch it first), but this function stays total
/// rather than panicking the dump itself.
fn class_json(path: &str) -> Value {
    match command_class::classify(path) {
        Some(Class::Read) => json!({"kind": "read", "reason": null}),
        Some(Class::Write) => json!({"kind": "write", "reason": null}),
        Some(Class::Exempt(reason)) => json!({"kind": "exempt", "reason": reason}),
        None => Value::Null,
    }
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
        // hidden (`_cli`) and clap's `help` pseudo-command absent
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
        assert_eq!(values, vec!["table", "json", "csv", "path", "path0"]);
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
    fn config_show_carries_its_own_class_not_the_parent() {
        let t = tree();
        // `config` itself has children, so it is not classified directly...
        assert!(command(&t, "config").get("class").unwrap().is_null());
        // ...only its leaves are.
        let show = arr(command(&t, "config"), "subcommands")
            .iter()
            .find(|c| field(c, "name") == "show")
            .unwrap();
        assert_eq!(field(show.get("class").unwrap(), "kind"), "read");
    }

    #[test]
    fn go_carries_an_exempt_class_with_a_reason() {
        let t = tree();
        let go = command(&t, "go");
        let class = go.get("class").unwrap();
        assert_eq!(field(class, "kind"), "exempt");
        assert!(!field(class, "reason").is_empty());
    }

    /// Every leaf's `(flattened path, class)` under `commands`, recursing
    /// into any node that itself has subcommands (`config`, `preset`) — the
    /// same tree `www/generated/cli.json` ships, walked back for the
    /// exhaustiveness tests below rather than re-derived from `Cli::command()`
    /// a second, possibly-diverging way.
    fn leaf_paths(commands: &[Value], prefix: &str, out: &mut Vec<(String, Value)>) {
        for cmd in commands {
            let name = field(cmd, "name");
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix} {name}")
            };
            let subs = arr(cmd, "subcommands");
            if subs.is_empty() {
                out.push((path, cmd.get("class").cloned().unwrap_or(Value::Null)));
            } else {
                leaf_paths(subs, &path, out);
            }
        }
    }

    /// The exhaustiveness gate this module exists for: the union of
    /// `command_class::CLASSIFICATION` must equal the visible leaf commands
    /// `mt _cli --json` reflects, in both directions — mirrors
    /// `check-doc-coverage`'s set-difference shape, failing naming the
    /// offender rather than healing silently.
    #[test]
    fn every_visible_leaf_is_classified_and_no_entry_is_stale() {
        let t = tree();
        let mut leaves = Vec::new();
        leaf_paths(arr(&t, "commands"), "", &mut leaves);

        let unclassified: Vec<&str> = leaves
            .iter()
            .filter(|(_, class)| class.is_null())
            .map(|(name, _)| name.as_str())
            .collect();
        assert!(
            unclassified.is_empty(),
            "command(s) visible in `mt _cli --json` with no entry in \
             command_class::CLASSIFICATION: {unclassified:?}"
        );

        let visible: Vec<&str> = leaves.iter().map(|(name, _)| name.as_str()).collect();
        let stale: Vec<&str> = command_class::CLASSIFICATION
            .iter()
            .map(|(name, _)| *name)
            .filter(|name| !visible.contains(name))
            .collect();
        assert!(
            stale.is_empty(),
            "command_class::CLASSIFICATION entry no longer visible in \
             `mt _cli --json`: {stale:?}"
        );
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
