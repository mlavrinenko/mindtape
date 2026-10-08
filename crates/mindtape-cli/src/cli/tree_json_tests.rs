#![allow(clippy::unwrap_used)]
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
fn filter_functions_carry_the_registry() {
    let t = tree();
    let names: Vec<_> = arr(&t, "filter_functions")
        .iter()
        .map(|f| field(f, "name"))
        .collect();
    assert!(names.contains(&"exists") && names.contains(&"has"));
    assert_eq!(names.len(), mindtape::filter::functions().len());
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
/// `cli::command_class::CLASSIFICATION` must equal the visible leaf commands
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

/// `completion_json` reflects the `cli::completion::SOURCES` table: a sourced
/// argument names its source, a deliberately-sourceless one carries its
/// reason, and an unnamed argument is `null`. A stub return of
/// `Default::default()` (always `null`) loses the first two.
#[test]
fn completion_json_reflects_the_source_table() {
    let sourced = completion_json("ls", "filter");
    assert_eq!(
        sourced.get("source").and_then(Value::as_str),
        Some("view-refs"),
        "{sourced}"
    );
    assert!(sourced.get("reason").unwrap().is_null(), "{sourced}");

    let reasonless = completion_json("ls", "limit");
    assert!(reasonless.get("source").unwrap().is_null(), "{reasonless}");
    assert!(
        reasonless.get("reason").and_then(Value::as_str).is_some(),
        "{reasonless}"
    );

    assert!(completion_json("nope", "nope").is_null());
}

/// Every visible read command prints JSON with `--json`; `config schema`
/// already prints JSON and is the one exemption.
#[test]
fn every_read_command_takes_json() {
    fn walk(node: &Value, prefix: &str, missing: &mut Vec<String>) {
        let name = field(node, "name");
        let path = flattened_name(prefix, name);
        let subs = arr(node, "subcommands");
        let is_read = node
            .get("class")
            .and_then(|class| class.get("kind"))
            .and_then(Value::as_str)
            == Some("read");
        if subs.is_empty()
            && is_read
            && path != "config schema"
            && !longs(arr(node, "args")).contains(&"json")
        {
            missing.push(path.clone());
        }
        for sub in subs {
            walk(sub, &path, missing);
        }
    }
    let mut missing = Vec::new();
    for top in arr(&tree(), "commands") {
        walk(top, "", &mut missing);
    }
    assert!(
        missing.is_empty(),
        "read commands without --json: {missing:?}"
    );
}
