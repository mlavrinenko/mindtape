# mindtape-eval

Typst evaluation and task extraction. Zero knowledge of storage or CLI.

For architecture details see [docs/design/typst-eval.md](../../docs/design/typst-eval.md)
and [docs/design/writeback.md](../../docs/design/writeback.md).

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
