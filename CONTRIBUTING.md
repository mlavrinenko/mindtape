# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the full toolchain (installs the
                     # pre-commit hook: asserts the gate is fresh, checks
                     # task files)
just check           # run before every commit
```

Use `just` recipes instead of raw `cargo` — they carry the right flags and
environment. `just` (no args) lists them. Tool inventory is `flake.nix`'s
subject, commented in place; this file covers the workflow around them.

This document is the single source for how work is done here — a rule about
process belongs here and nowhere else. `AGENTS.md` pulls it into agent context
and describes only what MindTape *is*. Deep dives sit in `docs/contributing/`.

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
- Tests that shell out to the `mt` binary (`assert_cmd::cargo_bin`) belong in
  `tests/` (integration tests), not inline: Cargo builds the binary and exports
  `CARGO_BIN_EXE_mt` before integration tests, a guarantee `--lib` unit tests
  lack.
- As a file nears the linecop limit, `just fix-check` ejects its inline tests
  into a `_tests.rs` sibling with [ejectest](https://github.com/mlavrinenko/ejectest).
- Run `just test`; count with `just count-tests`.
- Windows and the pty-driven terminal tests have their own seams — CI job,
  `just check-windows` cross-check, why `tests/interactive_e2e.rs` is
  `#![cfg(unix)]`: [docs/contributing/windows.md](docs/contributing/windows.md).
  Anything you write that shells out or joins paths needs a thought for that job.
- Mutation testing (`cargo-mutants`) proves whether tests pin behaviour, not just
  execute it. Run `just mutants-diff` before a PR and triage every survivor:
  [docs/contributing/mutation-testing.md](docs/contributing/mutation-testing.md).

## Quality Gates (run by `just check`)

| Gate | Tool |
|------|------|
| Format | `just fmt-check` — `cargo fmt` over Rust, `tombi format` over the project config TOML (`.mindtape/config.toml` and the per-example configs), and `typstyle` over the Typst sources (`lib/`, `truth/tasks/`, both template dirs, the learn-piano example, and `www/`). The non-Rust path lists live in the `TOML_FMT_PATHS`/`TYPST_FMT_PATHS` Justfile variables, one list feeding both `fmt` and `fmt-check` — add a path there, never to one recipe |
| Lints | `cargo clippy --all-targets -D warnings` |
| Tests | `cargo test --workspace` |
| Unused deps | `cargo machete` |
| File size | `linecop` — per-language caps and the tighter ones on the always-loaded docs are in [`.linecop.yaml`](.linecop.yaml), argued in place. `linecop --baseline 85` lists what is nearing its cap; run it before adding to an existing file |
| Copy-paste | `jscpd` (`just check-dry`) — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper |
| Doc drift | `outdatty check` |
| Fixtures | `just check-examples` — `mt check` against each example project: `examples/learn-piano/` (full config surface; feeds the docs generator and the skill's triage evals) and `examples/minimal/` (the floor config; feeds the skill's trap evals). Keep the recipe and the mmz `examples` scope in step. A fixture no gate validates rots silently |
| Doc coverage | `just check-doc-coverage` — set-differences `mt _cli --json`'s reflected command names against `www/utils/cli-notes.typ`'s dict keys; fails naming any visible subcommand with no matching entry, so a new verb can no longer ship silently undocumented |
| Doc facts | `just check-doc-facts` — regenerates `www/generated/{crate-map,dsl-fields}.json` (`www/generate-facts.sh`) and set-differences each against its `www/utils/{crate-notes,dsl-field-notes}.typ` prose dict; fails naming any crate or DSL field with no matching entry |
| Docs site | `just docs-check` — `docs-gen` first captures live `mt` output (`www/generate.sh`), then `tola build`, then `pagefind` indexes the built HTML for client-side search, then `tola validate` checks links + assets |
| CRAP (partial) | `just check-crap` — runs the CRAP gate when `target/coverage/lcov.info` is fresh, and otherwise SKIPS LOUDLY, saying so and exiting 0. Editing any source stales the lcov, so on the commits that matter it skips; it catches the free case only. `just check-full` is the arm that is always correct |

`just check-full` is `just check` plus `just cover` and `just crap` — always
correct about coverage and CRAP, at the price of an instrumented rebuild and a
full suite pass. Use it before opening a PR, plain `just check` while iterating.

A delegated worktree whose full aggregate runs at integration can scope the gate
with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which requires a matching
`Gate-Scope:` trailer so the commit never reads as fully gated:
[docs/contributing/gates.md](docs/contributing/gates.md). Never legitimate to
dodge a gate that is actually red.

Coverage and CRAP run outside `just check`: `just cover` fails under 90% lines,
then `just crap` fails above 30 and on any regression against
`crap-baseline.json`. Run `cover` first — `crap` refuses a stale lcov — and
`just crap-baseline` in the same commit as any deliberate complexity. Every one
of those numbers is argued in
[docs/contributing/coverage-crap.md](docs/contributing/coverage-crap.md); read it
before changing either recipe's flags.

`just nix-build` verifies `nix build` in a clean sandbox (no dev shell, no
sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; `just
release` calls it automatically.

## Signal Sweep (`just ask-refactoring`)

`contrib/ask-refactoring.sh` reads the *advisory* quality signals and files a
backlog task per finding. It is not a gate and never will be — a gate stops you
at the wall, the sweep names the wall you are walking toward. Run
`just ask-refactoring` to see what it would file today; the arms, the scoring and
the design rules are in
[docs/contributing/signal-sweep.md](docs/contributing/signal-sweep.md).

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) couples source files to the dependents that must
stay in sync (e.g. the CLI surface to `README.md`). After editing a source, update
the listed dependents, then `just outdatty-update` and commit `outdatty.lock`.

## Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates. Several
  files repeat it and all have to move together; the `typst-pkg-version` outdatty
  group enumerates them and fails the build otherwise. Trust the group, not a
  list here.
- `mindtape-cli`'s dependency on `mindtape` carries a version requirement (not
  path-only), because crates.io rejects path-only deps on publish; bump it with
  the workspace version.
- A release (`just release VERSION`) is gated: the tag must equal the workspace
  version, the working tree must be clean, and `CHANGELOG.md` may not have a
  non-empty `[Unreleased]` section (it moves into a dated `[VERSION]` section
  at release time). The recipe pushes `main` before the tag, so the docs site
  (`pages.yml`) and CI run on the release tree.

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
are tracked as task artifacts too; the ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config in
`.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs nothing.

## Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never mutate
the working tree; only explicit write commands do.

- Read (pure): `ls`, `check`, `show`, `export`, `config` queries — leave the tree
  byte-identical (enforced by test).
- Write (explicit): `add`, `recur`, `done`, `flip`, `rm`, `init`.

Recurrence never materializes as a side effect of a read. An instance is spawned
or refreshed only by an explicit `mt recur`; `mt done` closes a task and never
creates or advances anything.

"Byte-identical" is the *authored* tree — task files and config. A read may still
populate the git-ignored `.mindtape/cache/` scratch, which is derived state the
way `target/` is, not authored state.

## Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args` struct
  and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
