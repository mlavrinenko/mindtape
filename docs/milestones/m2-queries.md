# Milestone 2: Richer Queries + UX

## M2.1 — Search + Output Formats [COMPLETE]

- [x] Full-text search across task titles and `#let` binding values
- [x] `mindtape search "keyword"` command
- [x] Output formats: table (default), JSON, CSV via `--format` flag
- [x] `--json` kept as shorthand for `--format json`

**Implementation notes**:
- `Store::search()` method with LIKE-based matching (case-insensitive)
- `BindingView` and `SearchResults` domain types
- Schema v2 migration adds NOCASE indexes on `tasks.title` and `file_bindings.name`
- `OutputFormat` enum replaces `bool json` across all query commands
- CSV formatters for tasks, files, stats, and search results
- 204 tests across workspace

## M2.2 — Due Date Awareness + Agenda [COMPLETE]

- [x] Due date awareness: overdue tasks, upcoming tasks
- [x] `mindtape agenda` — tasks due today/this week

**Implementation notes**:
- `Store::query_agenda()` method returns `AgendaView` with overdue/today/this_week tasks
- Overdue uses `<` (strictly before today), not `<=`
- This week = today + 6 days (7 day window including today)
- CLI supports `--overdue`, `--today`, `--week` flags (default: all sections)
- All output formats supported: table, JSON, CSV
- Added 3 store tests + 4 CLI tests (153 total tests)

## M2.3 — Cross-File References [COMPLETE]

- [x] Track file dependencies during Typst evaluation
- [x] Store cross-file references in database (schema v3)
- [x] `mindtape deps` command to query dependencies
- [x] `mindtape deps --file <path>` to show specific file dependencies

**Implementation notes**:
- `MindTapeWorld` tracks accessed files via `source()` and `file()` methods
- `eval_file_full_with_deps()` extracts dependencies after evaluation
- Schema v3 adds `file_references` table with CASCADE DELETE
- `Store::upsert_file_references()`, `get_file_dependencies()`, `list_file_dependencies()`
- CLI supports table/JSON/CSV output formats for dependency queries
- 7 new tests covering reference tracking and queries (160 total tests)
