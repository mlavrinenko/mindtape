# MindTape Code Review

Date: 2026-02-15

## Summary

40 source files, 5,820 lines, 225 tests passing. Project is well-structured
but has accumulated documentation bloat, cross-file duplication, and stale
information that wastes agent context window on every interaction.

---

## 1. File Size Analysis

### Top 10 context-consuming files

| File | Lines | Bytes | Notes |
|------|------:|------:|-------|
| `src/cli.rs` | 725 | 25,648 | Largest file; arg parsing + 6 output formatters |
| `crates/mindtape-store/src/store/sqlite.rs` | 530 | 18,686 | SQL + tests inline |
| `crates/mindtape-eval/src/eval.rs` | 386 | 14,481 | Core eval logic |
| `src/watcher.rs` | 332 | 11,814 | Watcher + inline tests |
| `docs/DESIGN.md` | 269 | 8,993 | Bloated, duplicates CLAUDE.md |
| `CLAUDE.md` | 236 | 8,398 | Auto-loaded every session |
| `README.md` | 210 | 8,104 | |
| `tests/query_integration.rs` | 208 | 7,285 | |
| `src/config.rs` | 187 | 6,413 | |
| `crates/mindtape-store/src/store/indexer.rs` | 186 | 6,373 | |

### Category breakdown

| Category | Files | Lines | % |
|----------|------:|------:|--:|
| Rust source | 11 | 2,769 | 48% |
| Rust tests (integration) | 4 | 706 | 12% |
| Documentation | 14 | 1,203 | 21% |
| Config/build/scripts | 10 | 278 | 5% |

---

## 2. Code Duplications

### 2.1 `fn ymd` — duplicated test helper (HIGH)

Identical function in two files:

- `src/cli.rs:848` (test module)
- `tests/eval_integration.rs:60`

```rust
fn ymd(year: i32, month: u8, day: u8) -> Datetime {
    Datetime::from_ymd(year, month, day).unwrap()
}
```

### 2.2 Test setup boilerplate — duplicated 4x (HIGH)

Four integration test files repeat the same ~25-line pattern creating a temp
project with `lib/prelude.typ` + `lib/typst.toml` + `Cargo.toml`:

- `tests/eval_integration.rs` (lines 9-45, two setup variants)
- `tests/store_integration.rs` (lines 9-32)
- `tests/query_integration.rs` (lines 12-31)
- `tests/watcher_integration.rs` (lines 14-36)

The prelude content, typst.toml content, and directory structure are identical.
No shared `tests/common/` module exists.

### 2.3 File dependency queries — duplicated in sqlite.rs (MEDIUM)

`get_file_dependencies()` and `list_file_dependencies()` contain identical
~25-line blocks that query imports and imported_by from `file_references`.
Should be extracted to a private helper method.

### 2.4 TaskView construction — 3 inline copies (MEDIUM)

The tuple-to-TaskView construction with `fetch_task_properties()` appears in:
- `query_tasks()` (inline loop)
- `search()` (inline loop)
- `query_agenda()` (via `fetch_task_views()` helper)

The helper `fetch_task_views()` already exists but is only used by `query_agenda()`.
The other two methods should reuse it.

### 2.5 CSV formatters — similar boilerplate x6 (LOW)

Six CSV formatting functions in `cli.rs` follow the same header+rows pattern.
Not identical enough for a single abstraction, but a small macro or builder
could reduce ~60 lines of repetition.

### 2.6 Datetime formatting — duplicated logic (LOW)

`format_due()` in `cli.rs:817` and inline formatting in `eval.rs:180` both
extract `year/month/day` with `unwrap_or(0)` and format as `YYYY-MM-DD`.

---

## 3. Documentation Problems

### 3.1 Stale test counts across docs

| Document | Claims | Actual |
|----------|--------|--------|
| `CLAUDE.md` | 160 tests | **225 tests** |
| `ROADMAP.md` (M3.1) | 217 tests | **225 tests** |
| `TESTING.md` | 137 tests (references M1.4) | **225 tests** |
| `MEMORY.md` | 153 tests, then 160 | **225 tests** |

Every doc has a different stale number. Should be a single source of truth
or not tracked manually at all (use `just test` output).

### 3.2 DESIGN.md bloat and misplaced content

**269 lines** — the largest doc. Problems:

- **Duplicates CLAUDE.md**: workspace layout (identical), tech stack/dependencies,
  architecture diagram — all repeated from CLAUDE.md
- **Contains code**: Store trait signature, eval API signature, content tree example,
  prelude.typ source — all belong in source files as doc comments
- **Has milestone markers**: "M3.1 (Complete)", "M3.2 (Planned)" — belongs in ROADMAP.md
- **Mixes concerns**: design rationale + implementation details + API reference + status

### 3.3 ROADMAP.md is a wall of completed milestones

**134 lines**, ~80% is completed milestone details (M1.1–M3.1) with implementation
notes that are only useful as historical reference. The active work (M3.2) is
buried at the bottom.

### 3.4 TESTING.md references "M1.4" metrics

Still says "Current metrics (M1.4)" with 137 tests. Project is at M3.1 with 225 tests.
Coverage percentages are stale.

### 3.5 Research docs not archived

`docs/research/write-back.md` (~200 lines) was research for M3.1 which is now
complete. The "Next Steps" section still lists items as future work. Six other
research files are also historical — useful as reference but not needed in
regular context.

### 3.6 CLAUDE.md is decent but could be leaner

At 236 lines it's auto-loaded every session. The workspace layout with LOC
estimates and the CLI command reference are the most useful sections. Could
trim ~30 lines by removing the data flow ASCII art (also in DESIGN.md) and
condensing the testing section (just `just check` and `just all` would suffice).

---

## 4. Structural Recommendations

### 4.1 Split ROADMAP.md into milestones/

```
docs/
  ROADMAP.md              # Index: list milestones, status, link to details
  milestones/
    m1-mvp.md             # M1.1-M1.5 completed details
    m2-queries.md         # M2.1-M2.3 completed details
    m3-writeback.md       # M3.1 complete, M3.2 planned
```

ROADMAP.md becomes a concise index (~30 lines) that shows what's done and
what's next. Detail files are only read when working on specific milestones.

### 4.2 Split DESIGN.md into focused docs

```
docs/
  DESIGN.md               # Core: why Typst, data model, architecture diagram (~80 lines)
  design/
    typst-eval.md          # Eval strategy, content tree, extraction algorithm
    store.md               # Store trait, schema, indexing pipeline
    watcher.md             # File watching, config, ignore patterns
    writeback.md           # Write-back strategy, conflict detection
```

Remove all code snippets from DESIGN.md — link to source files instead.
Remove all milestone completion markers — that's ROADMAP.md's job.

### 4.3 Introduce archive/ folder

```
archive/
  research/               # Move docs/research/ here (6 files, ~260 lines)
  milestones/             # Or: keep completed milestone details here
```

Still accessible but won't be read routinely.

### 4.4 Create tests/common.rs

Shared test infrastructure:
- `setup_typst_project()` — creates temp dir with lib/prelude.typ + typst.toml
- `ymd()` — datetime helper
- Common prelude content as a constant

This eliminates ~100 lines of duplication across 4 test files.

### 4.5 Optimize CLAUDE.md for context efficiency

CLAUDE.md is auto-loaded every session. Keep it as lean as possible:

- Add a "files to read upfront" section listing small, frequently-needed files
- Remove content duplicated in DESIGN.md (keep it in one place only)
- Add an instruction: "Keep auto-loaded files under N lines"
- Add self-maintenance instruction: "After modifying any auto-loaded file,
  verify it hasn't grown beyond its target size"

### 4.6 Use Claude Code Skills for cross-project instructions

Skills can extract non-project-specific instructions to `~/.claude/skills/`:

```
~/.claude/skills/
  code-review/SKILL.md          # Review methodology, report format
  project-hygiene/SKILL.md      # Doc maintenance rules, archive patterns
  rust-conventions/SKILL.md     # Rust-specific patterns, testing conventions
```

These are available across all projects. Project-specific CLAUDE.md then
references skill conventions instead of repeating them.

Relevant docs:
- https://code.claude.com/docs/en/skills
- https://github.com/anthropics/skills (Agent Skills open standard)

Skills support:
- Personal (`~/.claude/skills/`) — available across all projects
- Project (`.claude/skills/`) — committed to repo
- Frontmatter for invocation control, tool restrictions, subagent execution
- Supporting files (templates, examples, scripts) alongside SKILL.md

Example personal skill for project hygiene:

```yaml
# ~/.claude/skills/project-hygiene/SKILL.md
---
name: project-hygiene
description: Rules for documentation maintenance, archiving, and context optimization
user-invocable: false
---
- Keep auto-loaded files (CLAUDE.md, MEMORY.md) under 200 lines
- Archive completed work to archive/ folder
- Use index files that link to details rather than inline everything
- Track test counts via CI, not manually in docs
- After any doc change, verify no duplication was introduced
```

---

## 5. Additional Suggestions

### 5.1 Justfile improvements

Current Justfile has basic recipes. Consider adding:

```just
# Count tests
count-tests:
    cargo test --workspace 2>&1 | grep "^test result:" | awk '{sum += $4} END {print sum " tests"}'

# Quick review: show file sizes sorted
file-sizes:
    find . -type f \( -name '*.rs' -o -name '*.md' \) ! -path './target/*' -exec wc -l {} + | sort -rn | head -20

# Lint docs: check for stale test counts, broken links
lint-docs:
    @echo "Test count:" && just count-tests
```

### 5.2 Auto-maintenance instructions in CLAUDE.md

Add a section to CLAUDE.md:

```markdown
## Context Hygiene (Self-Maintenance)

After completing any task:
1. If you modified a doc file, check it didn't grow beyond its target size
2. If you added tests, don't update test counts in docs (they go stale)
3. If you created a new pattern, check if it duplicates an existing one
4. Keep CLAUDE.md under 200 lines, DESIGN.md under 100 lines
```

### 5.3 Remove manual test counts from all docs

Test counts go stale immediately. Options:
- Remove them entirely (use `just count-tests` when needed)
- Or: keep only in ROADMAP.md milestone completion notes as historical snapshots

### 5.4 Single source of truth for schema

The database schema is described in DESIGN.md, store CLAUDE.md, and sqlite.rs.
The source code should be the only authority. Docs should say
"See `sqlite.rs` for current schema" with a brief summary.

---

## 6. Task List

### Documentation restructuring

- [ ] **[Sonnet]** Create `docs/milestones/` folder; split ROADMAP.md completed milestones (M1.x, M2.x, M3.1) into separate files; make ROADMAP.md a concise index
- [ ] **[Sonnet]** Create `docs/design/` folder; split DESIGN.md into focused files (typst-eval, store, watcher, writeback); keep core DESIGN.md under 80 lines
- [ ] **[Sonnet]** Remove all code snippets from DESIGN.md — replace with "See `<file>:<line>`" references
- [ ] **[Sonnet]** Remove milestone completion markers from DESIGN.md (belongs in ROADMAP.md only)
- [ ] **[Sonnet]** Create `archive/` folder; move `docs/research/` there (6 completed research files)
- [ ] **[Sonnet]** Remove stale test counts from CLAUDE.md, TESTING.md, MEMORY.md — add note "run `just count-tests` for current count"
- [ ] **[Sonnet]** Update TESTING.md: remove "M1.4" reference, remove stale coverage table, keep guidelines only
- [ ] **[Sonnet]** Trim CLAUDE.md: remove data flow diagram (keep in DESIGN.md), condense testing section, add "Context Hygiene" rules; target under 200 lines
- [ ] **[Sonnet]** Add "files to read upfront" list to CLAUDE.md with target line counts for each

### Code deduplication

- [ ] **[Sonnet]** Create `tests/common.rs` with shared `setup_typst_project()`, `ymd()`, and prelude constant; refactor all 4 integration test files to use it
- [ ] **[Opus]** Extract `fetch_file_deps()` private method in `sqlite.rs` to deduplicate `get_file_dependencies()` and `list_file_dependencies()`
- [ ] **[Opus]** Refactor `query_tasks()` and `search()` in `sqlite.rs` to use existing `fetch_task_views()` helper
- [ ] **[Sonnet]** Move `format_due()` to `mindtape-eval` crate as a public utility; use it in both `cli.rs` and `eval.rs`
- [ ] **[Sonnet]** Deduplicate `fn ymd()` — move to `tests/common.rs`

### Justfile & tooling

- [ ] **[Sonnet]** Add `count-tests` recipe to Justfile
- [ ] **[Sonnet]** Add `file-sizes` recipe to Justfile (top 20 files by line count)

### Cross-project skills

- [ ] **[Opus]** Design and create `~/.claude/skills/project-hygiene/SKILL.md` — doc maintenance rules, archiving, context optimization
- [ ] **[Opus]** Design and create `~/.claude/skills/rust-conventions/SKILL.md` — testing patterns, error handling, workspace conventions
- [ ] **[Sonnet]** Extract non-project-specific agent rules from CLAUDE.md into skills; keep only MindTape-specific rules in CLAUDE.md

### Memory & self-maintenance

- [ ] **[Sonnet]** Update MEMORY.md: fix stale test counts, remove info duplicated with CLAUDE.md, keep it under 200 lines
- [ ] **[Sonnet]** Add self-maintenance instructions to CLAUDE.md: check file sizes after edits, avoid introducing duplication, keep auto-loaded files lean

---

## 7. Priority Order

**Phase 1 — Quick wins (reduce context waste immediately)**
1. Create `tests/common.rs` and deduplicate test setup
2. Remove stale test counts from all docs
3. Trim CLAUDE.md to under 200 lines
4. Create `archive/` and move research docs

**Phase 2 — Documentation restructuring**
5. Split ROADMAP.md into milestones/
6. Split DESIGN.md into design/
7. Update TESTING.md
8. Add context hygiene rules

**Phase 3 — Code quality**
9. Deduplicate sqlite.rs (deps query + TaskView construction)
10. Move `format_due()` to shared location
11. Justfile improvements

**Phase 4 — Cross-project infrastructure**
12. Create personal skills for project hygiene
13. Create personal skills for Rust conventions
14. Refactor CLAUDE.md to reference skills
