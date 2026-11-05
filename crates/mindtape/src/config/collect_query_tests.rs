#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::config::parse_config;

/// A bad collector query names the key at fault, not serde's untagged-enum
/// message. `truth/tasks/name-the-bad-field-in-a-collector-query-error.typ`.
#[test]
fn a_bad_collector_query_names_the_field() {
    for (label, query, field) in [
        (
            "regex unknown key",
            r#"{ section = "Changes", pattern = "x", globb = "y" }"#,
            "globb",
        ),
        (
            "typst unknown key",
            r#"{ typst = "lib.typ", function = "f", globb = "y" }"#,
            "globb",
        ),
        (
            "ast-grep unknown key",
            r#"{ ast_grep = { language = "rust", pattern = "x", globb = "y" } }"#,
            "globb",
        ),
        ("regex missing key", r#"{ section = "Changes" }"#, "pattern"),
        ("typst missing key", r#"{ typst = "lib.typ" }"#, "function"),
        (
            "ast-grep missing key",
            r#"{ ast_grep = { language = "rust" } }"#,
            "pattern",
        ),
        (
            "regex wrong type",
            r#"{ section = "Changes", pattern = 5 }"#,
            "pattern",
        ),
        (
            "typst wrong type",
            r#"{ typst = "lib.typ", function = 5 }"#,
            "function",
        ),
        (
            "ast-grep wrong type",
            r#"{ ast_grep = { language = "rust", pattern = 5 } }"#,
            "pattern",
        ),
    ] {
        let err = parse_config(&format!(
            "[[tasks.collectors]]\nname = \"x\"\nquery = {query}\nreturns = [\"path\"]\n"
        ))
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains(field), "{label}: {text}");
        assert!(
            !text.contains("did not match any variant"),
            "{label}: {text}"
        );
    }
}
