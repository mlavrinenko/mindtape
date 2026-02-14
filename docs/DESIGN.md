# MindTape Design

## Overview

MindTape is a file-based task tracker that uses Typst files as the source of truth.
It watches configured folders, evaluates `.typ` files using the Typst compiler,
extracts structured task data, and indexes everything into a database for querying.

## Why Typst

Markdown task lists are flat text — no imports, no types, no compile-time checks.
Typst gives us:

- **Imports**: share statuses, tags, and schemas across files via `#import`
- **Typed values**: dates are `datetime`, not strings; statuses are constrained values
- **Compile-time checking**: broken imports or type mismatches are caught by the Typst evaluator
- **Extensibility**: users define their own `#let` bindings freely

## Data Model

### Task File

A `.typ` file represents a **milestone** (or just a container). It can contain:

- **Heading** (`= Title`): the milestone/group name
- **Checklist items** (`- [ ]` / `- [x]`): individual tasks
- **Inline functions**: `#due(...)`, `#tag(...)` attached to list items as task properties
- **Top-level `#let` bindings**: exported metadata (searchable/filterable via index)

Example:

```typ
#import "@mind-tape": due, tag

= Some Milestone

- [ ] small task 1 #due(datetime(year: 2026, month: 2, day: 5)) #tag("fun")
- [x] small task 2

#let note = "hello world"
```

### What Gets Indexed

From evaluation of a `.typ` file, we extract:

1. **Content tree** (headings, list items with checked/unchecked state, inline function calls)
2. **Scope bindings** (all `#let` values exported by the module: strings, dates, dicts, etc.)
3. **File metadata** (path, last modified, watched folder it belongs to)

### Core Entities (Database)

```
WatchedFolder
  id, path, created_at

TaskFile
  id, watched_folder_id, relative_path, title (from first heading), updated_at, eval_hash

Task
  id, task_file_id, title (list item text), is_done, position (order in file)

TaskProperty
  id, task_id, kind (due | tag | custom), key, value

FileBinding
  id, task_file_id, name, value_type, value_json
```

This is a starting point. The schema will evolve as we implement.

## Architecture

```
+------------------+     +----------------+     +-----------+
| Folder Watcher   |---->| Typst Evaluator|---->| Indexer    |
| (notify)         |     | (typst-eval)   |     | (database)|
+------------------+     +----------------+     +-----------+
                                                      |
                                                      v
                                                 +---------+
                                                 | Query   |
                                                 | Layer   |
                                                 +---------+
                                                      |
                                                      v
                                                 +---------+
                                                 | CLI     |
                                                 | (clap)  |
                                                 +---------+
```

### Crate / Module Boundaries

Single binary crate to start, with clear module separation:

```
src/
  main.rs              -- CLI entry point (clap)
  lib.rs               -- re-exports for potential library use

  config/
    mod.rs             -- config loading, watched folders, ignore rules

  watcher/
    mod.rs             -- folder watching via `notify`, debouncing, change events

  evaluator/
    mod.rs             -- Typst World implementation, file evaluation
    world.rs           -- World trait impl (source loading, fonts, packages)
    extract.rs         -- extract tasks/bindings from Module/Content

  store/
    mod.rs             -- database trait (anti-corruption layer)
    sqlite.rs          -- SQLite implementation
    models.rs          -- domain types (TaskFile, Task, TaskProperty, FileBinding)

  query/
    mod.rs             -- query builder / filter types

  commands/
    mod.rs             -- CLI command handlers
    watch.rs           -- `mind-tape watch` (start watcher + indexer)
    list.rs            -- `mind-tape list` (query tasks)
    status.rs          -- `mind-tape status` (show index stats)
```

### Anti-Corruption Layer (Store Trait)

```rust
// Conceptual — not final API
trait Store {
    fn upsert_task_file(&self, file: &TaskFile) -> Result<()>;
    fn upsert_tasks(&self, file_id: Id, tasks: &[Task]) -> Result<()>;
    fn upsert_bindings(&self, file_id: Id, bindings: &[FileBinding]) -> Result<()>;
    fn remove_task_file(&self, path: &Path) -> Result<()>;
    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>>;
    // ...
}
```

SQLite first. DuckDB or others can implement the same trait later.

## Typst Evaluation Strategy

We use `typst-eval::eval()` to stop at the evaluation stage — no layout, no PDF rendering.
This gives us a `Module` with:

- `module.scope()` — all `#let` bindings as typed `Value`s
- `module.content()` — the full content tree to traverse

Key crates: `typst`, `typst-eval`, `typst-library`, `typst-syntax`, `typst-kit`.

### World Implementation

We need a custom `World` impl that:

- Resolves the main file and relative imports within watched folders
- Provides minimal font book (we don't render, but Typst requires it)
- Handles `@mind-tape` package imports (our custom functions: `due`, `tag`, etc.)

### The `@mind-tape` Package

We provide a Typst package (or local import convention) that defines:

- `#let due(date)` — attach a due date to a task
- `#let tag(name)` — attach a tag to a task
- Possibly more in the future (priority, assignee, etc.)

These are Typst functions that produce **content elements** we can identify during
content tree traversal after evaluation.

**Open question**: Should this be a real Typst package published to `@preview`,
a local package, or resolved by MindTape's custom World? For MVP, a local approach
(MindTape's World resolves `@mind-tape` to built-in definitions) is simplest.

## Configuration

```toml
# ~/.config/mind-tape/config.toml (or mind-tape.toml in project root)

[database]
path = "~/.local/share/mind-tape/index.db"

[[watch]]
path = "~/projects/myproject"
recursive = true

[[watch]]
path = "~/notes/tasks"
recursive = true
```

Ignore files: `.mindtapeignore` in any watched folder, gitignore-style patterns.

## Open Questions

- How to handle Typst package imports (`@preview/...`) — do we support them?
- How to resolve cross-folder imports (file in folder A imports from folder B)?
- Exact content tree traversal strategy for extracting task properties from inline
  function calls (need to prototype with real Typst evaluation to understand the
  Content structure for `- [ ] text #due(...) #tag(...)`)
- Should the `@mind-tape` functions produce special labeled content (using Typst's
  `metadata()` + `label`) to make extraction easier via `typst query`?
