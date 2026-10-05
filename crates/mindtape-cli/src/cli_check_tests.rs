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
