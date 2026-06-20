# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the full toolchain
just check           # run before every commit
```

Use `just` recipes instead of raw `cargo` — they carry the right flags and
environment. `just` (no args) lists them.

## Code Style

All clippy lints are `deny` (workspace `Cargo.toml`); the project will not build
with violations. Key restrictions:

- No `unwrap()`/`expect()` outside tests — use `?` with `anyhow`/`thiserror`
- No `todo!()`/`unimplemented!()`/`unreachable!()`/`unsafe`/wildcard imports
- Functions: ≤70 lines, ≤5 args, cognitive complexity ≤20

## Error Handling

- `anyhow::Result` for application code (the CLI)
- `thiserror::Error` for library error types callers match on
  (`HarvestError`, `ConfigError`, `FilterError`)
- Keep `main.rs` a thin entry point; logic lives in library modules

## Testing

- Unit tests inline in `#[cfg(test)] mod tests` (or a sibling `_tests.rs` via
  `#[path]`) next to the code they exercise.
- As a file nears the linecop limit, `just fix-check` ejects its inline tests
  into a `_tests.rs` sibling with [ejectest](https://github.com/mlavrinenko/ejectest).
- Run `just test`; count with `just count-tests`.

## Quality Gates (run by `just check`)

| Gate | Tool |
|------|------|
| Format | `cargo fmt --check` |
| Lints | `cargo clippy --all-targets -D warnings` |
| Tests | `cargo test --workspace` |
| Unused deps | `cargo machete` |
| File size | `linecop` (Rust 500, Markdown 200) |
| Copy-paste | `jscpd` (`just check-dry`) — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper |
| Doc drift | `outdatty check` |

Coverage runs separately (`just cover`, CI `coverage` job) via `cargo-tarpaulin`.
`main.rs` is excluded — keep it thin.

`just nix-build` verifies that `nix build` succeeds in a clean sandbox (no dev
shell, no sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; it
is called automatically by `just release`.

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) couples source files to the dependents that must
stay in sync (e.g. the CLI surface to `README.md`). After editing a source,
review the listed dependents, update them, then run `just outdatty-update` to
record the new state into `outdatty.lock` and commit it.

## Commit Messages

[Conventional Commits](https://www.conventionalcommits.org/). Match the language
of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: truth/tasks/<stem>.typ
```

MindTape tracks its own backlog as per-file task artifacts under `truth/tasks/`
(MindTape dogfooding itself — see `.mindtape/config.toml`). Point `Refs:` at the
driving task file (`truth/tasks/<stem>.typ`). Architectural and tooling decisions
are tracked as task artifacts too; the ADR process is retired (`docs/decisions/`
is frozen history).

Commits that file or edit task artifacts themselves (and the backlog config in
`.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs nothing.

## Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args` struct
  and a `run` fn; wire it into `cli.rs`. See `list.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
