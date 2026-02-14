# mindtape-eval

Typst evaluation and task extraction. This is the foundation crate with zero
knowledge of storage or CLI.

## Responsibility

- Evaluate `.typ` files via `typst_eval::eval()` (eval-only, no layout/render)
- Extract tasks from the content tree (checklist items with `[ ]`/`[x]` markers)
- Extract metadata: `#due()`, `#tag()`, `#id()` (via `MetadataElem`)
- Extract `#let` bindings from module scope
- Provide `MindTapeWorld` (`typst::World` implementation)

## Key Types

- `Task` — a task extracted from a checklist item
- `EvalResult` — tasks + title + bindings from evaluating a file
- `MindTapeWorld` — World impl, resolves files relative to project root

## Key Functions

- `eval_file(world)` — evaluate and return tasks
- `eval_file_full(world)` — evaluate and return tasks + title + bindings
- `extract_task(item)` — parse a single `ListItem` as a task
- `collect_tasks(content, tasks)` — recursively traverse content tree
- `find_project_root(dir)` — walk up looking for project markers

## Technical Notes

- `MetadataElem` is in `typst_library::introspection`, NOT `model`
- No `ListElem` wrapper — `ListItem` nodes are flat in sequence
- Checkbox `[ ]`/`[x]` are NOT native Typst — parsed from `plain_text()`
- `@mindtape` package resolution routes to `{root}/lib/` directory
- `comemo = "0.5"` must match typst 0.14's version

## Dependencies

- `typst`, `typst-eval`, `typst-library`, `typst-syntax`, `comemo`
- No storage, CLI, or config dependencies
