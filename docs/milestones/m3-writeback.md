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

## M3.2 — Task Property Updates [PLANNED]

- [ ] `mindtape set <task-id> --due <date>` — update task due date
- [ ] `mindtape set <task-id> --tag <tag>` — add/remove tags
- [ ] Support for multi-line task modification
- [ ] Atomic writes using tempfile crate
