# Plan: Adopt clap + Split cli.rs

## Goal

Replace hand-rolled arg parsing with `clap` (derive) and restructure
`src/cli.rs` (1500 lines) into a `src/cli/` module with per-command files
and shared formatting.

## Target Structure

```
src/cli/
  mod.rs              -- re-exports, OutputFormat, shared args (GlobalOpts)
  commands/
    mod.rs            -- re-exports all command modules
    eval.rs           -- EvalArgs (clap), format_task, filter_and_sort, due_sort_key
    watch.rs          -- WatchArgs (clap)
    list.rs           -- ListArgs (clap)
    search.rs         -- SearchArgs (clap), format_search_results, format_search_csv
    agenda.rs         -- AgendaArgs (clap), format_agenda, format_agenda_csv
    status.rs         -- (uses GlobalOpts only, no extra args)
    files.rs          -- (uses GlobalOpts only, no extra args)
    deps.rs           -- DepsArgs (clap), format_deps, format_all_deps, format_deps_csv, format_all_deps_csv
    check.rs          -- CheckArgs (clap)
  format.rs           -- shared: format_task_view, format_file_view, format_stats,
                         csv_escape, format_tasks_csv, format_files_csv, format_stats_csv
```

## Step-by-step

### Step 1: Add clap dependency

In root `Cargo.toml`:
```toml
clap = { version = "4", features = ["derive"] }
```

### Step 2: Create `src/cli/mod.rs` with clap top-level enum

Define the top-level CLI using clap derive:

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "mindtape", about = "File-based task tracker using Typst")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    // Top-level eval mode (backwards compat: `mindtape file.typ`)
    // Handled via a default/fallback — see notes below.
}

#[derive(Subcommand)]
pub enum Command {
    Watch(WatchArgs),
    List(ListArgs),
    Search(SearchArgs),
    Agenda(AgendaArgs),
    Status(StatusArgs),
    Files(FilesArgs),
    Deps(DepsArgs),
    Check(CheckArgs),
}
```

**Backwards compat for `mindtape file.typ`:** Use clap's
`subcommand_negates_reqs` or an `external_subcommand` approach. The
cleanest way: make `Cli` have optional positional `file` + `--due` +
`-N` fields alongside the subcommand. When `command` is `None`, treat
it as eval mode. This preserves the `mindtape foo.typ --due -3` usage.

```rust
#[derive(Parser)]
#[command(name = "mindtape", about = "File-based task tracker using Typst")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Typst file to evaluate (shorthand for eval mode)
    pub file: Option<PathBuf>,

    /// Show only tasks with due dates, sorted by date
    #[arg(long)]
    pub due: bool,

    /// Limit output to N items
    #[arg(short = 'N', value_name = "N")]
    pub limit: Option<usize>,
}
```

Wait — clap doesn't support `-N` where N is a digit directly as a
dynamic short flag. **Workaround:** use `allow_hyphen_values` +
custom parsing for the `-N` shorthand. Alternatively, switch to
`-n <N>` or `--limit <N>` (breaking change). Or use a
`TrailingVarArg`-style approach.

**Recommended approach for `-N`:** Use `--limit` / `-n` as the clap
flag. Add a small pre-processing step that rewrites `-3` to `-n 3` in
the raw args before passing to clap. This keeps the UX unchanged and
clap happy. The preprocessor is ~10 lines.

### Step 3: Define shared args

`OutputFormat` as a `ValueEnum`:

```rust
#[derive(ValueEnum, Clone, Copy, Default, Debug, PartialEq)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Csv,
}
```

Shared query options (flattened into commands that need them):

```rust
#[derive(Args)]
pub struct QueryOpts {
    /// Path to SQLite database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Shorthand for --format json
    #[arg(long)]
    pub json: bool,
}
```

The `json` flag overrides `format` — resolve in a method:
```rust
impl QueryOpts {
    pub fn output_format(&self) -> OutputFormat {
        if self.json { OutputFormat::Json } else { self.format }
    }
}
```

### Step 4: Define per-command arg structs

Each in its own file under `src/cli/commands/`. Examples:

**`commands/list.rs`:**
```rust
#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    pub status: Option<StatusFilter>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long, value_name = "DATE")]
    pub due_before: Option<String>,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub folder: Option<PathBuf>,
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,
    #[command(flatten)]
    pub query: QueryOpts,
}
```

**`commands/search.rs`:**
```rust
#[derive(Args)]
pub struct SearchArgs {
    pub query: String,
    #[arg(short = 'n', long)]
    pub limit: Option<usize>,
    #[command(flatten)]
    pub opts: QueryOpts,
}
```

And so on for each command. `StatusArgs` and `FilesArgs` contain only
`#[command(flatten)] pub query: QueryOpts`.

### Step 5: Create `src/cli/format.rs` — shared formatters

Move these functions from the current `cli.rs`:
- `csv_escape`
- `format_task_view`
- `format_file_view`
- `format_stats`
- `format_tasks_csv`
- `format_files_csv`
- `format_stats_csv`

These are used by multiple commands (list, files, status, search, agenda).

### Step 6: Move per-command formatters into command files

Each command file gets its own formatting functions:

| File | Functions moved there |
|------|---------------------|
| `commands/eval.rs` | `format_task`, `format_due`, `due_sort_key`, `filter_and_sort` |
| `commands/search.rs` | `format_search_results`, `format_search_csv` |
| `commands/agenda.rs` | `format_agenda`, `format_agenda_csv` |
| `commands/deps.rs` | `format_deps`, `format_all_deps`, `format_deps_csv`, `format_all_deps_csv` |

### Step 7: Update `src/cli/mod.rs` re-exports

Re-export everything that `main.rs` needs:

```rust
pub mod commands;
pub mod format;

pub use commands::*;
// re-export key types
pub use self::{Cli, Command, OutputFormat, QueryOpts};
```

### Step 8: Update `main.rs`

Replace manual `parse_args` call with:

```rust
use clap::Parser;
use mindtape::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Watch(args)) => run_watch(&args),
        Some(Command::List(args)) => run_list(args),
        // ... etc
        None => {
            // Eval mode (backwards compat)
            let file = cli.file.unwrap_or_else(|| { /* print help, exit */ });
            run_eval(&file, cli.due, cli.limit);
        }
    }
}
```

Each `run_*` function changes minimally: just access `args.query.db`
instead of `args.db`, `args.query.output_format()` instead of
`args.format`, etc.

### Step 9: Add `-N` preprocessor

Before `Cli::parse()`, scan raw args and rewrite `-3` → `-n 3`:

```rust
fn preprocess_args() -> Vec<String> {
    let mut args: Vec<String> = std::env::args().collect();
    let mut i = 0;
    while i < args.len() {
        if let Some(rest) = args[i].strip_prefix('-') {
            if rest.parse::<usize>().is_ok() {
                let n = rest.to_string();
                args[i] = "-n".to_string();
                args.insert(i + 1, n);
                i += 1; // skip the inserted value
            }
        }
        i += 1;
    }
    args
}

fn main() {
    let args = preprocess_args();
    let cli = Cli::parse_from(args);
    // ...
}
```

### Step 10: Move tests

- **Parsing tests** (50+ tests in current cli.rs): Most become obsolete
  since clap handles parsing. Keep a handful of integration-style tests
  that verify `Cli::try_parse_from(...)` produces the right command
  variant. Put these in `src/cli/mod.rs` tests or a small
  `tests/cli_parse.rs`.

- **Formatting tests** stay with their respective files — each
  `format.rs` / `commands/*.rs` keeps its `#[cfg(test)]` module.

- **`-N` preprocessor test**: small unit test in `main.rs` or
  `cli/mod.rs`.

### Step 11: Update `src/lib.rs`

No change needed — `pub mod cli;` works whether cli is a file or directory.

### Step 12: Run `just check`, fix warnings

### Step 13: Update docs

- `CLAUDE.md`: update workspace layout section (cli.rs → cli/ module),
  mention clap, remove "Manual arg parsing (not clap)" from tech stack
- `docs/DESIGN.md`: if it mentions arg parsing approach, update
- `MEMORY.md`: update current status

## Notes

- **Binary size**: clap adds ~300KB. Acceptable for a CLI tool.
- **Breaking change risk**: `-N` shorthand is preserved via preprocessor.
  `--json` shorthand preserved via the `json` bool in `QueryOpts`.
  All existing CLI usage should work unchanged.
- **`parse_format`**: replaced by clap's `ValueEnum` derive on `OutputFormat`.
- **USAGE const**: replaced by clap's auto-generated `--help`.
- **Test count will drop**: many parse tests become unnecessary since
  clap is well-tested. Replace with a few smoke tests. Formatting tests
  stay.

## Rough line counts (estimated)

| File | Before | After |
|------|--------|-------|
| `src/cli.rs` | 1500 | (removed) |
| `src/cli/mod.rs` | — | ~60 |
| `src/cli/format.rs` | — | ~200 (shared formatters + tests) |
| `src/cli/commands/mod.rs` | — | ~15 |
| `src/cli/commands/eval.rs` | — | ~100 |
| `src/cli/commands/list.rs` | — | ~40 |
| `src/cli/commands/search.rs` | — | ~100 |
| `src/cli/commands/agenda.rs` | — | ~120 |
| `src/cli/commands/deps.rs` | — | ~130 |
| `src/cli/commands/check.rs` | — | ~20 |
| `src/cli/commands/watch.rs` | — | ~20 |
| `src/cli/commands/status.rs` | — | ~10 |
| `src/cli/commands/files.rs` | — | ~10 |
| `src/main.rs` | 434 | ~350 |
| **Total** | ~1930 | ~1175 |

Net reduction of ~750 lines, spread across focused files.
