# MindTape — AI Agent Context

## What is this?

A file-based task tracker that uses Typst (`.typ`) files as the source of truth.
Watches folders, evaluates Typst files via the compiler crates, indexes tasks
and metadata into SQLite, and exposes a CLI for querying.

**Repository**: https://github.com/mlavrinenko/mindtape

## Key Docs

- `CONTRIBUTING.md` — workflow, commit style, testing, quality standards
- `crates/mindtape-eval/AGENT.md` — eval crate architecture and technical notes
- `crates/mindtape-store/AGENT.md` — store crate architecture and schema procedures

## Tech Stack

- **Language**: Rust (workspace: `mindtape-eval` <- `mindtape-store` <- `mindtape`)
- **Build**: Nix flake + `just` task runner
- **Typst**: `typst`, `typst-eval`, `typst-library`, `typst-syntax` (0.14), `comemo` (0.5)
- **Database**: SQLite via `rusqlite` (behind a `Store` trait)
- **CLI**: `clap` 4 (derive), `serde_json` for `--json` output
- **Logging**: `log` 0.4 + `env_logger` 0.11

## Agent Rules

1. Use `just` recipes instead of raw cargo commands (see `Justfile`)
2. After any code changes, run `just check` and fix all warnings
3. Keep files small: Rust ≤500 lines, Markdown ≤200 lines (enforced by `just check-file-size`)
4. After completing a task, suggest a conventional commit message
5. Follow patterns in `CONTRIBUTING.md` for new CLI commands and store methods

## Architecture Principles

- Typst files are always the source of truth — the database is a derived index
- Evaluation-only: we use `typst_eval::eval()`, never layout or render
- Anti-corruption layer: `Store` trait abstracts the database
- Workspace with focused crates; root crate is the CLI binary
- All pure logic is testable; main.rs is a thin CLI wrapper

## Task Format (Typst Convention)

```typ
#import "@local/mindtape:0.1.0": due, start, id, tag, rank, high, medium, low

= Milestone Title

- [ ] task text #due(2026, 3, 1) #id("uuid") #tag("category")
- [ ] upcoming task #start(2026, 2, 1) #due(2026, 3, 1) #high #id("uuid2")
- [ ] minor fix #low #id("uuid3")
- [x] completed task
```

## Context Hygiene

- Keep CLAUDE.md under 200 lines — it's auto-loaded every session
- Don't duplicate content across docs — reference instead
- Archive completed research to `archive/research/`
