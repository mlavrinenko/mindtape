<!-- Generated from docs/src/contributing.typ by `just docs md`. Do not edit; edit the source. -->

# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the toolchain and the git hooks
just check           # run before every commit
```

Run `just` recipes, not raw `cargo`: the recipes carry the right flags and environment. Run `just` alone to list them. In the dev shell, `mt` is `target/mt-dev/debug/mt`. `just build-mt` builds it, and the worktrunk hooks in `.config/wt.toml` run it when `just fork` makes a worktree and after `just land` lands in a checkout. Between those builds, `mt` can be older than the code beside it.

This file holds the rules. The `docs/contributing/*.md` deep dives explain why and how they work, and `AGENTS.md` says what MindTape is.

## Code Style

The workspace `Cargo.toml` sets its clippy lints to `deny`, so the `clippy` gate fails on any violation. These limits shape the code most:

- No `unwrap()` or `expect()` outside tests. Use `?` with a `Diag` in the `mt` binary and `thiserror` in the library.
- No `todo!()`, `unimplemented!()`, `unreachable!()`, `unsafe` or wildcard imports.
- A function gets at most 70 lines, 5 arguments and a cognitive complexity of 20.
- A module’s path is its file path. An area is a folder with a `mod.rs` whose `//!` header lists its files, and the lint fails on `foo.rs` beside `foo/`. Use `#[path]` only for a test sibling. `just path-attr-check` refuses any other use.

## Error Handling

- Build every error the user sees as a `Diag` from `crates/mindtape-cli/src/diag/`, such as `Diag::usage("unknown field").value(name).hint(..)`. Its module docs hold the format. The text is a `&'static str`, so a value goes through `.value()`, which quotes it.
- Pick the kind, not the exit code: `Diag::usage` exits 2, `Diag::refused` and `Diag::failed` exit 1. `main` prints the error and exits with its code.
- Every command returns `Result<_, Diag>`, and the `mt` binary has no `anyhow`. clippy refuses `eprintln!`, `eprint!` and `std::io::stderr` outside `diag/`, so no line reaches stderr unrendered.
- If callers match on a library error type, derive `thiserror::Error` for it, as `RecurError`, `ConfigError` and `FilterError` do, and convert it in `diag/from_lib.rs` with a match on every variant.
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

In a delegated worktree, you can skip the hook’s freshness check with `MINDTAPE_GATE_SCOPE=<reason> git commit`. That’s safe there because `just land` runs `just check` on the branch’s final tree before it fast-forwards. Your message then needs a matching `Gate-Scope: <reason>` trailer. Never scope a commit to get past a failing gate.

## Worktrees

Work in parallel in a sibling worktree: `just fork` makes one and `just land` merges it back. Both wrap [worktrunk](https://worktrunk.dev) (`wt`, from the dev shell) with the flags this repo needs, because a bare `wt merge` squashes away each commit’s `Refs:` footer and commits a dirty tree.

```bash
just fork <slug>           # ../mindtape.<slug> on a new branch, its mt built
just fork <slug> --base=@  # the same, from the current commit, not main
just land                  # in the worktree: gate, wip check, fast-forward, remove
```

A worktree only gets commits, never your working tree, so commit first and branch before you flip a task: run `mt wip <stem>` inside the worktree. Don’t use `git stash` in a worktree: all worktrees share one stash list, so a `pop` can apply another worktree’s work. Make a WIP commit instead.

`.config/wt.toml` holds the hooks, so a clone with nix and direnv needs no worktrunk config of its own. They build the new worktree’s `mt`, run `just check` and `mt check --commits main..HEAD` before the fast-forward, which refuses a branch that names a still-`wip` task in `Refs:`, carry the gate records and the coverage lcov to the primary checkout, and rebuild the target’s `mt`.

## Commit Messages

- Use Conventional Commits, in the language of the earlier commits.
- End the message with a `Refs: <stem>` footer naming the task the commit works on. You can omit it only when the commit changes nothing but task files under `truth/tasks/`. Any other file needs it, `.mindtape/config.toml` included.
- File a task in its own `task:` commit, with no `Refs:`. The exception is a change to one source file that’s too small to plan: file it with `mt add -t tweak`, which files it done, in the commit that makes the change, and name it in `Refs:`.
- Stage a status flip with the work it claims: `mt wip` with the first work commit, `mt done` with the last. A commit holding only a flip is refused, and so is one naming a task that isn’t `wip` in the staged tree.
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
- `mindtape-cli` depends on `mindtape` by version and path, since crates.io refuses a path-only dependency. Bump it with the workspace version.
- `just release VERSION` is the whole release, and its tag push the only publish path: don’t run `cargo publish` or push `main` by hand. Keep the release’s entries under `[Unreleased]`; the recipe dates them. It runs [cargo-release](https://github.com/crate-ci/cargo-release) (`release.toml`), which refuses a dirty tree or a branch other than `main`, bumps the version, dates the entries, runs `.just/scripts/release-hook.sh` and commits. The hook refuses every other state a release has failed from. Then the recipe pushes `main`, so CI runs on the release tree, and tags only once that commit’s `ci.yml` run is green. `--dry-run` checks and pushes nothing. After a red CI, fix `main` and re-run: the bump and the dating stay done.
- A release whose workflow failed is finished on the same tag with `gh workflow run release.yml -f tag=vVERSION`, which skips each crate crates.io already has. A pushed tag never moves.

## Command Semantics

Read commands never change the authored tree. Only write commands do. [docs/contributing/command-semantics.md](docs/contributing/command-semantics.md) says which command is which.

## Documentation

Edit a doc’s Typst source, rewrite a file whole when you touch it, state each fact once, use checked examples, and write plain prose in the project’s voice. The rules are in [docs/contributing/documentation.md](docs/contributing/documentation.md).

## Extending MindTape

- A new CLI command is a module under `crates/mindtape-cli/src/cmd/` with an `Args` struct and a `run` function, wired into `cli/mod.rs`. Use `cmd/add/` as a model.
- A new domain capability goes in `crates/mindtape`, re-exported from its `lib.rs`. Keep task concepts out of `typst-harvest`, which is a general Typst crate other tools use.
- Record an architectural or tooling decision as a task under `truth/tasks/` tagged `decision`. There are no ADRs.
