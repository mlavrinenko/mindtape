#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{parse_headings, tree};

#[test]
fn test_parse_headings_depth_and_text() {
    let headings = parse_headings("= Top\n== Sub one\n=== Deep\n");
    assert_eq!(headings.len(), 3);
    assert_eq!(headings[0].depth, 1);
    assert_eq!(headings[0].text, "Top");
    assert_eq!(headings[1].depth, 2);
    assert_eq!(headings[1].text, "Sub one");
    assert_eq!(headings[2].depth, 3);
    assert_eq!(headings[2].text, "Deep");
}

#[test]
fn test_parse_headings_line_numbers() {
    let headings = parse_headings("= One\n\n== Two\n");
    assert_eq!(headings[0].line, 1);
    assert_eq!(headings[1].line, 3);
}

#[test]
fn test_parse_headings_skips_code_and_body() {
    let src = "#import \"x\": *\n#show: f\n\n== Summary\nprose here\n=== Detail\n";
    let headings = parse_headings(src);
    assert_eq!(headings.len(), 2);
    assert_eq!(headings[0].text, "Summary");
    assert_eq!(headings[0].depth, 2);
    assert_eq!(headings[1].text, "Detail");
}

#[test]
fn test_no_headings() {
    assert!(parse_headings("just prose, no headings\n").is_empty());
}

#[test]
fn test_tree_nesting() {
    let roots = tree(parse_headings("= A\n== B\n== C\n=== D\n= E\n"));
    assert_eq!(roots.len(), 2);
    assert_eq!(roots[0].heading.text, "A");
    assert_eq!(roots[0].children.len(), 2);
    assert_eq!(roots[0].children[1].heading.text, "C");
    assert_eq!(roots[0].children[1].children[0].heading.text, "D");
    assert_eq!(roots[1].heading.text, "E");
    assert!(roots[1].children.is_empty());
}

#[test]
fn test_tree_depth_jump() {
    let roots = tree(parse_headings("== Summary\n==== Deep\n"));
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].children.len(), 1);
    assert_eq!(roots[0].children[0].heading.text, "Deep");
}
