//! `mt _cli` — dump the command tree as JSON for the docs generator.
//!
//! Hidden. Reflects `Cli::command()` (the clap tree the parser already builds
//! from the `#[derive(Parser)]` structs) into a stable JSON schema consumed by
//! the docs site's rendered CLI reference (`www/content/cli.typ`). The derive
//! structs are the single source of truth; this only reads them back, so the
//! docs can never restate — or drift from — a flag fact.
//!
//! Every leaf command node also carries a `class` field — its read/write
//! classification from [`crate::cli::command_class`] — so `www/generated/cli.json`
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
use crate::cli::command_class::{self, Class};
use crate::cli::completion::{self, Source};

/// Arguments for the hidden `_cli` dump command.
#[derive(Debug, clap::Args)]
pub struct CliDumpArgs {
    /// Emit the command tree as JSON (the only supported format).
    #[arg(long)]
    pub json: bool,

    /// Judge one `mt ...` command line the way `mt` would parse it, and exit
    /// non-zero with the reason when it would refuse. A missing required
    /// argument passes.
    #[arg(long, value_name = "LINE")]
    pub check_argv: Option<String>,

    /// Status verbs `--check-argv` accepts as commands (`mt done ...`).
    #[arg(long, value_delimiter = ',', requires = "check_argv")]
    pub statuses: Vec<String>,
}

/// Run `mt _cli`: print the reflected command tree as pretty JSON.
///
/// `--json` is accepted for an explicit invocation; JSON is the only format, so
/// it is emitted regardless.
///
/// # Errors
/// Propagates a JSON serialization failure (not expected for this fixed shape).
pub fn run(args: &CliDumpArgs) -> Result<()> {
    if let Some(line) = &args.check_argv {
        return crate::cli::check_argv::run(line, &args.statuses);
    }
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
        // A global belongs to the root, so its completion entry is keyed by
        // the empty path — the same key `crate::cli::completion` walks it under.
        .map(|arg| arg_json(arg, ""))
        .collect();
    json!({
        "schema": 1,
        "bin": root.get_name(),
        "version": root.get_version(),
        "about": opt_str(root.get_about()),
        "globals": globals,
        "commands": subcommands(root, ""),
        "filter_functions": filter_functions(),
    })
}

/// The filter language's registry of predicates and functions, each with its
/// arity (`max` is null when variadic), so the docs gate reads the names off
/// the code that dispatches them.
fn filter_functions() -> Vec<Value> {
    mindtape::filter::functions()
        .iter()
        .map(|entry| {
            json!({
                "name": entry.name,
                "min_args": entry.arity.min,
                "max_args": entry.arity.max,
            })
        })
        .collect()
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
        .map(|arg| arg_json(arg, &path))
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
/// <name>"` — the same joining [`crate::cli::command_class::CLASSIFICATION`]'s keys
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
/// [`crate::cli::command_class`] entry — never expected to ship (the
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
fn arg_json(arg: &Arg, path: &str) -> Value {
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
        "completion": completion_json(path, arg.get_id().as_str()),
    })
}

/// Where the argument's values come from, as [`crate::cli::completion::SOURCES`]
/// declares it: `{"source": "stems", "reason": null}` for an argument with a
/// value source, `{"source": null, "reason": "..."}` for one deliberately
/// without, and `null` for an argument the table does not name at all.
///
/// That third state is the one `just check-doc-facts` fails on, which is why
/// it is reflected here rather than grepped out of `cli/completion/mod.rs`: the gate
/// then reads the same table the completers are built from, so it cannot be
/// satisfied by a mention the parser never sees.
fn completion_json(path: &str, id: &str) -> Value {
    match completion::lookup(path, id) {
        Some(Source::None(reason)) => json!({"source": Value::Null, "reason": reason}),
        Some(source) => json!({"source": source.label(), "reason": Value::Null}),
        None => Value::Null,
    }
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
#[path = "tree_json_tests.rs"]
mod tests;
