//! Write-back functionality for modifying `.typ` files.
//!
//! This module provides safe mutation of Typst files while preserving
//! formatting, comments, and surrounding content. It uses the `typst-syntax`
//! crate to parse files and locate specific tasks by ID, then performs
//! targeted text replacements.
//!
//! ## Operations
//!
//! - **Checkbox toggle**: `toggle_task_checkbox` (M3.1)
//! - **Due date**: `set_task_due` / `remove_task_due` (M3.2)
//! - **Tags**: `add_task_tag` / `remove_task_tag` (M3.2)

use std::path::Path;
use thiserror::Error;
use typst_syntax::{LinkedNode, Source, SyntaxKind};

/// Errors that can occur during write-back operations.
#[derive(Debug, Error)]
pub enum WriteError {
    /// The source file could not be read.
    #[error("file read error: {0}")]
    File(String),

    /// The task ID was not found in the file.
    #[error("task not found: {0}")]
    TaskNotFound(String),

    /// The task node is malformed or cannot be modified.
    #[error("invalid task structure: {0}")]
    InvalidTask(String),

    /// A date string could not be parsed.
    #[error("invalid date: {0}")]
    InvalidDate(String),
}

/// Load a `.typ` file and parse it into a `Source`.
///
/// # Errors
/// Returns `Err` if the file cannot be read.
pub fn load_source(path: &Path) -> Result<Source, WriteError> {
    std::fs::read_to_string(path)
        .map_err(|e| WriteError::File(e.to_string()))
        .map(|content| Source::detached(&content))
}

/// Find a task's `ListItem` node in the syntax tree by searching for its ID.
///
/// Walks the entire tree looking for list items that contain `#id("...")` metadata.
/// Returns the `LinkedNode` for the first matching list item.
///
/// # Errors
/// Returns `None` if no matching task is found.
pub fn find_task_node<'a>(source: &'a Source, root: &LinkedNode<'a>, task_id: &str) -> Option<LinkedNode<'a>> {
    // Walk the tree looking for ListItem nodes
    walk_tree(root, &mut |node| {
        if node.kind() == SyntaxKind::ListItem {
            // Extract text of this list item using source range
            let range = node.range();
            let text = &source.text()[range];

            // Look for #id("...") pattern
            if let Some(id) = extract_id_from_text(text) {
                if id == task_id {
                    return Some(node.clone());
                }
            }
        }
        None
    })
}

/// Walk the syntax tree depth-first, calling `visitor` on each node.
/// Returns the first `Some(T)` value returned by the visitor.
fn walk_tree<'a, T>(
    node: &LinkedNode<'a>,
    visitor: &mut dyn FnMut(LinkedNode<'a>) -> Option<T>,
) -> Option<T> {
    // Try current node
    if let Some(result) = visitor(node.clone()) {
        return Some(result);
    }

    // Recurse into children
    for child in node.children() {
        if let Some(result) = walk_tree(&child, visitor) {
            return Some(result);
        }
    }

    None
}

/// Extract the task ID from a text string containing `#id("...")`.
///
/// Simple pattern matching for the ID — looks for `#id("` followed by
/// characters until the closing `")`.
fn extract_id_from_text(text: &str) -> Option<String> {
    let prefix = "#id(\"";
    let suffix = "\")";

    if let Some(start) = text.find(prefix) {
        let id_start = start + prefix.len();
        if let Some(end) = text[id_start..].find(suffix) {
            return Some(text[id_start..id_start + end].to_string());
        }
    }

    None
}

/// Toggle the checkbox state of a task in the source text.
///
/// Finds the task by ID, determines its current checkbox state (`[ ]` or `[x]`),
/// and returns the modified source text with the checkbox toggled.
///
/// # Errors
/// Returns `Err` if the task is not found or cannot be modified.
pub fn toggle_task_checkbox(source: &Source, task_id: &str) -> Result<String, WriteError> {
    let root = LinkedNode::new(source.root());
    let task_node = find_task_node(source, &root, task_id)
        .ok_or_else(|| WriteError::TaskNotFound(task_id.to_string()))?;

    // Get the text range of this list item
    let range = task_node.range();
    let item_text = &source.text()[range.clone()];

    // Determine current checkbox state and compute replacement
    let (old_checkbox, new_checkbox) = if item_text.trim_start().starts_with("- [ ]") {
        ("- [ ]", "- [x]")
    } else if item_text.trim_start().starts_with("- [x]") || item_text.trim_start().starts_with("- [X]") {
        // Handle both lowercase and uppercase X
        if item_text.trim_start().starts_with("- [x]") {
            ("- [x]", "- [ ]")
        } else {
            ("- [X]", "- [ ]")
        }
    } else {
        return Err(WriteError::InvalidTask(
            "list item does not start with checkbox".to_string(),
        ));
    };

    // Replace the checkbox (replacen preserves the rest of the line)
    let new_item_text = item_text.replacen(old_checkbox, new_checkbox, 1);

    // Reconstruct the full source
    let before = &source.text()[..range.start];
    let after = &source.text()[range.end..];
    Ok(format!("{before}{new_item_text}{after}"))
}

// ---------------------------------------------------------------------------
// M3.2 — Task property updates
// ---------------------------------------------------------------------------

/// Apply a text transformation to a task's `ListItem` text, returning the
/// modified full source.
fn modify_task_text(
    source: &Source,
    task_id: &str,
    modify: impl FnOnce(&str) -> Result<String, WriteError>,
) -> Result<String, WriteError> {
    let root = LinkedNode::new(source.root());
    let task_node = find_task_node(source, &root, task_id)
        .ok_or_else(|| WriteError::TaskNotFound(task_id.to_string()))?;

    let range = task_node.range();
    let item_text = &source.text()[range.clone()];
    let new_item_text = modify(item_text)?;

    let before = &source.text()[..range.start];
    let after = &source.text()[range.end..];
    Ok(format!("{before}{new_item_text}{after}"))
}

/// Set or update the due date on a task.
///
/// If the task already has a `#due(datetime(...))` call, it is replaced.
/// Otherwise a new one is inserted before `#id("...")`.
///
/// `date_str` must be in `YYYY-MM-DD` format.
///
/// # Errors
/// Returns `Err` if the task is not found, the date is invalid, or the task
/// has no `#id(...)` for insertion.
pub fn set_task_due(source: &Source, task_id: &str, date_str: &str) -> Result<String, WriteError> {
    let dt = format_typst_datetime(date_str)?;
    let replacement = format!("#due({dt})");

    modify_task_text(source, task_id, |text| {
        if let Some((start, end)) = find_due_span(text) {
            // Replace existing due call
            Ok(format!("{}{replacement}{}", &text[..start], &text[end..]))
        } else if let Some(insert) = find_property_insert_point(text) {
            // Insert before #id(...): "...text {replacement} #id(...)"
            Ok(format!("{}{replacement} {}", &text[..insert], &text[insert..]))
        } else {
            Err(WriteError::InvalidTask(
                "task has no #id() — cannot determine insertion point".to_string(),
            ))
        }
    })
}

/// Remove the due date from a task.
///
/// If the task has no `#due(...)`, this is a no-op that returns the source
/// unchanged.
///
/// # Errors
/// Returns `Err` if the task is not found.
pub fn remove_task_due(source: &Source, task_id: &str) -> Result<String, WriteError> {
    modify_task_text(source, task_id, |text| {
        if let Some((start, end)) = find_due_span(text) {
            let (start, end) = trim_leading_space(text, start, end);
            Ok(format!("{}{}", &text[..start], &text[end..]))
        } else {
            // No due date — return as-is
            Ok(text.to_string())
        }
    })
}

/// Add a tag to a task.
///
/// Inserts `#tag("name")` before `#id("...")`.
/// Does **not** check for duplicates — callers should verify if needed.
///
/// # Errors
/// Returns `Err` if the task is not found or has no `#id(...)`.
pub fn add_task_tag(source: &Source, task_id: &str, tag: &str) -> Result<String, WriteError> {
    let new_tag = format!("#tag(\"{tag}\")");

    modify_task_text(source, task_id, |text| {
        if let Some(insert) = find_property_insert_point(text) {
            Ok(format!("{}{new_tag} {}", &text[..insert], &text[insert..]))
        } else {
            Err(WriteError::InvalidTask(
                "task has no #id() — cannot determine insertion point".to_string(),
            ))
        }
    })
}

/// Remove a specific tag from a task.
///
/// If the tag is not present, this is a no-op that returns the source
/// unchanged.
///
/// # Errors
/// Returns `Err` if the task is not found.
pub fn remove_task_tag(source: &Source, task_id: &str, tag: &str) -> Result<String, WriteError> {
    modify_task_text(source, task_id, |text| {
        if let Some((start, end)) = find_tag_span(text, tag) {
            let (start, end) = trim_leading_space(text, start, end);
            Ok(format!("{}{}", &text[..start], &text[end..]))
        } else {
            // Tag not present — return as-is
            Ok(text.to_string())
        }
    })
}

// ---------------------------------------------------------------------------
// Helpers for property span detection
// ---------------------------------------------------------------------------

/// Find the byte span of `#due(datetime(...))` in `text`.
///
/// Handles nested parentheses to correctly match the outer `#due(...)` call.
fn find_due_span(text: &str) -> Option<(usize, usize)> {
    let prefix = "#due(";
    let start = text.find(prefix)?;
    let end = find_matching_paren(text, start + prefix.len() - 1)?;
    Some((start, end + 1))
}

/// Find the byte span of `#tag("name")` for a specific tag value.
fn find_tag_span(text: &str, tag: &str) -> Option<(usize, usize)> {
    let needle = format!("#tag(\"{tag}\")");
    let start = text.find(&needle)?;
    Some((start, start + needle.len()))
}

/// Find the byte offset just before `#id("...")` — the insertion point for
/// new properties.
fn find_property_insert_point(text: &str) -> Option<usize> {
    text.find("#id(\"")
}

/// If the character before `start` is a space, extend the span left by 1
/// to avoid double spaces after removal.
fn trim_leading_space(text: &str, start: usize, end: usize) -> (usize, usize) {
    if start > 0 && text.as_bytes().get(start - 1) == Some(&b' ') {
        (start - 1, end)
    } else {
        (start, end)
    }
}

/// Find the index of the closing `)` that matches the opening `(` at `open`.
///
/// Returns `None` if parentheses are unbalanced.
fn find_matching_paren(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth: u32 = 1;
    for (i, &byte) in bytes.iter().enumerate().skip(open + 1) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse an ISO date string (`YYYY-MM-DD`) and format it as a Typst
/// `datetime(Y, M, D)` call.
///
/// # Errors
/// Returns `Err(WriteError::InvalidDate)` if the string is malformed.
fn format_typst_datetime(date_str: &str) -> Result<String, WriteError> {
    let parts: Vec<&str> = date_str.split('-').collect();
    let [year_str, month_str, day_str] = parts.as_slice() else {
        return Err(WriteError::InvalidDate(format!(
            "expected YYYY-MM-DD, got \"{date_str}\""
        )));
    };

    let year: i32 = year_str
        .parse()
        .map_err(|_| WriteError::InvalidDate(format!("invalid year in \"{date_str}\"")))?;
    let month: u32 = month_str
        .parse()
        .map_err(|_| WriteError::InvalidDate(format!("invalid month in \"{date_str}\"")))?;
    let day: u32 = day_str
        .parse()
        .map_err(|_| WriteError::InvalidDate(format!("invalid day in \"{date_str}\"")))?;

    if !(1..=12).contains(&month) {
        return Err(WriteError::InvalidDate(format!(
            "month {month} out of range 1..12"
        )));
    }
    if !(1..=31).contains(&day) {
        return Err(WriteError::InvalidDate(format!(
            "day {day} out of range 1..31"
        )));
    }

    Ok(format!("datetime({year}, {month}, {day})"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
    fn test_format_typst_datetime() {
        assert_eq!(
            format_typst_datetime("2026-03-15").unwrap(),
            "datetime(2026, 3, 15)"
        );
    }

    #[test]
    fn test_format_typst_datetime_single_digit() {
        assert_eq!(
            format_typst_datetime("2026-1-5").unwrap(),
            "datetime(2026, 1, 5)"
        );
    }

    #[test]
    fn test_format_typst_datetime_invalid() {
        assert!(format_typst_datetime("not-a-date").is_err());
        assert!(format_typst_datetime("2026-13-01").is_err());
        assert!(format_typst_datetime("2026-01-32").is_err());
        assert!(format_typst_datetime("2026-00-15").is_err());
        assert!(format_typst_datetime("2026-01-00").is_err());
    }

    #[test]
    fn test_find_due_span() {
        let text = "- [ ] Task #due(datetime(2026, 3, 15)) #id(\"t1\")";
        let (start, end) = find_due_span(text).unwrap();
        assert_eq!(&text[start..end], "#due(datetime(2026, 3, 15))");
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
        let text = "#due(datetime(2026, 3, 15))";
        // opening paren at index 4
        assert_eq!(find_matching_paren(text, 4), Some(26));
    }

    #[test]
    fn test_find_matching_paren_unbalanced() {
        let text = "#due(datetime(2026, 3, 15)";
        assert_eq!(find_matching_paren(text, 4), None);
    }

    // --- M3.2 set_task_due tests ---

    #[test]
    fn test_set_due_insert_new() {
        let content = "- [ ] Buy milk #id(\"task-1\")";
        let source = Source::detached(content);
        let result = set_task_due(&source, "task-1", "2026-03-15").unwrap();
        assert_eq!(
            result,
            "- [ ] Buy milk #due(datetime(2026, 3, 15)) #id(\"task-1\")"
        );
    }

    #[test]
    fn test_set_due_replace_existing() {
        let content = "- [ ] Buy milk #due(datetime(2026, 1, 1)) #id(\"task-1\")";
        let source = Source::detached(content);
        let result = set_task_due(&source, "task-1", "2026-03-15").unwrap();
        assert_eq!(
            result,
            "- [ ] Buy milk #due(datetime(2026, 3, 15)) #id(\"task-1\")"
        );
    }

    #[test]
    fn test_set_due_with_tags() {
        let content =
            "- [ ] Task #tag(\"work\") #id(\"task-1\")";
        let source = Source::detached(content);
        let result = set_task_due(&source, "task-1", "2026-06-01").unwrap();
        assert_eq!(
            result,
            "- [ ] Task #tag(\"work\") #due(datetime(2026, 6, 1)) #id(\"task-1\")"
        );
    }

    #[test]
    fn test_set_due_preserves_other_tasks() {
        let content = r#"
- [ ] First #id("task-1")
- [ ] Second #due(datetime(2026, 1, 1)) #id("task-2")
"#;
        let source = Source::detached(content);
        let result = set_task_due(&source, "task-2", "2026-12-25").unwrap();
        assert!(result.contains("- [ ] First #id(\"task-1\")"));
        assert!(result.contains("- [ ] Second #due(datetime(2026, 12, 25)) #id(\"task-2\")"));
    }

    // --- M3.2 remove_task_due tests ---

    #[test]
    fn test_remove_due() {
        let content = "- [ ] Task #due(datetime(2026, 3, 15)) #id(\"task-1\")";
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
        let content =
            "- [ ] Task #due(datetime(2026, 3, 15)) #tag(\"work\") #id(\"task-1\")";
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
        let content = "- [ ] Task #due(datetime(2026, 3, 15)) #id(\"task-1\")";
        let source = Source::detached(content);
        let result = add_task_tag(&source, "task-1", "home").unwrap();
        assert_eq!(
            result,
            "- [ ] Task #due(datetime(2026, 3, 15)) #tag(\"home\") #id(\"task-1\")"
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
        let content =
            "- [ ] Task #tag(\"work\") #tag(\"urgent\") #id(\"task-1\")";
        let source = Source::detached(content);
        let result = remove_task_tag(&source, "task-1", "work").unwrap();
        assert_eq!(
            result,
            "- [ ] Task #tag(\"urgent\") #id(\"task-1\")"
        );
    }

    #[test]
    fn test_remove_tag_preserves_due() {
        let content =
            "- [ ] Task #due(datetime(2026, 3, 15)) #tag(\"work\") #id(\"task-1\")";
        let source = Source::detached(content);
        let result = remove_task_tag(&source, "task-1", "work").unwrap();
        assert_eq!(
            result,
            "- [ ] Task #due(datetime(2026, 3, 15)) #id(\"task-1\")"
        );
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
}
