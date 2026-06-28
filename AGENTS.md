# MindTape — AI Agent Context

## What is this?

A daemon-free, file-based task tracker. Tasks are one-per-file Typst artifacts
(`#show: task.with(...)`). MindTape live-evaluates the configured roots on every
command via the Typst compiler crates — no watch daemon, no SQLite, no global
index. Identity is the filename stem.

Repository: https://github.com/mlavrinenko/mindtape

## Key Docs

- `CONTRIBUTING.md` — workflow, commit style, testing, quality gates
- `docs/decisions/` — retired ADRs, kept as frozen history; decisions now live
  as task artifacts under `truth/tasks/`
- `docs/redesign/` — the redesign brief and DSL shape probe (historical)
- `lib/prelude.typ` — the `@local/mindtape` Typst package (the task DSL)
- `truth/tasks/` — MindTape's own backlog as per-file task artifacts (dogfood);
  `mt ls` / `mt check` run against it via the in-tree `lib/`

## Workspace

Four crates (`cargo` workspace, virtual root manifest):

- `crates/typst-harvest` — package-agnostic eval-only Typst World + metadata
  marker harvesting. Destined to become its own repository; keep it
  consumer-blind (it knows nothing about tasks).
- `crates/typst-cst` — package-agnostic parse-plane structure extraction
  (heading tree with source spans) over `typst-syntax`; no eval, no World, no
  IO. Consumer-blind sibling to `typst-harvest`.
- `crates/mindtape` — domain library: config, task model, filter language,
  validation, project loader. Other tools (e.g. speconaut) depend on this.
- `crates/mindtape-cli` — the thin `mt` binary (clap).

## Tech Stack

- Rust (edition 2024, pinned `rust-toolchain.toml`)
- Typst 0.14 eval-only (`typst-eval`); never layout or render
- `evalexpr` 12 as a parser for the filter language (walked, not evaluated by it)
- `clap` 4 CLI; `serde`/`toml` config; `comfy-table`/`serde_json`/`csv` output
- Nix flake + `just`; dev tools: sccache, cargo-machete, linecop, outdatty,
  ejectest, cargo-tarpaulin

## Agent Rules

1. Use `just` recipes, not raw cargo (see `Justfile`). `just check` before done.
2. After code changes, `just check` must pass (fmt, clippy `-D warnings`, tests,
   machete, file-size, copy-paste, outdatty drift). Apply `cargo fmt` first.
   Copy-paste: `just check-dry` (jscpd) fails on any duplicated Rust block ≥70
   tokens — fix by extracting a shared helper, never by shuffling tokens.
3. File size: Rust ≤500 lines, Markdown ≤200 (enforced by `linecop`). Split up
   front; `just fix-check` ejects oversized inline tests via `ejectest`.
4. When `outdatty` reports drift, update the listed dependents, then
   `just outdatty-update` and commit `outdatty.lock`.
5. Record architectural/tooling decisions as task artifacts under `truth/tasks/`,
   not as ADRs. The ADR process is retired; `docs/decisions/` is frozen history.
6. Commits: Conventional Commits; footer `Refs: truth/tasks/<stem>.typ` (the
   driving task). Exception: commits that file or edit task artifacts themselves
   (and the backlog config in `.mindtape/config.toml`) carry no `Refs:` footer —
   a task commit refs nothing. Match prior commit language.

## Task DSL (lib/prelude.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Title",
  status: wip(2026, 6, 14)[note],    // (name, date, note); note is a trailing block
  priority: ice(impact: 8, confidence: 0.7, ease: 6), // or a raw int; difficulty too
  estimate: (days: 1, hours: 2),     // opaque unit dict; allowed units per project
  depends-on: (link("other.typ"),),  // task↔task links by file stem
  tags: ("backend",),
  x-speconaut: ...,                  // opaque extension slot (x-<ns>)
)
```

Priority can be a raw int on the scale or a framework call: thin packagers
`ice/rice/wsjf/eisenhower/moscow` (and the generic `framework(name, ..inputs)`)
shape a `(framework, inputs)` dict — no math in Typst. Config picks the active
`[tasks.priority] framework`; the Rust layer scores it (evalexpr), bands or
categorises the result, and keeps it on the scale.

The prelude enforces structure only (one task, the `x-<ns>` discipline). The
Rust layer enforces the status legend, numeric priority/difficulty scales and
classification, priority frameworks (scoring + in-bounds-by-construction), the
kind enum, required-unless-terminal, bounds, and link resolution. Consumers
validate their own `x-<ns>` fields.

### Templates (`mt add`)

`mt add [TITLE…] [-t template]` files a new task from a Typst template — a
task whose authored values are `slot(name, kind: …, default: …, prompt: …)` calls.
The positional words join into the `title` slot (`add cleanup kitchen` →
`cleanup-kitchen.typ`); `-t/--template` picks the template (default `[templates]
default`). `add` discovers each slot on the parse plane (`typst-edit`, no eval),
fills it from `-f values.json` / `--set k=v` / the positional title / the slot
default, injects the add-owned `status` (initial legend value, dated today) and
`id`, then writes `<root>/<stem>.typ` and validates it. `--describe[=json]` dumps
the slot schema; `--print` previews; `--into .` targets the first root; `--to
<subdir>` nests under it (`--to shopping` → `<root>/shopping/`). Templates live under
`[templates] dirs`; the built-in `task` fallback is barebones (one required
`title` slot). Filled tasks hold plain literals — no `slot` call survives.

Config-required core fields the chosen template omits are spliced in as slots
before discovery (`mindtape::required_core_slots`, `crates/mindtape/src/add_slots.rs`):
a required `priority`/`difficulty` → `int` slot, required `kind` → `choice` (its
allowed values), required `estimate` → `text` slot for the unit dict. So a
required field always surfaces in the scaffold, `--describe`, and the written
task even with the barebones template. Strict-framework priority (`framework` +
`allow-manual = false`) has no scalar form and is not spliced — declare it in a
template.

`--edit` (or `[add] interactive`) opens an editor on a commented YAML scaffold of
the slots (block scalars for `text` slots), then parses the saved buffer back
into the same fill values; editor resolves `[add] editor` → `$VISUAL` → `$EDITOR`
→ `vi`, and the editor only opens with a TTY (`crates/mindtape-cli/src/add_interactive.rs`).
The scaffold tail carries no trailing blank (jump-to-end lands on the last
fillable line). `[add] header` overrides the instruction block: custom text (auto
comment-prefixed) or `""` to drop it.

### Derived tags (`[tasks.derive-tags]`, `crates/mindtape/src/derive.rs`)

Tasks inherit virtual tags from their location under a root: the root name (the
table-form `[[tasks.roots]] name`, else the path's last segment) plus folder
segments. `collect` picks `segments` (`backend`, `auth`) and/or `breadcrumbs`
(`backend`, `backend:auth`); per-root `derive-tags` (bool/glob/list) gates which
files derive. Tags are virtual (never written), live on `Task.derived_tags`
(authored `tags` stay pristine; `Task::all_tags` unions them for filters), and
are validated against `[tasks.tags] allowed` strictly. `derive.rs` is pure and
consumer-blind enough for speconaut to converge onto.

## Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver)
- Typst library: `lib/typst.toml` `version` (independent). On bump, update the
  `TYPST_PKG_VERSION` const in `crates/mindtape-cli/src/init.rs` and the import
  examples in `README.md`, `www/index.html`, and this file.

## Context Hygiene

- Keep agent-facing docs under 200 lines; reference instead of duplicating.

## Temporary — active only on branch `redesign/per-file-rewrite`

Remove this whole section when this branch merges to `main`.

- Convergence directive: design `crates/mindtape` and its consumer-blind siblings
  (`typst-harvest`, `typst-cst`, and a planned `typst-edit`) so speconaut can delete
  its generic code and depend on them. Any generic capability speconaut hand-rolls —
  Typst-source span location, slot/link rewriting, git tombstone resolution — is a
  candidate to host here.
- Research workflow: spawn mid-tier sub-agents for codebase sweeps, but read the cited
  key files yourself to validate before deciding. speconaut reference points:
  `src/commands/{cleanup_cmd,cleanup_refs,cleanup_git_refs,link_scanner,git_ref_scanner}.rs`,
  `src/commands/{typ_source,specs_stage_rewrite}.rs`, `src/git.rs`,
  `src/engine/dsl_links.rs`.
- 0.2 is on pre-release stage. Don't bump Typst lib or other packages versions.
