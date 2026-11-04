#![allow(clippy::unwrap_used)]

use typst_harvest::HVal;

use super::{Query, parse, path_filter};
use crate::config::parse_config;
use crate::test_support::{git_tempdir, write_file};

fn hval(pairs: &[(&str, &str)]) -> HVal {
    HVal::Dict(
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), HVal::Str((*value).to_owned())))
            .collect(),
    )
}

#[test]
fn parses_each_query_tag() {
    assert_eq!(
        parse(&hval(&[
            ("query", "grep"),
            ("regex", "fn"),
            ("glob", "src/**"),
        ])),
        Some(Query::Grep {
            regex: "fn".to_owned(),
            glob: Some("src/**".to_owned()),
        })
    );
    assert_eq!(
        parse(&hval(&[("query", "glob"), ("pattern", "docs/*")])),
        Some(Query::Glob {
            pattern: "docs/*".to_owned(),
        })
    );
    assert_eq!(parse(&hval(&[("query", "other")])), None);
    assert_eq!(parse(&hval(&[("query", "grep")])), None);
    assert_eq!(parse(&hval(&[("query", "glob")])), None);
}

#[test]
fn labels_a_grep_with_its_optional_scope() {
    assert_eq!(
        Query::Grep {
            regex: "fn".to_owned(),
            glob: None,
        }
        .label(),
        "grep `fn`"
    );
    assert_eq!(
        Query::Grep {
            regex: "fn".to_owned(),
            glob: Some("src/**".to_owned()),
        }
        .label(),
        "grep `fn` in `src/**`"
    );
    assert_eq!(
        Query::Glob {
            pattern: "docs/*".to_owned(),
        }
        .label(),
        "glob `docs/*`"
    );
}

#[test]
fn an_unknown_path_set_is_a_problem() {
    let config = parse_config("[dicts.source]\nvalues = [\"src/**\"]\n").unwrap();
    assert_eq!(
        path_filter("@sources", config.compiled_dicts()).unwrap_err(),
        "no path set is named `@sources`"
    );
}

#[test]
fn a_glob_query_reads_the_listed_paths_inside_its_set() {
    let dir = git_tempdir();
    write_file(dir.path(), "src/a.rs", "");
    write_file(dir.path(), "docs/a.rs", "");
    let config = parse_config("").unwrap();
    let dicts = config.compiled_dicts();
    let admitted = path_filter("src/**", dicts).unwrap();
    let query = Query::Glob {
        pattern: "**/*.rs".to_owned(),
    };
    let matched = super::matches(dir.path(), "src/**", &query, &admitted, dicts).unwrap();
    assert_eq!(&*matched, ["src/a.rs"]);
}

#[test]
fn the_pinned_rust_grammar_compiles_and_matches() {
    use ast_grep_core::Pattern;
    use ast_grep_language::{LanguageExt, Rust};

    let pattern = Pattern::try_new("parse_task($$$ARGS)", Rust).unwrap();
    let root = Rust.ast_grep("fn a() { parse_task(1, 2); }");
    assert!(root.root().find(&pattern).is_some());
}
