# mindtape-eval

Typst evaluation and task extraction. This is the foundation crate with zero
knowledge of storage or CLI.

## Responsibility

- Evaluate `.typ` files via `typst_eval::eval()` (eval-only, no layout/render)
- Extract tasks from the content tree (checklist items with `[ ]`/`[x]` markers)
- Extract metadata: `#due()`, `#tag()`, `#id()` (via `MetadataElem`)
- Extract `#let` bindings and file title from module scope
- Provide `MindTapeWorld` (`typst::World` implementation)
- Write-back: safely modify `.typ` files (toggle, set due/tags)

## Source Layout

```
src/
  lib.rs           -- re-exports public API
  eval.rs          -- EvalError, Task, EvalResult, eval functions, content traversal
  world.rs         -- MindTapeWorld (World trait impl), project root detection
  write.rs         -- AST-based write-back (toggle, due, tags)
  write_tests.rs   -- separated tests for write.rs
```

## Key Types

- `Task` — extracted task (title, done, due, tags, id, position, milestone)
- `EvalResult` — tasks + title + bindings + dependencies
- `EvalError` — enum (File, Eval, World, NotMindtape)
- `WriteError` — enum (File, TaskNotFound, InvalidTask, InvalidDate)
- `MindTapeWorld` — World impl with source caching and dependency tracking

## Key Functions

**Evaluation** (`eval.rs`):
- `eval_file(world)` — evaluate and return tasks only
- `eval_file_full(world)` — evaluate and return tasks + title + bindings
- `eval_file_full_with_deps(world)` — full eval with dependency tracking
- `collect_tasks(content, tasks)` — recursively traverse content tree
- `extract_task(item)` — parse a single `ListItem` as a task
- `extract_bindings(scope)` — extract `#let` bindings as (name, type, json)
- `extract_file_title(content)` — first heading as file title
- `has_mindtape_import(text)` — check if source imports mindtape package
- `format_date(dt)` — format Typst `Datetime` as YYYY-MM-DD

**Write-back** (`write.rs`):
- `load_source(path)` — read and parse `.typ` file
- `toggle_task_checkbox(source, task_id)` — toggle `[ ]` ↔ `[x]`
- `set_task_due(source, task_id, date_str)` — add/update due date
- `remove_task_due(source, task_id)` — remove due date
- `add_task_tag(source, task_id, tag)` — add tag
- `remove_task_tag(source, task_id, tag)` — remove tag

**World** (`world.rs`):
- `MindTapeWorld::new(file_path)` — construct World for a file
- `MindTapeWorld::get_dependencies()` — list imported file paths
- `find_project_root(start_dir)` — walk up for Cargo.toml/lib/.git

## Technical Notes

- `MetadataElem` is in `typst_library::introspection`, NOT `model`
- No `ListElem` wrapper — `ListItem` nodes are flat in sequence
- Checkbox `[ ]`/`[x]` are NOT native Typst — parsed from `plain_text()`
- `@mindtape` package resolution routes to `{root}/lib/` directory
- `comemo = "0.5"` must match typst 0.14's version
- `HeadingElem.depth` requires `StyleChain::default()` to access

## Dependencies

- `typst`, `typst-eval`, `typst-library`, `typst-syntax`, `comemo`
- `chrono`, `dirs`, `thiserror`, `log`
- No storage, CLI, or config dependencies
