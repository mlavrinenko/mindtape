# Write-Back Research

## Goal

Implement safe mutation of `.typ` files to update task states while preserving:
- File formatting (whitespace, indentation, blank lines)
- Comments
- Surrounding content
- All other tasks and metadata

## Approach Options

### Option 1: AST-based Modification (Preferred)

Use `typst-syntax` to parse the file into a syntax tree, find the specific
list item by task ID, modify its checkbox state, and serialize back.

**Pros**:
- Preserves formatting and comments (if using `SyntaxNode::text()` ranges)
- Type-safe modification
- Can handle complex cases (nested content, multi-line items)

**Cons**:
- Need to implement serialization that preserves whitespace
- More complex than regex

**Key Types from `typst-syntax`**:
- `Source` — represents a file (text + path)
- `SyntaxNode` — AST node with `kind()`, `text()`, `range()`
- `SyntaxKind` — enum of node types (`ListItem`, `Text`, etc.)
- `LinkedNode` — tree navigation (parent, children, siblings)

**Strategy**:
1. Parse file to `Source` (which contains root `SyntaxNode`)
2. Walk the tree looking for `ListItem` nodes
3. For each list item:
   - Extract text to check if it contains our task ID
   - Check if ID matches (exact or masked)
   - If match: reconstruct the text with toggled checkbox
4. Build new file content by:
   - Keeping all text before the match unchanged
   - Replacing the matched line with modified version
   - Keeping all text after the match unchanged

### Option 2: Line-based Modification with Regex

Read the file line-by-line, use regex to find the line with the target
task ID, modify the checkbox pattern, write back.

**Pros**:
- Simple to implement
- Easy to preserve surrounding lines

**Cons**:
- Brittle if task spans multiple lines
- Risk of false positives if ID appears elsewhere
- Harder to validate we're modifying the right thing

### Option 3: Full Re-render

Re-evaluate the file, reconstruct tasks from DB, write out new `.typ` file
with canonical formatting.

**Pros**:
- Always produces valid Typst
- Can normalize formatting

**Cons**:
- Loses all user formatting, comments, and structure
- NOT acceptable for a file-first tool — defeats the purpose

---

## Chosen Approach: Hybrid (Simple AST + Line Replacement)

**Phase 1** (MVP for M3):
- Use `typst-syntax` to parse and find the task
- Use simple line replacement to modify the checkbox
- Validate via offset ranges that we're modifying the exact location

**Implementation**:
1. Parse file with `typst_syntax::parse()`
2. Walk AST to find the `ListItem` containing task ID
3. Extract the `SyntaxNode::range()` for that item
4. Get the line(s) covered by that range
5. Modify the checkbox in that substring
6. Reconstruct file: `before + modified + after`
7. Write atomically (write to temp, rename)

**Phase 2** (Future enhancement):
- Proper AST rewriting with whitespace preservation
- Handle multi-line tasks, complex formatting
- Support updating other properties (due date, tags)

---

## Task ID Masking

Support partial UUID matching: `*37f8` matches any task whose ID ends with `37f8`.

**Algorithm**:
1. If ID starts with `*`, treat rest as suffix pattern
2. Query DB for tasks with ID ending in that pattern
3. If exactly 1 match → proceed
4. If 0 or 2+ matches → error with helpful message

**Example queries**:
```rust
// Exact match
SELECT * FROM tasks WHERE id = ?

// Suffix match
SELECT * FROM tasks WHERE id LIKE '%' || ?

// Return count first to validate uniqueness
SELECT COUNT(*), id FROM tasks WHERE id LIKE '%' || ? 
```

---

## Conflict Detection

Before writing, check if the file has changed since last index:

1. Get the stored `eval_hash` from DB
2. Compute SHA-256 of current file content
3. Compare hashes:
   - Match → safe to write
   - Mismatch → error: "File changed on disk since last index. Re-index first."

---

## Safe File Writing

Use atomic write pattern:
```rust
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

fn write_file_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap();
    let mut temp = NamedTempFile::new_in(dir)?;
    temp.write_all(content.as_bytes())?;
    temp.persist(path)?;
    Ok(())
}
```

**Benefits**:
- Never leaves file in partial state
- Other processes see either old or new version (never mid-write)
- Automatically cleans up temp file on error

---

## Implementation Plan

### New Module: `crates/mindtape-eval/src/write.rs`

```rust
pub struct WriteError;

/// Find a task in the syntax tree by ID.
pub fn find_task_node(source: &Source, task_id: &str) -> Option<LinkedNode>;

/// Toggle the checkbox state of a task.
pub fn toggle_task_checkbox(source: &Source, task_id: &str) -> Result<String, WriteError>;

/// Update a task's due date.
pub fn update_task_due(source: &Source, task_id: &str, new_due: Datetime) -> Result<String, WriteError>;
```

### Store Extension

Add method to find task by ID (with masking support):

```rust
trait Store {
    /// Find task by exact ID or masked ID (e.g. "*37f8").
    /// Returns error if masked pattern matches 0 or 2+ tasks.
    fn find_task_by_id(&self, id_or_mask: &str) -> Result<TaskWithFile, StoreError>;
}

struct TaskWithFile {
    task_id: String,
    file_path: PathBuf,
    file_hash: String,
    // ... other task fields
}
```

### CLI Extension

```rust
enum Command {
    // ... existing commands
    Check { task_id: String, db_path: Option<PathBuf> },
}
```

---

## Testing Strategy

1. **Unit tests**: Parse, find, modify (in-memory)
2. **Integration tests**: Full cycle (write file, index, check task, verify file)
3. **Conflict tests**: Modify file after index, verify error
4. **Masking tests**: Unique match, ambiguous match, no match

---

## Dependencies

Add to `crates/mindtape-eval/Cargo.toml`:

```toml
[dependencies]
typst-syntax = "0.14"
tempfile = "3.15"  # For atomic writes
```

Already have `typst-syntax` as transitive dependency through `typst`, but
should add it explicitly since we're using it directly.

---

## Next Steps

1. ✅ Research write-back approach (this document)
2. Add `typst-syntax` to mindtape-eval dependencies
3. Implement `find_task_node()` function
4. Implement `toggle_task_checkbox()` function
5. Add `find_task_by_id()` to Store trait + SqliteStore
6. Implement `mindtape check` CLI command
7. Write tests
8. Update DESIGN.md and ROADMAP.md
