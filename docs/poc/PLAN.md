# PoC Implementation Plan

## Goal
Make `itest/basic.sh` pass. Given a `.typ` file, evaluate it with Typst,
extract tasks with metadata, filter/sort/limit, and print formatted output.

## Expected Test Behavior
```bash
mindtape res/piano.typ --due -2
```
Output:
```
- (due 2026-04-01) Learn 5 Hanon exercises
- (due 2026-05-01) Finish learning Lilium
```

## Steps

### 1. Create lib/prelude.typ
Location: `lib/prelude.typ` (project root)
Defines: `due(date)`, `id(uuid)` as functions producing `metadata()` content.

### 2. Fix piano.typ import
Change `#import "../../lib.typ": due, id`
to `#import "../../lib/prelude.typ": due, id`

### 3. Set up Cargo project
- `Cargo.toml` at project root
- Binary name: `mindtape`
- Dependencies: typst, typst-eval, typst-library, typst-syntax, typst-kit, comemo, clap

### 4. Implement src/main.rs — CLI
Using clap derive:
- Positional arg: file path
- `--due` flag: filter tasks with due dates, sort by due
- `-N` / `--limit N`: limit results

### 5. Implement src/world.rs — MindTapeWorld
Custom World trait implementation:
- Resolves main file from CLI path
- Resolves relative imports (for `../../lib/prelude.typ`)
- No package support needed (piano.typ uses relative import)
- Empty font book (eval-only)
- File caching in HashMap

### 6. Implement src/eval.rs — Evaluation + Extraction
- Call `typst_eval::eval()` with our World
- Traverse content tree
- For each ListItem: parse checkbox state from body text
- For each MetadataElem in ListItem body: extract due/id
- Return Vec<Task>

### 7. Implement src/task.rs — Task model
```rust
struct Task {
    title: String,
    done: bool,
    due: Option<NaiveDate>, // or typst Datetime
}
```

### 8. Implement filtering + sorting + output
- Filter: exclude done tasks (default), filter --due (has due date)
- Sort: if --due, sort by due date ascending
- Limit: take first N
- Format: `- (due YYYY-MM-DD) Title`

### 9. Update flake.nix
Add necessary native dependencies for typst crates:
- openssl (for typst-kit downloads, though we may not need it)
- pkg-config

### 10. Build and test
- `cargo build`
- Run `itest/basic.sh` from `itest/` directory

## File Layout
```
mind-tape/
  Cargo.toml
  src/
    main.rs      — CLI entry, orchestration
    world.rs     — MindTapeWorld (World trait impl)
    eval.rs      — evaluate .typ, extract tasks
    task.rs      — Task struct, filtering, sorting, formatting
  lib/
    prelude.typ  — due(), id() functions
  itest/
    basic.sh
    res/piano.typ
```
