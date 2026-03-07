//! Tests for write.rs — kept in a separate file to stay under line limits.
//!
//! Uses `#[path = "write_tests.rs"]` to attach to the parent module,
//! preserving access to private functions.

use super::*;

#[test]
fn test_extract_id_from_text() {
    let text = r#"- [ ] Task text #id("abc-123") #tag("foo")"#;
    assert_eq!(extract_id_from_text(text), Some("abc-123".to_string()));
}

#[test]
fn test_extract_id_no_match() {
    let text = "- [ ] Task without ID";
    assert_eq!(extract_id_from_text(text), None);
}

#[test]
fn test_toggle_unchecked_to_checked() {
    let content = r#"
= Tasks

- [ ] First task #id("task-1")
- [ ] Second task #id("task-2")
"#;
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "task-1").unwrap();

    assert!(result.contains("- [x] First task #id(\"task-1\")"));
    assert!(result.contains("- [ ] Second task #id(\"task-2\")"));
}

#[test]
fn test_toggle_checked_to_unchecked() {
    let content = r#"
= Tasks

- [x] Completed task #id("task-1")
- [ ] Pending task #id("task-2")
"#;
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "task-1").unwrap();

    assert!(result.contains("- [ ] Completed task #id(\"task-1\")"));
    assert!(result.contains("- [ ] Pending task #id(\"task-2\")"));
}

#[test]
fn test_toggle_uppercase_x() {
    let content = "- [X] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "task-1").unwrap();

    assert!(result.contains("- [ ] Task #id(\"task-1\")"));
}

#[test]
fn test_task_not_found() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "nonexistent");

    assert!(matches!(result, Err(WriteError::TaskNotFound(_))));
}

#[test]
fn test_preserves_surrounding_content() {
    let content = r#"
#import "@mindtape/mindtape:0.1.0": id

= My Tasks

Some text before.

- [ ] Task one #id("task-1")
- [ ] Task two #id("task-2")

Some text after.
"#;
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "task-2").unwrap();

    assert!(result.contains("Some text before."));
    assert!(result.contains("Some text after."));
    assert!(result.contains("#import"));
    assert!(result.contains("- [ ] Task one #id(\"task-1\")"));
    assert!(result.contains("- [x] Task two #id(\"task-2\")"));
}

#[test]
fn test_preserves_indentation() {
    let content = r#"
= Tasks

  - [ ] Indented task #id("task-1")
"#;
    let source = Source::detached(content);
    let result = toggle_task_checkbox(&source, "task-1").unwrap();

    // Should preserve the leading spaces
    assert!(result.contains("  - [x] Indented task #id(\"task-1\")"));
}

// --- M3.2 helper tests ---

#[test]
fn test_format_typst_date_args() {
    assert_eq!(format_typst_date_args("2026-03-15").unwrap(), "2026, 3, 15");
}

#[test]
fn test_format_typst_date_args_single_digit() {
    assert_eq!(format_typst_date_args("2026-1-5").unwrap(), "2026, 1, 5");
}

#[test]
fn test_format_typst_date_args_invalid() {
    assert!(format_typst_date_args("not-a-date").is_err());
    assert!(format_typst_date_args("2026-13-01").is_err());
    assert!(format_typst_date_args("2026-01-32").is_err());
    assert!(format_typst_date_args("2026-00-15").is_err());
    assert!(format_typst_date_args("2026-01-00").is_err());
}

#[test]
fn test_find_due_span() {
    let text = "- [ ] Task #due(2026, 3, 15) #id(\"t1\")";
    let (start, end) = find_due_span(text).unwrap();
    assert_eq!(&text[start..end], "#due(2026, 3, 15)");
}

#[test]
fn test_find_due_span_none() {
    let text = "- [ ] Task #id(\"t1\")";
    assert!(find_due_span(text).is_none());
}

#[test]
fn test_find_tag_span() {
    let text = "- [ ] Task #tag(\"work\") #id(\"t1\")";
    let (start, end) = find_tag_span(text, "work").unwrap();
    assert_eq!(&text[start..end], "#tag(\"work\")");
}

#[test]
fn test_find_tag_span_wrong_name() {
    let text = "- [ ] Task #tag(\"work\") #id(\"t1\")";
    assert!(find_tag_span(text, "home").is_none());
}

#[test]
fn test_find_matching_paren() {
    let text = "#due(2026, 3, 15)";
    // opening paren at index 4
    assert_eq!(find_matching_paren(text, 4), Some(16));
}

#[test]
fn test_find_matching_paren_unbalanced() {
    let text = "#due(2026, 3, 15";
    assert_eq!(find_matching_paren(text, 4), None);
}

// --- M3.2 set_task_due tests ---

#[test]
fn test_set_due_insert_new() {
    let content = "- [ ] Buy milk #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_due(&source, "task-1", "2026-03-15").unwrap();
    assert_eq!(result, "- [ ] Buy milk #due(2026, 3, 15) #id(\"task-1\")");
}

#[test]
fn test_set_due_replace_existing() {
    let content = "- [ ] Buy milk #due(2026, 1, 1) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_due(&source, "task-1", "2026-03-15").unwrap();
    assert_eq!(result, "- [ ] Buy milk #due(2026, 3, 15) #id(\"task-1\")");
}

#[test]
fn test_set_due_with_tags() {
    let content = "- [ ] Task #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_due(&source, "task-1", "2026-06-01").unwrap();
    assert_eq!(
        result,
        "- [ ] Task #tag(\"work\") #due(2026, 6, 1) #id(\"task-1\")"
    );
}

#[test]
fn test_set_due_preserves_other_tasks() {
    let content = r#"
- [ ] First #id("task-1")
- [ ] Second #due(2026, 1, 1) #id("task-2")
"#;
    let source = Source::detached(content);
    let result = set_task_due(&source, "task-2", "2026-12-25").unwrap();
    assert!(result.contains("- [ ] First #id(\"task-1\")"));
    assert!(result.contains("- [ ] Second #due(2026, 12, 25) #id(\"task-2\")"));
}

// --- M3.2 remove_task_due tests ---

#[test]
fn test_remove_due() {
    let content = "- [ ] Task #due(2026, 3, 15) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_due(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

#[test]
fn test_remove_due_not_present() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_due(&source, "task-1").unwrap();
    assert_eq!(result, content);
}

#[test]
fn test_remove_due_preserves_tags() {
    let content = "- [ ] Task #due(2026, 3, 15) #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_due(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #tag(\"work\") #id(\"task-1\")");
}

// --- M3.2 add_task_tag tests ---

#[test]
fn test_add_tag() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = add_task_tag(&source, "task-1", "work").unwrap();
    assert_eq!(result, "- [ ] Task #tag(\"work\") #id(\"task-1\")");
}

#[test]
fn test_add_tag_with_existing() {
    let content = "- [ ] Task #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = add_task_tag(&source, "task-1", "urgent").unwrap();
    assert_eq!(
        result,
        "- [ ] Task #tag(\"work\") #tag(\"urgent\") #id(\"task-1\")"
    );
}

#[test]
fn test_add_tag_with_due() {
    let content = "- [ ] Task #due(2026, 3, 15) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = add_task_tag(&source, "task-1", "home").unwrap();
    assert_eq!(
        result,
        "- [ ] Task #due(2026, 3, 15) #tag(\"home\") #id(\"task-1\")"
    );
}

// --- M3.2 remove_task_tag tests ---

#[test]
fn test_remove_tag() {
    let content = "- [ ] Task #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_tag(&source, "task-1", "work").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

#[test]
fn test_remove_tag_not_present() {
    let content = "- [ ] Task #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_tag(&source, "task-1", "home").unwrap();
    assert_eq!(result, content);
}

#[test]
fn test_remove_tag_one_of_many() {
    let content = "- [ ] Task #tag(\"work\") #tag(\"urgent\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_tag(&source, "task-1", "work").unwrap();
    assert_eq!(result, "- [ ] Task #tag(\"urgent\") #id(\"task-1\")");
}

#[test]
fn test_remove_tag_preserves_due() {
    let content = "- [ ] Task #due(2026, 3, 15) #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_tag(&source, "task-1", "work").unwrap();
    assert_eq!(result, "- [ ] Task #due(2026, 3, 15) #id(\"task-1\")");
}

// --- start property ---

#[test]
fn test_find_start_span() {
    let text = "- [ ] Task #start(2026, 3, 15) #id(\"t1\")";
    let (start, end) = find_start_span(text).unwrap();
    assert_eq!(&text[start..end], "#start(2026, 3, 15)");
}

#[test]
fn test_find_start_span_none() {
    let text = "- [ ] Task #id(\"t1\")";
    assert!(find_start_span(text).is_none());
}

#[test]
fn test_set_start_insert_new() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_start(&source, "task-1", "2026-03-15").unwrap();
    assert_eq!(result, "- [ ] Task #start(2026, 3, 15) #id(\"task-1\")");
}

#[test]
fn test_set_start_replace_existing() {
    let content = "- [ ] Task #start(2026, 1, 1) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_start(&source, "task-1", "2026-06-15").unwrap();
    assert_eq!(result, "- [ ] Task #start(2026, 6, 15) #id(\"task-1\")");
}

#[test]
fn test_remove_start() {
    let content = "- [ ] Task #start(2026, 3, 15) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_start(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

#[test]
fn test_remove_start_not_present() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_start(&source, "task-1").unwrap();
    assert_eq!(result, content);
}

#[test]
fn test_set_start_with_due() {
    let content = "- [ ] Task #due(2026, 6, 1) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_start(&source, "task-1", "2026-03-01").unwrap();
    assert_eq!(
        result,
        "- [ ] Task #due(2026, 6, 1) #start(2026, 3, 1) #id(\"task-1\")"
    );
}

// --- rank property ---

#[test]
fn test_find_rank_span() {
    let text = "- [ ] Task #rank(50) #id(\"t1\")";
    let (start, end) = find_rank_span(text).unwrap();
    assert_eq!(&text[start..end], "#rank(50)");
}

#[test]
fn test_find_rank_span_none() {
    let text = "- [ ] Task #id(\"t1\")";
    assert!(find_rank_span(text).is_none());
}

#[test]
fn test_set_rank_insert_new() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_rank(&source, "task-1", 100).unwrap();
    assert_eq!(result, "- [ ] Task #rank(100) #id(\"task-1\")");
}

#[test]
fn test_set_rank_replace_existing() {
    let content = "- [ ] Task #rank(50) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_rank(&source, "task-1", 100).unwrap();
    assert_eq!(result, "- [ ] Task #rank(100) #id(\"task-1\")");
}

#[test]
fn test_remove_rank() {
    let content = "- [ ] Task #rank(50) #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_rank(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

#[test]
fn test_remove_rank_not_present() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_rank(&source, "task-1").unwrap();
    assert_eq!(result, content);
}

#[test]
fn test_set_rank_with_due_and_tags() {
    let content = "- [ ] Task #due(2026, 3, 15) #tag(\"work\") #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_rank(&source, "task-1", 75).unwrap();
    assert_eq!(
        result,
        "- [ ] Task #due(2026, 3, 15) #tag(\"work\") #rank(75) #id(\"task-1\")"
    );
}

// --- Rank alias tests ---

#[test]
fn test_find_rank_span_high_alias() {
    let text = "- [ ] Task #high #id(\"t1\")";
    let (start, end) = find_rank_span(text).unwrap();
    assert_eq!(&text[start..end], "#high");
}

#[test]
fn test_find_rank_span_medium_alias() {
    let text = "- [ ] Task #medium #id(\"t1\")";
    let (start, end) = find_rank_span(text).unwrap();
    assert_eq!(&text[start..end], "#medium");
}

#[test]
fn test_find_rank_span_low_alias() {
    let text = "- [ ] Task #low #id(\"t1\")";
    let (start, end) = find_rank_span(text).unwrap();
    assert_eq!(&text[start..end], "#low");
}

#[test]
fn test_find_rank_span_prefers_rank_over_alias() {
    let text = "- [ ] Task #rank(75) #high #id(\"t1\")";
    let (start, end) = find_rank_span(text).unwrap();
    assert_eq!(&text[start..end], "#rank(75)");
}

#[test]
fn test_set_rank_replaces_high_alias() {
    let content = "- [ ] Task #high #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_rank(&source, "task-1", 50).unwrap();
    assert_eq!(result, "- [ ] Task #rank(50) #id(\"task-1\")");
}

#[test]
fn test_set_rank_replaces_low_alias() {
    let content = "- [ ] Task #low #id(\"task-1\")";
    let source = Source::detached(content);
    let result = set_task_rank(&source, "task-1", 100).unwrap();
    assert_eq!(result, "- [ ] Task #rank(100) #id(\"task-1\")");
}

#[test]
fn test_remove_rank_removes_high_alias() {
    let content = "- [ ] Task #high #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_rank(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

#[test]
fn test_remove_rank_removes_medium_alias() {
    let content = "- [ ] Task #medium #id(\"task-1\")";
    let source = Source::detached(content);
    let result = remove_task_rank(&source, "task-1").unwrap();
    assert_eq!(result, "- [ ] Task #id(\"task-1\")");
}

// --- Combined operations ---

#[test]
fn test_set_due_invalid_date() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    assert!(matches!(
        set_task_due(&source, "task-1", "not-a-date"),
        Err(WriteError::InvalidDate(_))
    ));
}

#[test]
fn test_operations_task_not_found() {
    let content = "- [ ] Task #id(\"task-1\")";
    let source = Source::detached(content);
    assert!(matches!(
        set_task_due(&source, "nope", "2026-01-01"),
        Err(WriteError::TaskNotFound(_))
    ));
    assert!(matches!(
        remove_task_due(&source, "nope"),
        Err(WriteError::TaskNotFound(_))
    ));
    assert!(matches!(
        add_task_tag(&source, "nope", "x"),
        Err(WriteError::TaskNotFound(_))
    ));
    assert!(matches!(
        remove_task_tag(&source, "nope", "x"),
        Err(WriteError::TaskNotFound(_))
    ));
}
