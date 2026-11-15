use super::*;

#[test]
fn split_words_keeps_quotes_together() {
    assert_eq!(
        split_words(r#"mt ls 'a "b"' "c d" '' e"#),
        ["mt", "ls", r#"a "b""#, "c d", "", "e"]
    );
}

#[test]
fn placeholders_are_stand_ins() {
    for w in ["<cmd>", "TERM", "FILTER_X", "...", "*", "a…"] {
        assert!(is_placeholder(w), "{w}");
    }
    for w in ["ls", "Ls", "", "--json", "X1"] {
        assert!(!is_placeholder(w), "{w}");
    }
}

#[test]
fn a_line_that_is_not_mt_is_refused() {
    assert!(check("cargo build", &[]).is_err());
}

#[test]
fn a_refusal_keeps_only_its_first_line() {
    let err = Cli::command()
        .try_get_matches_from(["mt", "ls", "--limit", "abc"])
        .expect_err("a non-numeric --limit is refused");
    let line = first_line(&err);
    assert!(!line.starts_with("error: "), "{line}");
    assert!(!line.contains('\n'), "{line}");
}
