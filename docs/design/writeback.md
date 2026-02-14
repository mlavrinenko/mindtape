# Write-Back Strategy

## Overview

MindTape supports safe mutation of `.typ` files while preserving formatting and comments.
The database is still a derived index — Typst files remain the source of truth.

## Architecture

```
User Command → Store Lookup → Conflict Check → AST Parse → Modify → Write → Re-index
                    ↓              ↓               ↓          ↓        ↓
              find_task_by_id  hash_file()  typst-syntax  toggle   atomic
```

## Safe Modification Strategy

**Approach**: AST-based text replacement

1. Parse file with `typst_syntax::parse()` to get `Source`
2. Walk AST to find `ListItem` containing task ID
3. Extract text range for that list item
4. Modify checkbox pattern in substring (`[ ]` ↔ `[x]`)
5. Reconstruct file: `before + modified + after`
6. Write atomically with `std::fs::write`

See `crates/mindtape-eval/src/write.rs` for implementation.

### Key Functions

- `load_source(path)` — read and parse `.typ` file
- `find_task_node(source, root, task_id)` — locate task in AST by ID
- `toggle_task_checkbox(source, task_id)` — modify checkbox state

### Format Preservation

Uses `SyntaxNode::range()` to get exact byte offsets. Replaces only the checkbox
pattern, preserving:

- Indentation and whitespace
- Comments
- All surrounding content
- Line breaks

## Task ID Masking

Users don't need to type full UUIDs. Masked patterns like `*37f8` match any task
whose ID ends with that suffix.

**Algorithm**:

```rust
if id_or_mask.starts_with('*') {
    // Suffix pattern: WHERE id LIKE '%37f8'
} else {
    // Exact match: WHERE id = 'full-uuid'
}
```

Returns error if pattern matches 0 or 2+ tasks (ambiguous).

## Conflict Detection

Before writing, verify the file hasn't changed since last index:

1. Get stored `eval_hash` from database (SHA-256 of file content)
2. Compute current file hash
3. If mismatch → error: "File changed on disk since last index. Re-index first."

This prevents overwriting user edits with stale modifications.

## Commands

**Implemented**:
- `mindtape check <task-id>` — toggle task checkbox
- `mindtape check *37f8` — toggle task with ID ending in "37f8"

**Planned (M3.2)**:
- `mindtape set <task-id> --due <date>` — update task due date
- `mindtape set <task-id> --tag <tag>` — add/remove tags

## Open Questions

- Should we use `tempfile` for atomic writes instead of `std::fs::write`?
- How to handle concurrent modifications (file changed between conflict check and write)?
- Should we support batch operations (check multiple tasks at once)?
