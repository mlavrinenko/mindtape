# Typst Evaluation Strategy

## Overview

MindTape uses `typst_eval::eval()` to stop at the evaluation stage — no layout, no
PDF rendering. This gives us a `Module` with:

- `module.scope()` — all `#let` bindings as typed `Value`s
- `module.content()` — the full content tree to traverse

## Eval API

See `crates/mindtape-eval/src/eval.rs` for the evaluation implementation.

The eval API signature (typst 0.14):

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

## Key Dependencies

```toml
typst = "0.14"
typst-eval = "0.14"
typst-library = "0.14"
typst-syntax = "0.14"
comemo = "0.5"   # must match typst 0.14's comemo version
```

`typst-kit` is NOT needed for eval-only.

## World Implementation

See `crates/mindtape-eval/src/world.rs` for the complete implementation.

`MindTapeWorld` implements `typst::World`:

- `library()`: `Library::default()` wrapped in `LazyHash`
- `book()`: Empty `FontBook::new()` (no fonts needed for eval-only)
- `font()`: Returns `None` always
- `main()`: `FileId` for the input file
- `source(id)`: Read from disk, cache in `HashMap`
- `file(id)`: Read raw bytes from disk
- `today()`: Current date via `chrono_free_today()` (no chrono dependency)

Project root is auto-detected by walking up from the file's directory looking
for `Cargo.toml`, `lib/`, or `.git` markers.

## Checkbox Convention (Not Native Typst)

Typst does NOT have built-in `- [ ]` / `- [x]` checklist syntax. In Typst:
- `- item` creates a `ListItem` with a `body: Content` field
- `[ ]` and `[x]` inside a list item are just text content
- There is no `checked` state on `ListItem`

We parse the checkbox pattern from `ListItem.body.plain_text()`:
- `[ ] ` prefix = unchecked task
- `[x] ` or `[X] ` prefix = checked task

## Content Tree Structure

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

## Task Property Functions

See `lib/prelude.typ` for the implementation.

Functions like `due()`, `id()`, `tag()` all use `metadata()`:

```typ
#let due(date) = metadata(("due", date))
#let id(uuid) = metadata(("id", uuid))
#let tag(name) = metadata(("tag", name))
```

`metadata()` produces a `MetadataElem` with a `value: Value` field.
The value is a Typst `Array`:
- Index 0: `Str` — the kind (`"due"`, `"id"`, `"tag"`)
- Index 1: `Datetime` or `Str` — the actual value

Imported via the `@local/mindtape` package namespace, resolved by the World:

```typ
#import "@local/mindtape:0.1.0": due, id, tag
```

## Extraction Algorithm

See `crates/mindtape-eval/src/eval.rs` for the implementation.

For each `ListItem` found via `content.traverse()`:
1. Get `body.plain_text()` — e.g. `"[ ] Task text  "`
2. Parse checkbox: `strip_prefix("[x] ")` or `strip_prefix("[ ] ")`
3. `trim()` remaining text to remove trailing whitespace
4. Traverse body with `body.traverse()` for `MetadataElem` nodes
5. For each MetadataElem, check if value is `Array` with `slice[0] == "due"`
6. Extract `slice[1]` as `Value::Datetime`

## Known Gotchas

- `comemo::Track::track()` is on `dyn World`, not concrete types. When
  `eval_file` takes `&dyn World`, `world.track()` works directly.
- `comemo = "0.5"` must match typst 0.14's pinned version exactly.
- `MetadataElem` is in `typst_library::introspection`, not `model`
- No `ListElem` wrapper exists in content tree
- Checkbox patterns must be parsed from plain text, not AST
