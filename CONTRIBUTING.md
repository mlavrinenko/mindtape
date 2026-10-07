<!-- Generated from docs/src/contributing.typ by `just docs md`. Do not edit; edit the source. -->

# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the toolchain and the git hooks
just check           # run before every commit
```

Run `just` recipes, not raw `cargo`: the recipes carry the right flags and environment. Run `just` alone to list them. In the dev shell, `mt` is `target/mt-dev/debug/mt`. `just build-mt` builds it, `just fork` builds a new worktree’s, and `just integrate` rebuilds this checkout’s. Between those builds, `mt` can be older than the code beside it.

This file holds the rules. The `docs/contributing/*.md` deep dives explain why and how they work, and `AGENTS.md` says what MindTape is.

## Code Style

The workspace `Cargo.toml` sets its clippy lints to `deny`, so the `clippy` gate fails on any violation. These limits shape the code most:

- No `unwrap()` or `expect()` outside tests. Use `?` with `anyhow` or `thiserror`.
- No `todo!()`, `unimplemented!()`, `unreachable!()`, `unsafe` or wildcard imports.
- A function gets at most 70 lines, 5 arguments and a cognitive complexity of 20.
- A module’s path is its file path. An area is a folder with a `mod.rs` whose `//!` header lists its files, and the lint fails on `foo.rs` beside `foo/`. Use `#[path]` only for a test sibling. `just path-attr-check` refuses any other use.

## Error Handling

- Application code (the CLI) returns `anyhow::Result`.
- If callers match on a library error type, derive `thiserror::Error` for it, as `RecurError`, `ConfigError` and `FilterError` do.
- Keep `main.rs` a thin entry point and put the logic in library modules.

## Testing

- Put unit tests in `#[cfg(test)] mod tests` beside the code, or in a sibling `_tests.rs` loaded with `#[path]`.
- Put tests that run the `mt` binary (`assert_cmd::cargo_bin`) in `tests/`. Cargo sets `CARGO_BIN_EXE_mt` only for integration tests.
- When a file nears its linecop cap, run `just fix-check`. It moves the file’s inline tests into a `_tests.rs` sibling (`tests.rs` for a `mod.rs`) with [ejectest](https://github.com/mlavrinenko/ejectest).
- Run the suite with `just test` and count it with `just count-tests`.
- If your code runs a process or joins paths, it also runs on the Windows CI job. See [docs/contributing/windows.md](docs/contributing/windows.md).
- Before a PR, run `just mutants-diff` and triage every survivor. A survivor is code the tests run but don’t check. A run over 100 mutants, about 20 minutes, needs the owner’s approval, so an agent stops and asks instead of setting `MINDTAPE_MUTANTS_APPROVED=1` itself. The triage classes are in [docs/contributing/mutation-testing.md](docs/contributing/mutation-testing.md).

## Quality Gates (run by `just check`)

The pre-commit hook refuses your commit unless every gate passed against the current tree. To see the gates and their state:

```bash
just --list --list-submodules  # every gate under [gate], with what it checks
mmz --status --tag gate        # which gates are fresh, stale or failed
```

[docs/contributing/gates.md](docs/contributing/gates.md) covers how `just check` orders the gates, how it skips unchanged work, and the formatter path lists. When you add a TOML file, a Typst directory or a justfile module, add it to its `*_FMT_PATHS` list in the `Justfile`. `just fmt` and `just fmt-check` both read those lists.

If you add complexity on purpose, run `just crap-baseline` in the same commit. The thresholds and their reasons are in [docs/contributing/coverage-crap.md](docs/contributing/coverage-crap.md).

If you touch `flake.nix` or `.cargo/config.toml`, run `just nix-build`. It builds the flake in a clean sandbox, and `just release` runs it too.

In a delegated worktree, you can skip the hook’s freshness check with `MINDTAPE_GATE_SCOPE=<reason> git commit`. That’s safe there because `just integrate` refuses the branch until every gate passes on its final tree. Your message then needs a matching `Gate-Scope: <reason>` trailer. Never scope a commit to get past a failing gate.

## Worktrees

To work on something in parallel, run `just fork <slug> [base]`. You get a worktree at `../mindtape-<slug>` on a new branch from `base` (`HEAD` by default). Commit first: `git worktree add` only copies commits, so `fork` refuses a dirty tree. For the same reason, fork before you flip a task, and run `mt wip <stem>` inside the worktree. Don’t use `git stash` in a worktree: all worktrees share one stash list, so a `pop` can apply another fork’s work. Make a WIP commit instead.

When the branch is ready, run `just integrate <branch>` from this checkout. It rebuilds the worktree’s `mt` from the branch tip, so a branch rebased over a config change is judged by its own config. It refuses the branch if it names a still-`wip` task in `Refs:`, or if any gate hasn’t passed against the worktree’s current files. Then it does four things:

- Fast-forwards this checkout.
- Copies the worktree’s `.mmz/cache/*.yaml` gate records here, so your next commit isn’t refused for a stale gate (mmz still judges each record, and one whose inputs differ stays stale).
- Carries `target/coverage/lcov.info` over when it’s fresh and newer than this checkout’s, so `just crap` doesn’t wait on a `just cover`.
- Removes the worktree.

`.just/scripts/fork.sh` and `.just/scripts/integrate.sh` explain both.

## Commit Messages

- Use Conventional Commits, in the language of the earlier commits.
- End the message with a `Refs: <stem>` footer naming the task the commit works on. You can omit it only when the commit changes nothing but task files under `truth/tasks/`. Any other file needs it, `.mindtape/config.toml` included.
- File a task in its own `task:` commit, with no `Refs:`. The exception is a change to one source file that’s too small to plan: file it with `mt add -t tweak`, which files it done, in the commit that makes the change, and name it in `Refs:`.
- Commit a status flip with the work it claims: `mt wip` in the first work commit, `mt done` in the last. Leave the flip uncommitted until then, because a commit with a flip and no other change is refused. Stage the `mt wip` flip with that first commit, because a commit naming a task that isn’t `wip` in the staged tree is refused too.
- If a commit changes product code (`crates/**` and `lib/**`, tests excepted), it also adds an entry under `[Unreleased]` in `CHANGELOG.md`. If there’s nothing to announce, add a `Changelog: none` trailer instead. The commit-msg hook and CI refuse the commit otherwise, whatever its type.
- A task tagged `decision` has no code of its own. It can close in a commit that changes only task files, with a `Refs:` footer naming it.

[docs/contributing/commit-messages.md](docs/contributing/commit-messages.md) says what checks these rules, and why `Refs:` names a stem.

## Findings and the Signal Sweep

When you notice a real problem outside your task, file it with `mt add --template finding` (tag `found`) before you report, and don’t fix it in passing. A report reaches one reader once, and a task stays in the backlog.

`just ask-refactoring` reads the quality signals that no gate enforces and prints one task per finding. Add `--write` to file them. The arms and their scoring are in [docs/contributing/signal-sweep.md](docs/contributing/signal-sweep.md).

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) lists, for each source file, the files that restate it.

- After you edit a source, update its dependents, run `just outdatty-update <group>` and commit `outdatty.lock`.
- After you edit only a dependent, record that one file in the same commit with `just outdatty-update <group> <path>`.
- A recorded hash says someone reviewed the dependents. So run `just outdatty-update-all` only after you’ve reviewed every group.
- A new file that a glob dependent matches needs a hash of its own. `just outdatty-check` fails until the lock holds it.

A rebase or merge joins two branches’ `outdatty.lock` key by key through `.just/scripts/outdatty-lock-merge.sh`, which the dev shell registers. It stops only where both branches changed one hash and the merged file matches neither.

## Version Management

- There’s no backward compatibility before the first public release. A rename lands whole, callers and configs included, with no alias or migration.
- The workspace crates take their version from `[workspace.package] version` in `Cargo.toml`.
- The Typst library’s version is `version` in `lib/typst.toml`, independent of the crates. The `typst-pkg-version` outdatty group lists every file that repeats it.
- `mindtape-cli` depends on `mindtape` by version as well as by path, because crates.io refuses a path-only dependency. Bump it with the workspace version.
- `just release VERSION` refuses a `VERSION` that isn’t plain semver or differs from the workspace version. It also refuses a dirty tree, and a `CHANGELOG.md` with content under `[Unreleased]`, so move that content into a dated section first. It pushes `main` before the tag, so the docs site and CI run on the release tree.

## Command Semantics

Read commands never change the authored tree. Only write commands do. [docs/contributing/command-semantics.md](docs/contributing/command-semantics.md) says which command is which.

## Documentation

Edit a doc’s Typst source, rewrite a file whole when you touch it, state each fact once, use checked examples, and write plain prose in the project’s voice. The rules are in [docs/contributing/documentation.md](docs/contributing/documentation.md).

## Extending MindTape

- A new CLI command is a module under `crates/mindtape-cli/src/cmd/` with an `Args` struct and a `run` function, wired into `cli/mod.rs`. Use `cmd/add/` as a model.
- A new domain capability goes in `crates/mindtape`, re-exported from its `lib.rs`. Keep task concepts out of `typst-harvest`, which is a general Typst crate other tools use.
- Record an architectural or tooling decision as a task under `truth/tasks/` tagged `decision`. There are no ADRs.
