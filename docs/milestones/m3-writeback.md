# Milestone 3: Write-Back (Mutations via API)

## M3.1 — Task Checkbox Toggle [COMPLETE]

- [x] `mindtape check <task-id>` — toggle task checkbox in `.typ` file
- [x] Task ID masking: `*37f8` matches tasks with IDs ending in "37f8"
- [x] Safe file modification via AST: parse -> find task -> modify checkbox -> write back
- [x] Conflict detection: verify file hash matches last index, error if changed
- [x] Format preservation: keep whitespace, comments, surrounding content intact

**Implementation notes**:
- New `write` module in `mindtape-eval` with AST-based modification (~140 LOC)
- Uses `typst-syntax` to parse files and locate `ListItem` nodes by ID
- `find_task_by_id()` method in Store trait with masked pattern support
- Conflict detection via SHA-256 hash comparison (matches indexer)
- 8 new unit tests covering toggle, format preservation, error cases
- 217 total tests passing (was 209 before M3.1)

## M3.2 — Task Property Updates [COMPLETE]

- [x] `mindtape set <task-id> --due <date>` — set/update task due date
- [x] `mindtape set <task-id> --no-due` — remove task due date
- [x] `mindtape set <task-id> --add-tag <tag>` — add a tag
- [x] `mindtape set <task-id> --remove-tag <tag>` — remove a tag
- [x] Combinable flags: `--due`, `--add-tag`, `--remove-tag` in one call
- [x] Atomic writes using tempfile crate (applied to both `check` and `set`)

**Implementation notes**:
- Extended `write` module with `set_task_due`, `remove_task_due`, `add_task_tag`, `remove_task_tag`
- Shared `modify_task_text` helper factors out find-task + reconstruct pattern
- Text-level pattern matching (no regex): `find_due_span`, `find_tag_span`, `find_matching_paren`
- Properties inserted before `#id(...)` to maintain conventional ordering
- Date input: ISO `YYYY-MM-DD` → Typst `datetime(Y, M, D)` with validation
- Space cleanup on removal: `trim_leading_space` avoids double spaces
- Atomic writes via `tempfile::NamedTempFile` + `persist()` (both `check` and `set`)
- 25 new unit tests + 4 CLI parse tests
