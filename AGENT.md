# MindTape — AI Agent Context

## What is this?

A daemon-free, file-based task tracker. Tasks are one-per-file Typst artifacts
(`#show: task.with(...)`). MindTape live-evaluates the configured roots on every
command via the Typst compiler crates — no watch daemon, no SQLite, no global
index. Identity is the filename stem.

Repository: https://github.com/mlavrinenko/mindtape

## Key Docs

- `CONTRIBUTING.md` — workflow, commit style, testing, quality gates
- `docs/decisions/` — Architecture Decision Records (ADRs); 002 is the rewrite
- `docs/redesign/` — the redesign brief and DSL shape probe (historical)
- `lib/prelude.typ` — the `@local/mindtape` Typst package (the task DSL)
- `truth/tasks/` — MindTape's own backlog as per-file task artifacts (dogfood);
  `mindtape list` / `mindtape check` run against it via the in-tree `lib/`

## Workspace

Three crates (`cargo` workspace, virtual root manifest):

- `crates/typst-harvest` — package-agnostic eval-only Typst World + metadata
  marker harvesting. Destined to become its own repository; keep it
  consumer-blind (it knows nothing about tasks).
- `crates/mindtape` — domain library: config, task model, filter language,
  validation, project loader. Other tools (e.g. speconaut) depend on this.
- `crates/mindtape-cli` — the thin `mindtape` binary (clap).

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
   machete, file-size, outdatty drift). Apply `cargo fmt` first.
3. File size: Rust ≤500 lines, Markdown ≤200 (enforced by `linecop`). Split up
   front; `just fix-check` ejects oversized inline tests via `ejectest`.
4. When `outdatty` reports drift, update the listed dependents, then
   `just outdatty-update` and commit `outdatty.lock`.
5. Record architectural/tooling decisions as an ADR in `docs/decisions/` using
   the next number. Include Status, Date, Context, Decision, Consequences,
   Deprecation Criteria.
6. Commits: Conventional Commits; footer `Refs: truth/tasks/<stem>.typ` (the
   driving task) or `docs/decisions/00N-<slug>.md` for an architectural decision.
   Match prior commit language.

## Task DSL (lib/prelude.typ)

```typ
#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Title",
  status: wip(2026, 6, 14),         // (name, date, note) dict
  priority: "high",                  // configurable enum
  estimate-hours: 8,
  depends-on: (link("other.typ"),),  // task↔task links by file stem
  tags: ("backend",),
  x-speconaut: ...,                  // opaque extension slot (x-<ns>)
)
```

The prelude enforces structure only (one task, the `x-<ns>` discipline). The
Rust layer enforces config-driven enums / required-unless-terminal / bounds /
link resolution. Consumers validate their own `x-<ns>` fields.

## Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver)
- Typst library: `lib/typst.toml` `version` (independent). On bump, update the
  `TYPST_PKG_VERSION` const in `crates/mindtape-cli/src/init.rs` and the import
  examples in `README.md`, `www/index.html`, and this file.

## Context Hygiene

- Keep agent-facing docs under 200 lines; reference instead of duplicating.
