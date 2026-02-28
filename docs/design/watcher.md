# Folder Watcher

## Overview

MindTape watches configured directories for `.typ` file changes, automatically
indexing new files and re-indexing modified files. Uses `notify` for filesystem
events and `ignore` for `.mindtapeignore` support.

## File Watching

- `notify` 7.0 for filesystem events
- `notify-debouncer-mini` 0.5 with 300ms debounce window
- Debouncer collapses event kinds to `Any`/`AnyContinuous`
- Use `path.exists()` to distinguish modify vs delete events

## Watcher Architecture

See `src/watcher.rs` for implementation.

```rust
pub struct Watcher {
    store: SqliteStore,
    entries: Vec<ResolvedEntry>,  // resolved watch paths from config
}
```

### Initial Scan

```rust
fn initial_scan(&mut self) -> Result<()>
```

1. Walk directories via `ignore` crate
2. Filter for `.typ` files
3. Index each file via `index_file()` (with hash-based skip)
4. Print summary (files indexed, skipped, errors)

### Event Handling

```rust
fn handle_event(&mut self, path: &Path) -> Result<()>
```

- If `path.exists()`: re-index the file
- Otherwise: remove from store

### Event Loop

```rust
pub fn run(self) -> Result<()>
```

Blocking loop on `mpsc` channel from notify debouncer. Calls `handle_event()`
for each debounced filesystem event.

## Configuration

See `src/config.rs` for TOML parsing and tilde expansion.

### Config Format

```toml
# ~/.config/mindtape/config.toml

[database]
path = "~/.local/share/mindtape/index.db"

[[watch]]
path = "~/projects/myproject"
recursive = true

[[watch]]
path = "~/notes/tasks"
recursive = true
```

### Config Auto-Discovery

Checks `~/.config/mindtape/config.toml`.
Falls back to watching the current directory if no config is found.

### Tilde Expansion

`~` and `~user` patterns are expanded via `shellexpand::tilde()`.
All paths are converted to absolute paths before watching.

## Ignore Patterns

Uses the `ignore` crate for `.mindtapeignore` support:

- Gitignore-style pattern syntax
- `.mindtapeignore` in any watched folder
- Automatically ignores hidden files/directories (`.git`, etc.)
- Respects `.gitignore` files (can be disabled if needed)

## Borrow Pattern

The `Watcher` struct owns both the store and entry list. To avoid
simultaneous `&self` + `&mut self` borrows, methods collect data from
`&self.entries` into local variables first, then call `&mut self` methods.

Example:

```rust
fn initial_scan(&mut self) -> Result<()> {
    // Collect paths from &self.entries first
    let paths: Vec<_> = self.entries.iter()
        .map(|e| e.path.clone())
        .collect();
    
    // Now we can use &mut self without borrow conflict
    for path in paths {
        self.index_path(&path)?;
    }
}
```

## Config Hot-Reload

When started with a config file (`--config` or auto-discovered), the watcher
monitors the config file for changes and dynamically updates watched paths.

- Watches the config file's **parent directory** (not the file itself) so
  atomic-save editors (vim/emacs: write-tmp + rename) are detected correctly
- On config change: reloads TOML, diffs watch entries by canonical path,
  calls `unwatch()`/`watch()` on the notify watcher, scans new directories
- Parse errors are **non-fatal** — a warning is logged and old config is kept
- Database path changes are logged but require a manual restart

## Error Handling

Watcher continues on individual file errors (e.g., parse failures) but
propagates fatal errors (e.g., database connection lost, config not found).

File-level errors are logged and counted, not fatal to the watch loop.
