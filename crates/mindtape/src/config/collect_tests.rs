#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::layers::{LayerSource, load_config_layered_from};
use crate::config::pack_registry::RuleOrigin;
use crate::config::{
    CollectorAtom, CollectorQuery, CollectorSpec, CollectorType, RegexQuery, SectionSpec,
    parse_config,
};
use crate::test_support::{home_and_project, write_file};

fn config(text: &str) -> crate::config::Config {
    parse_config(text).unwrap()
}

fn refusal(text: &str) -> crate::problem::Problem {
    config(text).validate().unwrap_err()
}
const CHANGES: &str = r#"
[[tasks.collectors]]
name = "changes"
when = "!terminal"
query = { section = "Changes", pattern = "(?m)^\\s*-\\s+`([^`]+)`" }
returns = ["path"]
"#;

#[test]
fn a_regex_collector_parses() {
    let config = config(CHANGES);
    let spec = &config.tasks.collectors[0];
    assert_eq!(spec.name, "changes");
    assert_eq!(spec.when.as_deref(), Some("!terminal"));
    assert_eq!(
        spec.returns,
        CollectorType::List(Box::new(CollectorType::Atom(CollectorAtom::Path)))
    );
    assert!(matches!(&spec.query, CollectorQuery::Regex(query) if query.pattern.contains("`")));
    assert!(config.validate().is_ok());
}

#[test]
fn a_record_returns_parses() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "summary"
query = { section = "Changes", pattern = "x" }
returns = { count = "number", files = ["path"] }
"#,
    );
    let returns = &config.tasks.collectors[0].returns;
    assert_eq!(
        returns.field("count"),
        Some(&CollectorType::Atom(CollectorAtom::Number))
    );
    assert_eq!(
        returns.field("files"),
        Some(&CollectorType::List(Box::new(CollectorType::Atom(
            CollectorAtom::Path
        ))))
    );
}

#[test]
fn an_unknown_type_atom_is_refused() {
    let err = parse_config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = "Changes", pattern = "x" }
returns = "paths"
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("unknown type"), "{err}");
}

#[test]
fn a_regex_must_return_a_string_or_path_list() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = "Changes", pattern = "x" }
returns = "string"
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("returns"), "{problem}");
}

#[test]
fn a_bad_group_is_refused() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = "Changes", pattern = "(a)" , group = 2 }
returns = ["string"]
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("group"), "{problem}");
}

#[test]
fn two_collectors_in_one_file_cannot_share_a_name() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = "Changes", pattern = "x" }
returns = ["string"]

[[tasks.collectors]]
name = "x"
query = { section = "Changes", pattern = "y" }
returns = ["string"]
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("duplicate name"), "{problem}");
}

#[test]
fn a_filter_naming_an_undeclared_derived_field_fails_load() {
    let config = config(&format!(
        "{CHANGES}\n[[tasks.rules]]\nname = \"r\"\nassert = 'derived.missing == \"x\"'\n"
    ));
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("derived.missing"), "{problem}");
}

#[test]
fn a_filter_naming_a_declared_derived_field_passes_load() {
    let config = config(&format!(
        "{CHANGES}\n[[tasks.rules]]\nname = \"r\"\nassert = 'len(derived.changes) > 0'\n"
    ));
    assert!(config.validate().is_ok(), "{:?}", config.validate());
}

#[test]
fn a_commit_rule_cannot_read_a_derived_field() {
    let config = config(&format!(
        "{CHANGES}\n[[commits.rules]]\nname = \"r\"\nassert = 'derived.changes == \"x\"'\n"
    ));
    let problem = config.validate().unwrap_err();
    assert!(
        problem.text.contains("not available to commit rules"),
        "{problem}"
    );
}

#[test]
fn a_list_derived_field_cannot_sort() {
    let config = config(&format!(
        "{CHANGES}\n[ls]\ndefault-sort = [\"derived.changes\"]\n"
    ));
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("only a scalar sorts"), "{problem}");
}

#[test]
fn a_scalar_derived_field_sorts() {
    let spec = CollectorSpec {
        name: "summary".to_owned(),
        when: None,
        query: CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: CollectorType::Atom(CollectorAtom::Number),
        root: None,
    };
    assert!(crate::sort::validate_sort_spec_with("derived.summary", &[spec]).is_ok());
}

const PACK_COLLECTOR: &str = r#"
[[tasks.collectors]]
name = "changes"
query = { section = "Changes", pattern = "a" }
returns = ["path"]
"#;

#[test]
fn a_pack_collector_merges_by_name_and_records_the_winning_origin() {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, "pack.toml", PACK_COLLECTOR);
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./pack.toml\"]\n\n[[tasks.collectors]]\nname = \"changes\"\n\
         query = { section = \"Changes\", pattern = \"b\" }\nreturns = [\"path\"]\n",
    );
    let config = load_config_layered_from(&project, Some(&home), None)
        .unwrap()
        .0;
    assert_eq!(config.tasks.collectors.len(), 1);
    assert!(
        matches!(&config.tasks.collectors[0].query, CollectorQuery::Regex(query) if query.pattern == "b")
    );
    assert!(matches!(
        config.collector_origins()[0].origin,
        RuleOrigin::Layer(LayerSource::Project(_))
    ));
}

#[test]
fn disable_drops_a_pack_collector_and_its_readers_fail_load() {
    let (_tmp, home, project) = home_and_project();
    write_file(&project, "pack.toml", PACK_COLLECTOR);
    write_file(
        &project,
        ".mindtape/config.toml",
        "extends = [\"./pack.toml\"]\ndisable = [\"changes\"]\n\n[[tasks.rules]]\n\
         name = \"r\"\nassert = 'len(derived.changes) > 0'\n",
    );
    let err = load_config_layered_from(&project, Some(&home), None).unwrap_err();
    assert!(err.to_string().contains("derived.changes"), "{err}");
}

#[test]
fn every_type_atom_describes_itself() {
    use crate::config::CollectorAtom::*;
    for (atom, text) in [
        (Bool, "bool"),
        (Int, "int"),
        (Number, "number"),
        (String, "string"),
        (Path, "path"),
        (Date, "date"),
    ] {
        assert_eq!(atom.as_str(), text);
        assert_eq!(CollectorType::Atom(atom).describe(), text);
    }
    let list = CollectorType::List(Box::new(CollectorType::Atom(CollectorAtom::Path)));
    assert_eq!(list.describe(), "[path]");
    assert!(list.is_path_list());
    assert!(!list.is_scalar());
    let record = CollectorType::Record(std::collections::BTreeMap::from([(
        "count".to_owned(),
        CollectorType::Atom(CollectorAtom::Number),
    )]));
    assert_eq!(record.describe(), "{ count = number }");
    assert_eq!(
        record.field("count"),
        Some(&CollectorType::Atom(CollectorAtom::Number))
    );
    assert_eq!(record.field("missing"), None);
    assert_eq!(CollectorType::Atom(CollectorAtom::Int).field("x"), None);
    assert_eq!(format!("{record}"), record.describe());
    let spec = CollectorSpec {
        name: "x".to_owned(),
        when: None,
        query: CollectorQuery::Regex(RegexQuery {
            section: SectionSpec::Name("Changes".to_owned()),
            pattern: "x".to_owned(),
            group: None,
        }),
        returns: record,
        root: None,
    };
    assert_eq!(crate::returns_of(&spec), "{ count = number }");
}

#[test]
fn accepts_matrix_checks_every_atom() {
    use crate::config::CollectorAtom::*;
    use typst_harvest::HVal;
    let atom = |name| CollectorType::Atom(name);
    assert!(atom(Bool).accepts(&HVal::Bool(true)));
    assert!(!atom(Bool).accepts(&HVal::Int(1)));
    assert!(atom(Int).accepts(&HVal::Int(1)));
    assert!(!atom(Int).accepts(&HVal::Float(1.0)));
    assert!(atom(Number).accepts(&HVal::Int(1)));
    assert!(atom(Number).accepts(&HVal::Float(1.5)));
    assert!(atom(String).accepts(&HVal::Str("x".to_owned())));
    assert!(!atom(String).accepts(&HVal::Int(1)));
    assert!(atom(Path).accepts(&HVal::Str("src/a".to_owned())));
    assert!(atom(Date).accepts(&HVal::Str("2026-11-04".to_owned())));
    assert!(!atom(Date).accepts(&HVal::Str("not a date".to_owned())));
    let list = CollectorType::List(Box::new(atom(Int)));
    assert!(list.accepts(&HVal::Array(vec![HVal::Int(1), HVal::Int(2)])));
    assert!(!list.accepts(&HVal::Array(vec![HVal::Int(1), HVal::Str("x".to_owned())])));
    assert!(!list.accepts(&HVal::Int(1)));
    let record = CollectorType::Record(std::collections::BTreeMap::from([(
        "count".to_owned(),
        atom(Int),
    )]));
    assert!(
        record.accepts(&HVal::Dict(std::collections::BTreeMap::from([(
            "count".to_owned(),
            HVal::Int(1)
        )])))
    );
    assert!(!record.accepts(&HVal::Dict(std::collections::BTreeMap::new())));
    assert!(!record.accepts(&HVal::Int(1)));
}

#[test]
fn a_section_may_be_a_path() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = ["AS-IS", "Detail"], pattern = "x" }
returns = ["string"]
"#,
    );
    let spec = &config.tasks.collectors[0];
    assert!(matches!(&spec.query, CollectorQuery::Regex(query) if query.section.path().len() == 2));
}

#[test]
fn a_list_type_takes_exactly_one_element() {
    for returns in ["[]", "[\"string\", \"path\"]"] {
        let err = parse_config(&format!(
            "[[tasks.collectors]]\nname = \"x\"\nquery = {{ section = \"Changes\", pattern = \"x\" }}\nreturns = {returns}\n"
        ))
        .unwrap_err();
        assert!(err.to_string().contains("exactly one element"), "{err}");
    }
}

#[test]
fn an_invalid_collector_name_is_refused() {
    for name in ["", "1x", "a.b"] {
        let config = config(&format!(
            "[[tasks.collectors]]\nname = \"{name}\"\nquery = {{ section = \"Changes\", pattern = \"x\" }}\nreturns = [\"string\"]\n"
        ));
        let problem = config.validate().unwrap_err();
        assert!(
            problem.text.contains("not a field name"),
            "{name}: {problem}"
        );
    }
}

#[test]
fn an_empty_section_a_bad_regex_and_a_bad_when_are_refused() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
when = "not a filter ("
query = { section = [], pattern = "[" }
returns = ["string"]
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("must name a section"), "{problem}");
    assert!(problem.text.contains("invalid regex"), "{problem}");
    assert!(problem.text.contains("when"), "{problem}");
}

#[test]
fn a_section_path_with_a_bad_regex_is_refused() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { section = [{ match = "[" }], pattern = "x" }
returns = ["string"]
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("query.section"), "{problem}");
}

#[test]
fn an_undeclared_derived_field_fails_in_every_parsed_filter() {
    for (label, text) in [
        (
            "a view",
            format!("{CHANGES}\n[views.bad]\nfilter = 'has(derived.nope)'\n"),
        ),
        (
            "a body rule",
            format!(
                "{CHANGES}\n[[tasks.body.content.rules]]\npath = [\"Changes\"]\nassert = \"non-empty\"\nwhen = 'has(derived.nope)'\n"
            ),
        ),
        (
            "an ls rule",
            format!("{CHANGES}\n[[ls.rules]]\nfilter = 'has(derived.nope)'\n"),
        ),
        (
            "rm",
            format!("{CHANGES}\n[rm]\ndefault-filter = 'has(derived.nope)'\n"),
        ),
    ] {
        let config = config(&text);
        let problem = config.validate().unwrap_err();
        assert!(problem.text.contains("derived.nope"), "{label}: {problem}");
    }
}

#[test]
fn a_when_reading_its_own_field_through_a_view_is_refused() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "changes"
when = "@self"
query = { section = "Changes", pattern = "x" }
returns = ["path"]

[views.self]
filter = "has(derived.changes)"
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(problem.text.contains("declaration order"), "{problem}");
}

#[test]
fn a_typst_collector_parses_and_accepts_every_return_type() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "title"
query = { typst = "lib.typ", function = "title", field = "title" }
returns = "string"

[[tasks.collectors]]
name = "summary"
query = { typst = "lib.typ", function = "summary" }
returns = { count = "int", files = ["path"] }
"#,
    );
    assert_eq!(config.tasks.collectors[0].query.kind(), "typst");
    assert!(config.validate().is_ok(), "{:?}", config.validate());
}

#[test]
fn an_ast_grep_collector_parses() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "callers"
query = { ast_grep = { language = "rust", pattern = "parse_task($$$ARGS)", glob = "src/**" } }
returns = ["path"]
"#,
    );
    let spec = &config.tasks.collectors[0];
    assert_eq!(spec.query.kind(), "ast-grep");
    assert!(matches!(
        &spec.query,
        CollectorQuery::AstGrep(query)
            if query.ast_grep.language == "rust"
                && query.ast_grep.pattern == "parse_task($$$ARGS)"
                && query.ast_grep.glob.as_deref() == Some("src/**")
    ));
    assert!(config.validate().is_ok(), "{:?}", config.validate());
}

#[test]
fn an_ast_grep_collector_takes_only_a_path_list() {
    for returns in [r#""string""#, r#"["string"]"#, r#""path""#] {
        let problem = refusal(&format!(
            "[[tasks.collectors]]\nname = \"x\"\nquery = {{ ast_grep = {{ language = \"rust\", pattern = \"fn $N()\" }} }}\nreturns = {returns}\n"
        ));
        assert!(problem.text.contains("returns"), "{returns}: {problem}");
    }
}

#[test]
fn an_ast_grep_pattern_language_and_glob_are_checked_at_load() {
    for (field, body) in [
        ("pattern", r#"language = "rust", pattern = "fn (""#),
        ("language", r#"language = "python", pattern = "x = 1""#),
        (
            "glob",
            r#"language = "rust", pattern = "fn $N()", glob = "[""#,
        ),
    ] {
        let problem = refusal(&format!(
            "[[tasks.collectors]]\nname = \"x\"\nquery = {{ ast_grep = {{ {body} }} }}\nreturns = [\"path\"]\n"
        ));
        assert!(problem.text.contains(field), "{field}: {problem}");
    }
}

#[test]
fn an_empty_typst_module_or_function_is_refused() {
    let config = config(
        r#"
[[tasks.collectors]]
name = "x"
query = { typst = "", function = "" }
returns = "string"
"#,
    );
    let problem = config.validate().unwrap_err();
    assert!(
        problem.text.contains("must name a Typst module"),
        "{problem}"
    );
    assert!(problem.text.contains("must name a function"), "{problem}");
}
