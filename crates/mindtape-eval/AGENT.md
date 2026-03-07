# mindtape-eval

Typst evaluation and task extraction. Zero knowledge of storage or CLI.

## Eval API (typst 0.14)

```rust
typst_eval::eval(routines, world, traced, sink, route, source) -> SourceResult<Module>
```

`typst-kit` is NOT needed for eval-only.

## World Implementation

`MindTapeWorld` implements `typst::World`:

- `library()`: `Library::default()` wrapped in `LazyHash`
- `book()`: Empty `FontBook::new()` (no fonts needed for eval-only)
- `main()`: `FileId` for the input file
- `source(id)`: Read from disk, cache in `HashMap`
- `file(id)`: Read raw bytes from disk
- `today()`: Current date via `chrono_free_today()`

Project root: auto-detected by walking up from file directory looking for
`Cargo.toml`, `lib/`, or `.git` markers.

## Content Tree Structure

There is NO `ListElem` wrapper. `ListItem` nodes appear directly in a flat
`SequenceElem`. Checkbox `[ ]`/`[x]` is parsed from `plain_text()`, not AST.

Given `- [ ] Task text #due(2026, 3, 1)`, the actual content tree is:

```
ListItem { body: SequenceElem [
  Text([), SpaceElem, Text(]),   // "[ ]" split into parts
  SpaceElem, Text(Task text), SpaceElem,
  MetadataElem { value: ["due", Date(2026-05-01)] },
]}
```

## Task Property Functions (lib/prelude.typ)

`due()`, `start()`, `id()`, `tag()`, `rank()` produce `MetadataElem` via
`metadata()`. Value is a Typst `Array`: index 0 = kind string, index 1 = value.
`high`, `medium`, `low` are `let` bindings (used as `#high`, not `#high()`).

## Write-Back

AST-based text replacement via `typst_syntax::parse()`:
1. Parse file → walk AST to find `ListItem` by task ID
2. Extract text range, modify checkbox/properties in substring
3. Atomic write via `tempfile` + rename
4. Conflict detection: SHA-256 hash check before write.

## Source Layout

```
src/
  lib.rs           -- re-exports public API
  eval.rs          -- EvalError, Task, EvalResult, eval functions, content traversal
  world.rs         -- MindTapeWorld (World trait impl), project root detection
  write.rs         -- AST-based write-back (toggle, due, tags)
  write_tests.rs   -- separated tests for write.rs
```

## Technical Notes

- `MetadataElem` is in `typst_library::introspection`, NOT `model`
- No `ListElem` wrapper — `ListItem` nodes are flat in sequence
- Checkbox `[ ]`/`[x]` are NOT native Typst — parsed from `plain_text()`
- `@local/mindtape` package resolution routes to `~/.local/share/typst/packages/local/mindtape/VERSION/`
- `comemo = "0.5"` must match typst 0.14's version
- `HeadingElem.depth` requires `StyleChain::default()` to access

## Dependencies

- `typst`, `typst-eval`, `typst-library`, `typst-syntax`, `comemo`
- `chrono`, `dirs`, `thiserror`, `log`
- No storage, CLI, or config dependencies
