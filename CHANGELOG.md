# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`typst-cst` crate** — package-agnostic parse-plane structure extraction:
  the heading tree (depth, text, source span) parsed via `typst-syntax`, with no
  eval, World, or IO. A consumer-blind substrate for gating document structure.
- **`typst-edit` crate** — consumer-blind in-place Typst source editing:
  parse-plane span queries plus an atomic, non-overlapping changeset (`apply`)
  over `typst-syntax`, no eval/World/IO. `find_calls(source, name)` locates any
  function call with its positional/named argument value spans and trailing
  `[…]` content block (`find_link_targets` is the thin link-only case); `line_of`
  maps an offset to a 1-based line. Foundation for `mindtape cleanup` link
  rewriting and the speconaut scanner collapse (see
  `docs/decisions/003-swhid-cleanup-tombstones-and-shared-edit-substrate.md`).
- **`swhid-mint` crate** — mint and resolve Software Heritage identifiers
  (SWHID, ISO/IEC 18670:2025). VCS-agnostic core (`render`/`parse`/`is_swhid_url`)
  plus a `git` backend feature (via the `gix` crate, no subprocess, no libgit2)
  that mints a revision-plus-path id for a tracked file and resolves one back. The
  tombstone substrate for `mindtape cleanup`.
- **`typst-cleanup` crate** — consumer-blind orchestration for retiring Typst
  artifacts: dirty-tree guard, per-file SWHID tombstone minting, and atomic
  inbound-`link()` rewriting, planned in memory before any write. The consumer
  supplies eligibility, link identity, and tombstone policy via a `Policy` trait.
  `mindtape cleanup` rides it; speconaut adopts it under a separate task. See
  `docs/decisions/004-gix-git-substrate.md` for the git substrate.
- **`mindtape cleanup`** — retire terminal-status tasks: deletes each terminal
  task file and rewrites every surviving `link("<stem>.typ")` that pointed at it
  into an SWHID tombstone `link("swh:1:rev:<commit>;path=<path>")`. Does not
  commit — the user reviews the diff and commits. Refuses a dirty working tree on
  a real run; `--dry-run` previews the plan without writing. Atomic — every mint
  and rewrite is planned in memory before a byte is written, so a wrong-shaped
  file or an unrecoverable commit aborts with no changes. `swh:` link targets are
  treated as history pointers by the loader/validator (not task graph edges).

## [0.4.0] - 2026-06-14

Per-file rewrite. See `docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md`.

### Added

- **Per-file task model** — one task per file via `#show: task.with(...)`, with
  a `(name, date, note)` status, configurable enum/number core fields, task
  links, tags, and an `x-<ns>` extension seam for downstream tools
- **`typst-harvest` crate** — a package-agnostic, eval-only Typst World plus
  labelled-metadata-marker harvesting; a reusable substrate for file-based tools
- **`.mindtape/config.toml`** — per-project status lifecycle, enum/number field
  constraints (required-unless-terminal, bounds), tag allow-list, default filter
- **`mindtape check`** — config-driven validation incl. link resolution
- **`mindtape install-lib`** — installs the Typst library (once per machine)
- **`mindtape init`** — scaffolds project config (once per project)
- Dev tooling: parallel `just check`, sccache, cargo-machete, linecop, outdatty

### Changed

- Split into a three-crate workspace: `typst-harvest`, `mindtape` (library),
  `mindtape-cli` (binary). The filter language now runs over live-evaluated
  tasks instead of SQL
- Typst library bumped to 0.2.0 (new task DSL; breaking)

### Removed

- The `watch` daemon, the SQLite index, the `mindtape-store` crate,
  eval-hash skipping, the global cross-project agenda, inline checkbox markers
  and `#id()` identity, and the NixOS service module

## [0.3.0] - 2026-03-23

### Added

- **Global agenda filter** — `[agenda]` now supports a top-level `filter` field
  that is AND-ed into every task section, e.g. hide tasks with a future
  `#start()` date across all sections
- **Default `#start()` filter** in the built-in agenda: tasks whose start date
  hasn't arrived are automatically hidden

### Changed

- **Agenda config restructured** — `[[agenda]]` sections are now nested under
  `[agenda]` as `[[agenda.sections]]`, enabling agenda-level settings alongside
  per-section configuration (breaking change to config format)

### Fixed

- Resolve workspace version for naersk Nix build

## [0.2.0] - 2026-03-16

### Added

- **`mindtape agenda`** subcommand — configurable Typst-rendered agenda with
  pluggable sections (task list, file index)
- **File index section** for the agenda, showing indexed files and their status
- **`mindtape inspect`** improvements — comfy-table rendering, `--format typst`
  output, `~`-shortened paths, and file indexing error tracking

### Fixed

- Exclude imported duplicate tasks from query results
- Exclude nested sub-items from parent task title during evaluation
- Clear stale file errors on unchanged content hash and prune deleted files on scan
- Remove incorrect header from `mindtape list` output, adjust header levels

### Changed

- CI: use stable Rust toolchain, guard unnecessary workflow runs, allow manual
  workflow dispatch

## [0.1.0] - 2026-03-07

Initial release.

### Added

- **Typst-based task format** with `#due()`, `#start()`, `#id()`, `#tag()`,
  `#rank()` metadata functions and `#high`, `#medium`, `#low` aliases
- **Typst evaluation engine** (`mindtape-eval`) — extracts structured task data
  from `.typ` files using `typst-eval`, with AST-based write-back support
- **SQLite store** (`mindtape-store`) — indexes tasks into a queryable database
  with FTS5 full-text search, schema migrations, and a `Store` trait abstraction
- **File watcher** with `.gitignore`-aware filtering, hot-reload config, and
  cross-file dependency tracking
- **CLI commands**:
  - `mindtape <file>` — evaluate a single Typst file
  - `mindtape watch` — watch folders and build the index
  - `mindtape list` — query tasks with `--filter` expressions, `--sort`, and
    `--format` (table/json/csv/typst) output
  - `mindtape check <id>` — toggle task checkboxes (writes back to `.typ` files)
  - `mindtape set <id>` — update task properties (due, start, rank, tags)
  - `mindtape inspect` — show index stats, files, and dependency graphs
  - `mindtape id` — generate and convert base62-encoded UUIDv7 task IDs
  - `mindtape init` — install the Typst prelude library
  - `mindtape eval` — evaluate filter expressions
  - `mindtape deps` — show file dependencies
- **Filter expressions**: `has(due)`, `miss(tag)`, `has_tag("backend")`,
  `search("auth")`, comparisons on `due`, `start`, `rank`, and boolean combinators
- **NixOS module** for declarative service configuration
- **Cross-platform release binaries** (Linux x86_64/aarch64, macOS x86_64/aarch64)

[0.3.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.3.0
[0.2.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.2.0
[0.1.0]: https://github.com/mlavrinenko/mindtape/releases/tag/v0.1.0
