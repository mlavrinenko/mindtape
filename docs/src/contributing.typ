// docs/src/contributing.typ renders the repo-root CONTRIBUTING.md through
// docs/generate-md.sh (typlite; docs/src/lib.typ lists the constraints).
//
// Every agent session in this repo loads CONTRIBUTING.md, so it holds the
// rules and nothing else. A rule's reasons and mechanics go in a
// docs/contributing/*.md deep dive, and how a gate works goes in its recipe
// comment or script header. .linecop.yaml caps the rendered file.
//
// The gate table reads the gates `just check` runs, in order, from
// www/generated/gates.json, and each gate's clause from
// www/utils/gate-notes.typ. What a gate runs is in its recipe or script.
#import "lib.typ": doc-title, fact, just
#import "/www/utils/gate-notes.typ": gate-notes

#metadata((
  output: "CONTRIBUTING.md",
)) <mindtape-md>

#doc-title[Contributing to MindTape]

= Quick Start

// docs:hand-typed a clone-and-enter sequence that no fixture can run
```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the toolchain and the git hooks
just check           # run before every commit
```

Use `just` recipes rather than raw `cargo`, because the recipes carry the
right flags and environment. `just` with no arguments lists them. In the dev
shell, `mt` is `target/mt-dev/debug/mt`. #just.build-mt() builds it,
#just.fork() builds a new worktree's, and #just.integrate() rebuilds this
checkout's. Between those builds, `mt` can be older than the code beside it.

This file holds the rules for how work is done here. The
`docs/contributing/*.md` deep dives give the reasons and mechanics behind
them. `AGENTS.md` describes what MindTape is.

= Code Style

The workspace `Cargo.toml` sets its clippy lints to `deny`, so the `clippy`
gate fails on any violation. The limits that shape code most:

- No `unwrap()` or `expect()` outside tests. Use `?` with `anyhow` or
  `thiserror`.
- No `todo!()`, `unimplemented!()`, `unreachable!()`, `unsafe` or wildcard
  imports.
- A function has at most 70 lines and 5 arguments, and its cognitive
  complexity is at most 20.

= Error Handling

- Application code (the CLI) returns `anyhow::Result`.
- A library error type that callers match on derives `thiserror::Error`, as
  `RecurError`, `ConfigError` and `FilterError` do.
- `main.rs` stays a thin entry point. Logic lives in library modules.

= Testing

- Put unit tests in `#[cfg(test)] mod tests` beside the code, or in a sibling
  `_tests.rs` loaded with `#[path]`.
- Put tests that run the `mt` binary (`assert_cmd::cargo_bin`) in `tests/`.
  Cargo sets `CARGO_BIN_EXE_mt` for integration tests only.
- When a file nears its linecop cap, #just.fix-check() moves its inline tests
  into a `_tests.rs` sibling with
  #link("https://github.com/mlavrinenko/ejectest")[ejectest].
- Run #just.test(). Count tests with #just.count-tests().
- Code that runs a process or joins paths also runs on the Windows CI job,
  described in
  #link("docs/contributing/windows.md")[docs/contributing/windows.md].
- Run #just.mutants-diff() before a PR and triage every survivor, because a
  survivor is code the tests run but do not check. The triage classes are in
  #link("docs/contributing/mutation-testing.md")[docs/contributing/mutation-testing.md].

= Quality Gates (run by #just.check())

The pre-commit hook refuses a commit unless every gate below passed against
the current tree.

#let gate-rows = fact("gates").at("gates")
#table(
  columns: 2,
  table.header([Gate], [Without it]),
  ..gate-rows.map(g => (raw(g.name), gate-notes.at(g.name))).flatten(),
)

How the gates run and skip unchanged work, and the formatter path lists, are
in #link("docs/contributing/gates.md")[docs/contributing/gates.md]. A new TOML
file, Typst directory or justfile module joins its `*_FMT_PATHS` list in the
`Justfile`, which #just.fmt() and #just.fmt-check() both read.

#just.check-full() adds #just.cover() and #just.crap() to #just.check(). Run
it before a PR. When you add complexity on purpose, run #just.crap-baseline()
in the same commit. The thresholds and their reasons are in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md].

Run #just.nix-build() when you touch `flake.nix` or `.cargo/config.toml`. It
builds the flake in a clean sandbox, and #just.release() runs it too.

A commit in a delegated worktree may skip the hook's freshness check with
`MINDTAPE_GATE_SCOPE=<reason> git commit`, because #just.integrate() refuses
the branch until every gate passes on its final tree. The message then needs
a matching `Gate-Scope: <reason>` trailer. Never scope a commit to get past a
failing gate.

= Worktrees

#just.fork("<slug>", "<base>") creates the worktree `../mindtape-<slug>` on a
new branch off `<base>` (default `HEAD`). It refuses when this checkout has
uncommitted changes, because `git worktree add` copies commits only.
#just.integrate("<branch>") refuses a branch that names a still-`wip` task in
`Refs:`, and refuses unless every gate passed against the worktree's current
files. It then fast-forwards this checkout and removes the
worktree. `.just/scripts/fork.sh` and `.just/scripts/integrate.sh` explain
both.

Fork before you flip a task, because the uncommitted flip would make `fork`
refuse. Run `mt wip <stem>` inside the worktree instead.

= Commit Messages

- Use Conventional Commits in the language of prior commits.
- End the message with a `Refs: <stem>` footer naming the task the commit
  works on. Only a commit that changes nothing but task files under
  `truth/tasks/` may omit it. A change to any other file needs it,
  `.mindtape/config.toml` included.
- File a task in its own `task:` commit with no `Refs:`. A change to one
  source file too small to plan is the exception: file it with
  `mt add -t tweak`, which files it done, in the commit that makes the
  change, and name it in `Refs:`.
- Commit a status flip together with the work it claims. `mt wip` goes in
  the first work commit and `mt done` in the last. Leave the flip uncommitted
  until then, because a commit with a flip and no other change is refused.
  Stage the `mt wip` flip with that commit: a commit naming a task that is not
  `wip` in the staged tree is refused too.
- A task tagged `decision` has no code of its own. It may close in a commit
  that changes only task files, with a `Refs:` footer naming it.

What checks these rules, and why `Refs:` names a stem, is in
#link("docs/contributing/commit-messages.md")[docs/contributing/commit-messages.md].

= Findings and the Signal Sweep

When you notice a real problem outside your task, file it with
`mt add --template finding` (tag `found`) before you report. Do not fix it in
passing. A report reaches one reader once, and a task stays in the backlog.

#just.ask-refactoring() reads the quality signals that no gate enforces and
prints one task per finding. Add `--write` to file them. The arms and their
scoring are in
#link("docs/contributing/signal-sweep.md")[docs/contributing/signal-sweep.md].

= Dependency Drift (outdatty)

#link("outdatty.yaml")[`outdatty.yaml`] lists, for each source file, the
files that restate it. After you edit a source, update its dependents, run
#just.outdatty-update("<group>") and commit `outdatty.lock`. A recorded hash
says someone reviewed the dependents, so run #just.outdatty-update-all() only
after reviewing every group. After you edit only a dependent, record that file
alone in the same commit with #just.outdatty-update("<group> <path>").

= Version Management

- There is no backward compatibility before the first public release. A
  rename lands whole, callers and configs included, with no alias or
  migration.
- The workspace crates take their version from `[workspace.package] version`
  in `Cargo.toml`.
- The Typst library's version is `version` in `lib/typst.toml`, independent of
  the crates. The `typst-pkg-version` outdatty group lists every file that
  repeats it.
- `mindtape-cli` depends on `mindtape` by version as well as by path, because
  crates.io refuses a path-only dependency. Bump it with the workspace
  version.
- #just.release("VERSION") refuses a `VERSION` that is not plain semver or
  differs from the workspace version. It also refuses a dirty tree and a
  `CHANGELOG.md` whose `[Unreleased]` section has content, so move that
  content into a dated section first. It pushes `main` before the tag, so the
  docs site and CI run on the release tree.

= Command Semantics

Read commands never change the authored tree. Only write commands do. Which
command is which is in
#link("docs/contributing/command-semantics.md")[docs/contributing/command-semantics.md].

= Documentation

Edit a doc's Typst source, rewrite a touched file whole, state each fact once,
use checked examples and write plain prose. The rules are in
#link("docs/contributing/documentation.md")[docs/contributing/documentation.md].

= Extending MindTape

- A new CLI command is `crates/mindtape-cli/src/<name>.rs` with an `Args`
  struct and a `run` function, wired into `cli.rs`. `ls.rs` is a model.
- A new domain capability goes in `crates/mindtape` and is re-exported from
  its `lib.rs`. Keep task concepts out of `typst-harvest`, a general Typst
  crate other tools use.
- An architectural or tooling decision is a task under `truth/tasks/` tagged
  `decision`. There are no ADRs.
