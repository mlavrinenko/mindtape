//! Write-back functionality for modifying `.typ` files.
//!
//! This module provides safe mutation of Typst files while preserving
//! formatting, comments, and surrounding content. It uses the `typst-syntax`
//! crate to parse files and locate specific tasks by ID, then performs
//! targeted text replacements.

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
}
