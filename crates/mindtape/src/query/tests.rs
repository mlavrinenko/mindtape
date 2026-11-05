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

/// The files `query` matches inside `src/**`, over a git tempdir holding
/// `files`.
fn matched(files: &[(&str, &[u8])], query: &Query) -> Result<Vec<String>, String> {
    let dir = git_tempdir();
    for (path, contents) in files {
        write_file(dir.path(), path, contents);
    }
    let config = parse_config("").unwrap();
    let dicts = config.compiled_dicts();
    let admitted = path_filter("src/**", dicts).unwrap();
    super::matches(dir.path(), "src/**", query, &admitted, dicts).map(|files| files.to_vec())
}

#[test]
fn a_glob_query_reads_the_listed_paths_inside_its_set() {
    let query = Query::Glob {
        pattern: "**/*.rs".to_owned(),
    };
    let matched = matched(&[("src/a.rs", b""), ("docs/a.rs", b"")], &query).unwrap();
    assert_eq!(matched, ["src/a.rs"]);
}

#[test]
fn the_pinned_rust_grammar_compiles_and_matches() {
    use ast_grep_core::Pattern;
    use ast_grep_language::{LanguageExt, Rust};

    let pattern = Pattern::try_new("parse_task($$$ARGS)", Rust).unwrap();
    let root = Rust.ast_grep("fn a() { parse_task(1, 2); }");
    assert!(root.root().find(&pattern).is_some());
}

fn ast_grep(language: &str, pattern: &str, glob: Option<&str>) -> Query {
    Query::AstGrep {
        language: language.to_owned(),
        pattern: pattern.to_owned(),
        glob: glob.map(str::to_owned),
    }
}

#[test]
fn parses_an_ast_grep_query() {
    assert_eq!(
        parse(&hval(&[
            ("query", "ast-grep"),
            ("language", "rust"),
            ("pattern", "parse_task($$$ARGS)"),
            ("glob", "crates/**"),
        ])),
        Some(ast_grep("rust", "parse_task($$$ARGS)", Some("crates/**")))
    );
    assert_eq!(
        parse(&hval(&[
            ("query", "ast-grep"),
            ("language", "rust"),
            ("pattern", "fn $NAME()"),
        ])),
        Some(ast_grep("rust", "fn $NAME()", None))
    );
}

#[test]
fn labels_an_ast_grep_query_with_its_language_and_scope() {
    assert_eq!(
        ast_grep("rust", "parse_task($$$ARGS)", None).label(),
        "ast-grep rust `parse_task($$$ARGS)`"
    );
    assert_eq!(
        ast_grep("rust", "parse_task($$$ARGS)", Some("crates/**")).label(),
        "ast-grep rust `parse_task($$$ARGS)` in `crates/**`"
    );
}

#[test]
fn a_bad_pattern_or_unknown_language_is_a_problem() {
    let files = [("src/a.rs", b"fn a() { parse_task(1); }\n".as_slice())];
    let problem = matched(&files, &ast_grep("rust", "fn (", None)).unwrap_err();
    assert!(problem.contains("isn't a pattern"), "{problem}");
    let problem = matched(&files, &ast_grep("python", "x = 1", None)).unwrap_err();
    assert!(problem.contains("supported: rust (alias rs)"), "{problem}");
}

#[test]
fn a_file_matching_twice_counts_once_inside_the_ast_grep_glob() {
    let files = [
        (
            "src/a.rs",
            b"fn a() { parse_task(1); parse_task(2); }".as_slice(),
        ),
        ("src/b.rs", b"fn b() { other(); }\n".as_slice()),
        ("lib/c.rs", b"fn c() { parse_task(3); }\n".as_slice()),
    ];
    let query = ast_grep("rs", "parse_task($$$ARGS)", Some("src/*.rs"));
    assert_eq!(matched(&files, &query).unwrap(), ["src/a.rs"]);
}

#[test]
fn a_non_utf8_file_matches_nothing() {
    let query = ast_grep("rust", "fn $NAME()", None);
    assert!(
        matched(&[("src/a.rs", &[0xff, 0xfe])], &query)
            .unwrap()
            .is_empty()
    );
}
