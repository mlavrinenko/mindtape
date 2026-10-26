//! `[params]` and `param()`: a pack's parameters, set from its `extends`
//! entry's `with` table and written into the pack's expressions as literals
//! at load. The shared `layer` fixture stays in the parent test module and is
//! reached through `super::tests`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::merge_layers_with_packs_using;
use super::tests::layer;
use crate::config::ConfigError;
use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::packs_toml::read_array;
use crate::test_support::{home_and_project, tempdir, write_file};

/// A pack capping `estimate.src-files` at its `max-files` parameter.
const BUDGET: &str = "[params]\nmax-files = 5\n\n\
    [[tasks.rules]]\nname = \"small\"\nassert = 'estimate.src-files <= param(\"max-files\")'\n";

/// Merge one embedded layer whose own text is `own`, with `budget` as the
/// pack `mindtape:budget`.
fn merge_with(budget: &str, own: &str) -> Result<super::PackMerge, ConfigError> {
    let root = tempdir();
    let layers = [layer(LayerSource::Embedded, own)];
    merge_layers_with_packs_using(&layers, root.path(), &[("budget", budget)])
}

/// The `assert` of the first `[[tasks.rules]]` entry `own` merges to.
fn first_assert(budget: &str, own: &str) -> String {
    let merged = merge_with(budget, own).unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    rules[0]["assert"].as_str().unwrap().to_owned()
}

/// The name `err` suggests in place of an unknown one.
fn nearest(err: ConfigError) -> Option<String> {
    let ConfigError::Invalid { unknown, .. } = err else {
        panic!("not Invalid: {err}")
    };
    unknown.and_then(|unknown| unknown.nearest)
}

fn error_of(budget: &str, own: &str) -> String {
    merge_with(budget, own).unwrap_err().to_string()
}

#[test]
fn a_bare_entry_writes_the_default_into_the_expression() {
    let assert = first_assert(BUDGET, "extends = [\"mindtape:budget\"]\n");
    assert_eq!(assert, "estimate.src-files <= 5");
}

#[test]
fn a_with_value_replaces_the_default() {
    let own = "extends = [{ pack = \"mindtape:budget\", with = { max-files = 8 } }]\n";
    assert_eq!(first_assert(BUDGET, own), "estimate.src-files <= 8");
}

#[test]
fn the_params_table_never_reaches_the_merged_config() {
    let merged = merge_with(BUDGET, "extends = [\"mindtape:budget\"]\n").unwrap();
    assert!(merged.table.get("params").is_none());
}

#[test]
fn a_param_call_inside_a_string_literal_is_left_alone() {
    let pack = "[params]\nmax-files = 5\n\n[[tasks.rules]]\nname = \"small\"\n\
        assert = 'title != \"param(\\\"max-files\\\")\" && estimate.src-files <= param(\"max-files\")'\n";
    let assert = first_assert(pack, "extends = [\"mindtape:budget\"]\n");
    assert_eq!(
        assert,
        r#"title != "param(\"max-files\")" && estimate.src-files <= 5"#
    );
}

#[test]
fn string_float_and_boolean_values_become_their_literals() {
    let pack = "[params]\nkind = 'a \"b\" \\ c'\nratio = 2.0\nstrict = true\n\n\
        [[tasks.rules]]\nname = \"r\"\n\
        assert = 'kind == param(\"kind\") && x <= param(\"ratio\") && param(\"strict\")'\n";
    let assert = first_assert(pack, "extends = [\"mindtape:budget\"]\n");
    assert_eq!(assert, r#"kind == "a \"b\" \\ c" && x <= 2.0 && true"#);
}

#[test]
fn every_expression_key_of_the_pack_is_rewritten() {
    let pack = "[params]\np = 7\n\n\
        [[tasks.rules]]\nname = \"r\"\nwhen = 'a == param(\"p\")'\nassert = 'b == param(\"p\")'\n\n\
        [[commits.rules]]\nname = \"c\"\nwhen = 'c == param(\"p\")'\nassert = 'd == param(\"p\")'\n\n\
        [[tasks.body.headers.rules]]\nwhen = 'e == param(\"p\")'\n\n\
        [[on.flip]]\nwhen = 'f == param(\"p\")'\n\n\
        [views.mine]\nfilter = 'g == param(\"p\")'\n\n\
        [tasks.priority]\nrequired = 'h == param(\"p\")'\n\n\
        [tasks.fields.risk]\nrequired = 'i == param(\"p\")'\n\n\
        [[ls.rules]]\ncondition = 'j == param(\"p\")'\nfilter = 'l == param(\"p\")'\n\n\
        [ls.columns.title]\nwhen = 'm == param(\"p\")'\n\n\
        [rm]\ndefault-filter = 'k == param(\"p\")'\n";
    let merged = merge_with(pack, "extends = [\"mindtape:budget\"]\n").unwrap();
    let doc = toml::Value::Table(merged.table).to_string();
    assert!(!doc.contains("param("), "{doc}");
    assert_eq!(doc.matches("== 7").count(), 13, "{doc}");
}

#[test]
fn a_param_call_in_a_message_is_not_an_expression() {
    let pack = "[params]\np = 7\n\n[[tasks.rules]]\nname = \"r\"\nassert = 'true'\n\
        message = 'see param(\"p\")'\n";
    let merged = merge_with(pack, "extends = [\"mindtape:budget\"]\n").unwrap();
    let rules = read_array(&merged.table, &["tasks", "rules"]);
    assert_eq!(rules[0]["message"].as_str(), Some("see param(\"p\")"));
}

#[test]
fn a_with_key_the_pack_does_not_declare_is_refused_with_the_nearest_name() {
    let own = "extends = [{ pack = \"mindtape:budget\", with = { max-file = 8 } }]\n";
    let err = merge_with(BUDGET, own).unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid config: extends: pack 'mindtape:budget' has no parameter: 'max-file'"
    );
    assert_eq!(nearest(err).as_deref(), Some("max-files"));
}

#[test]
fn a_with_value_of_another_type_than_the_default_is_refused() {
    let own = "extends = [{ pack = \"mindtape:budget\", with = { max-files = \"8\" } }]\n";
    assert_eq!(
        error_of(BUDGET, own),
        "invalid config: extends: pack 'mindtape:budget' parameter 'max-files' takes an integer, \
         not a string"
    );
}

#[test]
fn a_param_call_naming_no_parameter_is_refused_with_the_key() {
    let pack = BUDGET.replace("param(\"max-files\")", "param(\"max-file\")");
    let err = merge_with(&pack, "extends = [\"mindtape:budget\"]\n").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid config: extends: pack 'mindtape:budget' reads an undeclared parameter in \
         'tasks.rules[0].assert': 'max-file'"
    );
    assert_eq!(nearest(err).as_deref(), Some("max-files"));
}

#[test]
fn a_param_call_that_is_not_one_quoted_name_is_refused() {
    let pack = BUDGET.replace("param(\"max-files\")", "param(max)");
    assert_eq!(
        error_of(&pack, "extends = [\"mindtape:budget\"]\n"),
        "invalid config: extends: pack 'mindtape:budget' calls param() without one quoted name in \
         'tasks.rules[0].assert'"
    );
}

#[test]
fn a_param_value_that_is_no_scalar_is_refused() {
    assert_eq!(
        error_of(
            "[params]\nnames = [\"a\"]\n",
            "extends = [\"mindtape:budget\"]\n"
        ),
        "invalid config: extends: pack 'mindtape:budget' parameter 'names' must be a string, \
         a number or a boolean"
    );
}

#[test]
fn a_param_call_in_a_layers_own_expression_is_refused() {
    let own =
        "[[tasks.rules]]\nname = \"r\"\nassert = 'estimate.src-files <= param(\"max-files\")'\n";
    assert_eq!(
        error_of(BUDGET, own),
        "invalid config: tasks.rules[0].assert: param() reads a pack's parameter, and this \
         layer is no pack: 'max-files'"
    );
}

#[test]
fn one_pack_extended_twice_in_a_layer_is_refused() {
    let own = "extends = [\"mindtape:budget\", { pack = \"mindtape:budget\", with = { max-files = 8 } }]\n";
    assert_eq!(
        error_of(BUDGET, own),
        "invalid config: extends: pack 'mindtape:budget' is listed twice"
    );
}

#[test]
fn an_entry_that_is_neither_a_name_nor_a_pack_table_is_refused() {
    for own in [
        "extends = [5]\n",
        "extends = [{ with = { max-files = 8 } }]\n",
    ] {
        assert_eq!(
            error_of(BUDGET, own),
            "invalid config: extends: an entry is a pack name or a { pack, with } table",
            "{own}"
        );
    }
    assert_eq!(
        error_of(
            BUDGET,
            "extends = [{ pack = \"mindtape:budget\", width = {} }]\n"
        ),
        "invalid config: extends: unknown key in an entry: 'width'"
    );
}

#[test]
fn a_path_pack_with_params_loads_with_the_with_value() {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, "team.toml", BUDGET);
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [{ pack = \"./team.toml\", with = { max-files = 8 } }]\n",
    );

    let config = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0;
    let rule = config
        .tasks
        .rules
        .iter()
        .find(|rule| rule.name.as_deref() == Some("small"))
        .unwrap();
    assert_eq!(rule.assertion.as_deref(), Some("estimate.src-files <= 8"));
}
