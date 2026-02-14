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
#import "lib/prelude.typ": due, id

= Some Milestone

- [ ] small task 1 #due(datetime(year: 2026, month: 2, day: 5))
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
                                                 +---------+
```

### Current Module Layout

```
src/
  lib.rs        -- pub mod declarations (cli, eval, world)
  main.rs       -- thin CLI entry point (~20 lines)
  cli.rs        -- arg parsing, task filtering/sorting, output formatting
  eval.rs       -- Typst evaluation, content tree traversal, task extraction
  world.rs      -- World trait impl, project root detection, date utility

tests/
  eval_integration.rs  -- end-to-end evaluation tests with temp .typ files

lib/
  prelude.typ   -- due(), id() functions using metadata()

itest/
  basic.sh      -- shell integration test
  res/piano.typ -- test fixture
```

### Future Module Layout (M1.3+)

```
src/
  config/       -- config loading, watched folders, ignore rules
  watcher/      -- folder watching via `notify`, debouncing
  store/        -- database trait + SQLite implementation
  query/        -- query builder / filter types
  commands/     -- CLI command handlers (watch, list, status)
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
}
```

SQLite first. DuckDB or others can implement the same trait later.

## Typst Evaluation Details

### Strategy

We use `typst_eval::eval()` to stop at the evaluation stage — no layout, no
PDF rendering. This gives us a `Module` with:

- `module.scope()` — all `#let` bindings as typed `Value`s
- `module.content()` — the full content tree to traverse

### Eval API (v0.14)

```rust
typst_eval::eval(
    routines: &Routines,         // typst::ROUTINES static
    world: Tracked<dyn World>,   // world.track()
    traced: Tracked<Traced>,     // Traced::default().track()
    sink: TrackedMut<Sink>,      // sink.track_mut()
    route: Tracked<Route>,       // Route::default().track()
    source: &Source,
) -> SourceResult<Module>
```

### Key Dependencies

```toml
typst = "0.14"
typst-eval = "0.14"
typst-library = "0.14"
typst-syntax = "0.14"
comemo = "0.5"   # must match typst 0.14's comemo version
```

`typst-kit` is NOT needed for eval-only. `clap` is not used — manual arg
parsing supports the `-N` shorthand.

### World Implementation

`MindTapeWorld` in `src/world.rs` implements `typst::World`:

- `library()`: `Library::default()` wrapped in `LazyHash`
- `book()`: Empty `FontBook::new()` (no fonts needed for eval-only)
- `font()`: Returns `None` always
- `main()`: `FileId` for the input file
- `source(id)`: Read from disk, cache in `HashMap`
- `file(id)`: Read raw bytes from disk
- `today()`: Current date via `chrono_free_today()` (no chrono dependency)

Project root is auto-detected by walking up from the file's directory looking
for `Cargo.toml`, `lib/`, or `.git` markers.

### Checkbox Convention (Not Native Typst)

Typst does NOT have built-in `- [ ]` / `- [x]` checklist syntax. In Typst:
- `- item` creates a `ListItem` with a `body: Content` field
- `[ ]` and `[x]` inside a list item are just text content
- There is no `checked` state on `ListItem`

We parse the checkbox pattern from `ListItem.body.plain_text()`:
- `[ ] ` prefix = unchecked task
- `[x] ` or `[X] ` prefix = checked task

### Content Tree Structure

There is NO `ListElem` wrapper in the content tree. `ListItem` nodes
appear directly in a flat `SequenceElem`.

Given `- [ ] Task text #due(datetime(...))`, the actual content tree is:

```
ListItem { body: SequenceElem [
  Text([), SpaceElem, Text(]),   // "[ ]" split into parts
  SpaceElem,
  Text(Task text),
  SpaceElem,
  MetadataElem { value: ["due", Date(2026-05-01)] },
  SpaceElem,                      // trailing whitespace
]}
```

Key observations:
- `plain_text()` concatenates all text: `"[ ] Task text  "` (with trailing spaces)
- Must `trim()` the title after stripping the checkbox prefix
- `MetadataElem` is in `typst_library::introspection`, NOT `typst_library::model`
- Traverse for `ListItem` directly, NOT `ListElem`

### Task Property Functions (`lib/prelude.typ`)

```typ
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
```

`metadata()` produces a `MetadataElem` with a `value: Value` field.
The value is a Typst `Array`:
- Index 0: `Str` — the kind (`"due"`, `"id"`)
- Index 1: `Datetime` or `Str` — the actual value

Currently imported via relative path (`#import "lib/prelude.typ": due`).
Future: resolve via `@mind-tape` package namespace in the World.

### Extraction Algorithm

For each `ListItem` found via `content.traverse()`:
1. Get `body.plain_text()` — e.g. `"[ ] Task text  "`
2. Parse checkbox: `strip_prefix("[x] ")` or `strip_prefix("[ ] ")`
3. `trim()` remaining text to remove trailing whitespace
4. Traverse body with `body.traverse()` for `MetadataElem` nodes
5. For each MetadataElem, check if value is `Array` with `slice[0] == "due"`
6. Extract `slice[1]` as `Value::Datetime`

### Known Gotchas

- `comemo::Track::track()` is on `dyn World`, not concrete types. When
  `eval_file` takes `&dyn World`, `world.track()` works directly.
- `comemo = "0.5"` must match typst 0.14's pinned version exactly.

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
- Should `@mind-tape` be a real Typst package or resolved by the custom World?
