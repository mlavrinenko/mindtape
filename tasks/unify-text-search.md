# Unify text search to use FTS5 consistently

## Problem

The `list` command has three text-matching filters that work differently:

| Flag | Field | Mechanism | Searches |
|------|-------|-----------|----------|
| `--search` | `TaskFilter::search` | FTS5 MATCH | title + milestone |
| `--title` | `TaskFilter::title_contains` | SQL LIKE | title only |
| `--milestone` | `TaskFilter::milestone` | SQL LIKE | milestone only |

FTS5 supports ranking, prefix matching, phrase queries, and boolean operators.
LIKE is just substring matching. This inconsistency is confusing:
- `--search "buy groceries"` does FTS5 matching (tokenized, ranked)
- `--title "buy groceries"` does `LIKE '%buy groceries%'` (exact substring)

## Options

### Option A: Replace --title and --milestone with FTS5 (recommended)

Remove `--title` and `--milestone` flags. The `--search` flag already
searches both fields. Users who want to scope to one field can use
FTS5 column filters: `--search "title:groceries"` or
`--search "milestone:shopping"`.

**Pros**: Simpler API, one search mechanism, consistent behavior.
**Cons**: Loses exact substring matching (FTS5 tokenizes differently).

### Option B: Keep all three, upgrade --title and --milestone to FTS5

Change `title_contains` and `milestone` filters to also use FTS5,
but scoped to their respective columns.

**Pros**: Keeps familiar flags, consistent mechanism underneath.
**Cons**: More code, FTS5 column filters are unusual in CLIs.

### Option C: Keep as-is but document the difference

Just add help text explaining that `--search` uses full-text search
while `--title`/`--milestone` use substring matching.

**Pros**: No code changes, backward compatible.
**Cons**: Stays inconsistent.

## What to do (Option A)

### 1. Remove fields from TaskFilter

In `crates/mindtape-store/src/store/mod.rs`, remove:
```rust
pub milestone: Option<String>,
pub title_contains: Option<String>,
```

### 2. Update SQL in build_task_query()

In `crates/mindtape-store/src/store/sqlite.rs`, remove the LIKE
clauses for milestone and title_contains. FTS5 already covers these
via `tasks_fts MATCH ?`.

### 3. Update ListArgs

In `src/cli/commands/list.rs`, remove `--milestone` and `--title` flags.

### 4. Update help text

Add to `--search` help: "Searches task titles and milestones. Supports
FTS5 syntax: prefix*, \"phrases\", column:term."

### 5. Update tests

Remove/update tests that use `milestone` and `title_contains` filters.
Add tests for FTS5 column-scoped queries.

### Verify

- `just check` passes
- `just itest` passes

## Files to modify

1. `crates/mindtape-store/src/store/mod.rs` — remove TaskFilter fields
2. `crates/mindtape-store/src/store/sqlite.rs` — remove LIKE clauses
3. `src/cli/commands/list.rs` — remove CLI flags
4. `tests/query_integration.rs` — update tests
5. `tests/store_integration.rs` — update TaskFilter construction
6. `CLAUDE.md` — update CLI docs

## Decision needed

Pick Option A, B, or C before implementing.
