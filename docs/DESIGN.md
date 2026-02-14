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

### Workspace Layout

```
crates/
  mindtape-eval/        -- Typst evaluation + task extraction
    src/eval.rs          -- EvalError, Task, EvalResult, eval_file(), content traversal
    src/world.rs         -- MindTapeWorld (World trait impl), project root detection

  mindtape-store/       -- Store trait + SQLite backend
    src/store/mod.rs     -- domain types, Store trait, StoreError
    src/store/sqlite.rs  -- SqliteStore impl, schema, migrations
    src/store/indexer.rs -- hash_file(), to_store_records(), index_file()

src/                    -- root crate: CLI binary
  cli.rs                -- Command enum, arg parsing, formatting, --json support
  config.rs             -- TOML config loading, WatchEntry, tilde expansion
  watcher.rs            -- Watcher struct, initial_scan, handle_event, run
  main.rs               -- thin CLI entry point, command routing

tests/                  -- integration tests
  eval_integration.rs, store_integration.rs, query_integration.rs, watcher_integration.rs

lib/
  prelude.typ           -- due(), id(), tag() functions using metadata()
  typst.toml            -- package manifest for @mindtape/mindtape:0.1.0

itest/
  basic.sh              -- shell integration test (8 tests)
  res/piano.typ         -- test fixture
```

### Anti-Corruption Layer (Store Trait)

```rust
pub trait Store {
    fn upsert_task_file(&mut self, file: &TaskFile) -> Result<i64, StoreError>;
    fn upsert_tasks(&mut self, file_id: i64, tasks: &[TaskRecord], props: &[Vec<TaskProperty>]) -> Result<(), StoreError>;
    fn upsert_bindings(&mut self, file_id: i64, bindings: &[FileBinding]) -> Result<(), StoreError>;
    fn remove_task_file(&mut self, path: &Path) -> Result<(), StoreError>;
    fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskView>, StoreError>;
    fn get_file_hash(&self, path: &Path) -> Result<Option<String>, StoreError>;
}
```

SQLite first (`SqliteStore`). DuckDB or others can implement the same trait later.

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

`MindTapeWorld` in `crates/mindtape-eval/src/world.rs` implements `typst::World`:

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
#let tag(name) = metadata(("tag", name))
```

`metadata()` produces a `MetadataElem` with a `value: Value` field.
The value is a Typst `Array`:
- Index 0: `Str` — the kind (`"due"`, `"id"`, `"tag"`)
- Index 1: `Datetime` or `Str` — the actual value

Imported via the `@mindtape` package namespace, resolved by the World:

```typ
#import "@mindtape/mindtape:0.1.0": due, id, tag
```

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
# ~/.config/mindtape/config.toml (or mindtape.toml or ./.mindtape/config.toml in project root)

[database]
path = "~/.local/share/mindtape/index.db"

[[watch]]
path = "~/projects/myproject"
recursive = true

[[watch]]
path = "~/notes/tasks"
recursive = true
```

Ignore files: `.mindtapeignore` in any watched folder, gitignore-style patterns.

## Folder Watcher (M1.4)

### File Watching

- `notify` 7.0 for filesystem events + `notify-debouncer-mini` 0.5 (300ms debounce)
- Debouncer collapses event kinds to `Any`/`AnyContinuous` — use `path.exists()`
  to distinguish modify (re-index) vs delete (remove from store)
- `ignore` crate for `.mindtapeignore` support (gitignore-style patterns)

### Watcher Architecture

```
Watcher {
    store: SqliteStore,
    entries: Vec<ResolvedEntry>,  // resolved watch paths from config
}
```

- `initial_scan()`: walk directories via `ignore` crate, collect `.typ` files,
  index each via `index_file()` (with SHA-256 hash-based skip)
- `handle_event(path)`: re-index on modify, remove from store on delete
- `run()`: blocking notify event loop on `mpsc` channel

### Config Auto-Discovery

Searches in order:
1. `mindtape.toml` in the current working directory
2. `~/.config/mindtape/config.toml`

### Borrow Pattern

The `Watcher` struct owns both the store and entry list. To avoid
simultaneous `&self` + `&mut self` borrows, methods collect data from
`&self.entries` into local variables first, then call `&mut self` methods.

## Open Questions

- How to handle Typst package imports (`@preview/...`) — do we support them?
- How to resolve cross-folder imports (file in folder A imports from folder B)?
