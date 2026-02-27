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

A `.typ` file can contain:

- **Headings** (`= Title`): milestone/group names
- **Checklist items** (`- [ ]` / `- [x]`): individual tasks
- **Inline functions**: `#due()`, `#tag()`, `#id()` as task properties
- **Top-level bindings**: `#let` exports for file-level metadata

Database entities: `WatchedFolder`, `TaskFile`, `Task`, `TaskProperty`, `FileBinding`, `FileReferences`.

See `docs/design/store.md` for full schema details.

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

## Design Documents

- **[Typst Evaluation](design/typst-eval.md)** — eval strategy, content tree, extraction algorithm
- **[Store Architecture](design/store.md)** — Store trait, schema, indexing pipeline
- **[Folder Watcher](design/watcher.md)** — file watching, config, ignore patterns
- **[Write-Back](design/writeback.md)** — safe modification, conflict detection, task ID masking

## Configuration

```toml
# ~/.config/mindtape/config.toml or mindtape.toml in project root

[database]
path = "~/.local/share/mindtape/index.db"

[[watch]]
path = "~/projects/myproject"
recursive = true
```

See `docs/design/watcher.md` for configuration details.

## NixOS Module

The flake exposes `nixosModules.default` (and `.mindtape`) for declarative service
configuration. The module generates a TOML config in the Nix store and runs
`mindtape watch --config <path>` as a systemd service with sandboxing
(`ProtectHome=read-only`, `ProtectSystem=strict`). See `nix/module.nix`.

## Open Questions

- How to handle Typst package imports (`@preview/...`) — do we support them?
- How to resolve cross-folder imports (file in folder A imports from folder B)?
