// docs/src/contributing.typ renders the repo-root CONTRIBUTING.md through
// docs/generate-md.sh (typlite; docs/src/lib.typ lists the constraints).
//
// Every agent session loads CONTRIBUTING.md, so it holds the rules and nothing
// else. A rule's reasons and mechanics go in a docs/contributing/*.md deep
// dive or in a recipe comment or script header. No list of the gates lives
// here: the reader asks `just`, so the list can't drift from the Justfile.
#import "lib.typ": doc-title, just, mindtape

#metadata((
  output: "CONTRIBUTING.md",
)) <mindtape-md>

#doc-title[Contributing to #mindtape]

= Quick Start

// docs:hand-typed a clone-and-enter sequence that no fixture can run
```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the toolchain and the git hooks
just check           # run before every commit
```

Run `just`, not raw `cargo`: it has the right flags and environment.
#just.build-mt() builds `mt` at `target/mt-dev/debug/mt`; the worktrunk hooks
rebuild it on #just.fork() and #just.land(). Between builds, `mt` can be older
than the code beside it.

= Code Style

The workspace `Cargo.toml` denies every clippy violation. These limits affect
the code you write:

- No `unwrap()` or `expect()` outside tests. Use `?` with a `Diag` in the
  `mt` binary and `thiserror` in the library.
- No `todo!()`, `unimplemented!()`, `unreachable!()`, `unsafe` or wildcard
  imports.
- A function gets at most 70 lines, 5 arguments and a cognitive complexity
  of 20.
- A module's path is its file path. An area is a folder with a `mod.rs` whose
  `//!` header lists its files, and the lint fails on `foo.rs` beside `foo/`.
  Use `#[path]` only for a test sibling. #just.lint-path-attr() refuses any
  other use.

= Error Handling

- Build every error as a `Diag` from `crates/mindtape-cli/src/diag/`, such as
  `Diag::usage("unknown field").value(name).hint(..)`. Its module docs hold
  the format. The text is a `&'static str`, so a value goes through `.value()`,
  which quotes it.
- Pick the kind, not the exit code: `Diag::usage` exits 2, `Diag::refused`
  and `Diag::failed` exit 1. `main` prints the error and exits with its code.
- Every command returns `Result<_, Diag>`; the `mt` binary has no `anyhow`.
  clippy refuses `eprintln!`, `eprint!` and `std::io::stderr` outside
  `diag/`, so no line reaches stderr raw.
- If callers match on a library error type, derive `thiserror::Error` for it,
  as `RecurError`, `ConfigError` and `FilterError` do, and convert it in
  `diag/from_lib.rs` matching every variant.
- Say a count with `mindtape::count(n, "task")`, which prints `1 task` or
  `12 tasks`. #just.lint-plural() refuses `task(s)` in a string outside
  tests.
- Keep `main.rs` thin and put the logic in library modules.

= Testing

- Put unit tests beside the code, or in a sibling `_tests.rs` loaded with
  `#[path]`.
- Put tests that run the `mt` binary (`assert_cmd::cargo_bin`) in `tests/it/`,
  which each crate links as one binary: declare a new file in
  `tests/it/main.rs`. Only they get `CARGO_BIN_EXE_mt`.
- When a file nears its linecop cap, run #just.fix-check(). It moves the
  file's inline tests into a `_tests.rs` sibling (`tests.rs` for a `mod.rs`)
  with #link("https://github.com/mlavrinenko/ejectest")[ejectest].
- Run the suite with #just.test() and count it with #just.count-tests().
- If your code runs a process or joins paths, the Windows CI job runs it too.
  See #link("docs/contributing/windows.md")[docs/contributing/windows.md].
- Before a PR, run #just.mutants-diff() and triage every survivor. A run over
  100 mutants, about 20 minutes, needs the owner's approval. Stop and ask
  instead of setting `MINDTAPE_MUTANTS_APPROVED=1`. The triage classes are in
  #link("docs/contributing/mutation-testing.md")[docs/contributing/mutation-testing.md].

= Quality Gates (run by #just.check())

The pre-commit hook refuses a commit unless every gate passed against the
current tree. To see the gates:

// docs:hand-typed the commands that list the gates, run against the live Justfile
```bash
just --list --list-submodules  # every gate under [gate], with what it checks
mmz --status --tag gate        # which gates are fresh, stale or failed
```

#link("docs/contributing/gates.md")[docs/contributing/gates.md] covers how
#just.check() orders the gates and skips unchanged work. It also describes the
formatter path lists. When you add a TOML file or a Typst directory, add it to
its `*_FMT_PATHS` list in the `Justfile`. #just.fmt() and #just.fmt-check()
both read those lists.

If you add a function or complexity, run #just.crap-baseline() in the same
commit. The thresholds and their reasons are in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md].

If you touch `flake.nix`, `nix/dev/` or `.cargo/config.toml`, run
#just.nix-build(), and for the flake #just.smoke-flake() too.

In a delegated worktree, skip the hook's freshness check with
`MINDTAPE_GATE_SCOPE=<reason> git commit` and a matching
`Gate-Scope: <reason>` footer. That's safe because #just.land() runs
#just.check() on the branch's final tree before it fast-forwards. Never scope
a commit to get past a failing gate.

= Worktrees

Work in parallel in a sibling worktree: #just.fork() makes one and #just.land()
merges it back. Both wrap #link("https://worktrunk.dev")[worktrunk] (`wt`, from
the dev shell) with this repo's flags, because a bare `wt merge` squashes away
each commit's `Refs:` footer and commits a dirty tree.

// docs:hand-typed a worktree session that no fixture can run
```bash
just fork <slug>           # ../mindtape-<slug> on a new branch, its mt built
just fork <slug> --base=@  # the same, from the current commit, not main
just land                  # in the worktree: gate, wip check, fast-forward, remove
```

A worktree gets commits, not your working tree. Commit before you flip a task
and run `mt wip <stem>` there. Don't `git stash`: all worktrees share one
stash list, so a `pop` can apply another's work. Make a WIP commit instead.

`.config/wt.toml` holds the hooks, so a clone with nix and direnv needs no
worktrunk config of its own. Before the fast-forward they run #just.check() and
`mt check --commits main..HEAD`, which refuses a branch that names a
still-`wip` task in `Refs:`.

#just.install-all() stamps each installed skill with `.mindtape-source-hash`,
a hash of its `skills/<name>` source. #just.check() fails when a stamp differs
or is missing, and stays quiet when no copy exists.

= Commit Messages

- Use Conventional Commits, in the language of the earlier commits.
- End the message with a `Refs: <stem>` footer naming the task the commit
  works on. You can omit it only when the commit changes nothing but task
  files under `truth/tasks/`. Any other file needs it, `.mindtape/config.toml`
  too. A merge commit needs none. `mt check` skips merges by default.
- File a task in its own `task:` commit, with no `Refs:`. The exception is a
  change to one source file that's too small to plan. Use `mt add -t tweak` in
  the commit that makes the change, so the task is created as done. Name the
  task in `Refs:`.
- Stage a status flip with the work it claims: `mt wip` with the first work
  commit, `mt done` with the last. A commit holding only a flip is refused,
  and so is one naming a task that isn't `wip` in the staged tree.
- If a commit changes product code (`crates/**` and `lib/**`, tests
  excepted), it also adds an entry under `[Unreleased]` in `CHANGELOG.md`. If
  there's nothing to announce, add a `Changelog: none` trailer instead. The
  commit-msg hook and CI refuse the commit otherwise, whatever its type.
- A `decision` task or a milestone has no code of its own. It can close in a
  commit that changes only task files, with a `Refs:` footer naming it.

#link(
  "docs/contributing/commit-messages.md",
)[docs/contributing/commit-messages.md]
says what checks these rules, and why `Refs:` names a stem.

= Findings and the Signal Sweep

When you see a real problem outside your task, file it with
`mt add --template finding` (tag `found`) before you report, and don't fix it
in passing. The template asks for a `surfaced-by` link to the task where you
found it.

#just.ask-refactoring() reads the quality signals that no gate enforces and
prints one task per finding. Add `--write` to file them. The arms and their
scoring are in
#link("docs/contributing/signal-sweep.md")[docs/contributing/signal-sweep.md].

= Dependency Drift (outdatty)

#link(".config/outdatty.yaml")[`.config/outdatty.yaml`] lists, for each source
file, the files that restate it.

- After you edit a source, update its dependents, run
  #just.outdatty-update("<group>") and commit `.config/outdatty.lock`. If
  every dependent still matches and the commit edits none, add a
  `Docs-Unaffected: <group>` footer, one line per group you read. The
  commit-msg hook refuses a commit without it and lists the files to read.
- After you edit only a dependent, record that one file in the same commit
  with #just.outdatty-update("<group> <path>").
- A recorded hash means you reviewed the dependents, so run
  #just.outdatty-update-all() only after reviewing every group.
- A new file that a glob dependent matches needs a hash of its own.
  #just.outdatty-check() fails until the lock holds it.

A rebase or merge joins the two branches' locks key by key through
`outdatty merge-lock`. When both branches edited one source and re-recorded
it, its hash comes out as zeros. #just.outdatty-check() reports that group
until you review it and commit the re-record with a `Docs-Unaffected: <group>`
footer. A group one branch dropped stays dropped, so a split or rename leaves
no orphan hashes. #just.outdatty-check() fails on one that got in anyway.
`.config/jscpd-baseline.json` merges fingerprint by fingerprint through
`jscpd --merge-baseline`. `.config/crap-baseline.json` merges function by
function through `.just/scripts/crap-baseline-merge.sh`, which keeps the
stricter score when both sides re-scored a function.
#just.crap-check() decides whether the merged tree passes. The dev shell
registers all three merge drivers: `outdatty`, `jscpd-baseline` and
`crap-baseline`.

= Version Management

- There's no backward compatibility before the first public release. A
  rename updates every caller and config at once, with no alias or migration.
- The Typst library's version is `version` in `lib/typst.toml`, independent of
  the crates. The `typst-pkg-version` outdatty group lists every file that
  repeats it. #link("docs/contributing/release.md")[Releases] archive it
  under `crates/mindtape/src/loader/preludes/`.
- `mindtape-cli` depends on `mindtape` by version and path, since crates.io
  refuses a path-only dependency. Bump that dependency with the workspace
  version in `[workspace.package]`.
- #just.release("VERSION") runs every release step, and its tag push is the
  only publish path: don't run `cargo publish` or push `main` by hand. Keep the
  release's entries under `[Unreleased]`. The recipe dates them.
  #link("docs/contributing/release.md")[docs/contributing/release.md] says what
  it runs and how to finish a failed release.

= Command Semantics

Read commands never change the authored tree. Only write commands do.
#link(
  "docs/contributing/command-semantics.md",
)[docs/contributing/command-semantics.md]
says which command is which.

= Documentation

Edit a doc's Typst source, rewrite a file whole when you touch it, state each
fact once, use checked examples, and write plain prose in the project's
voice. The rules are in
#link("docs/contributing/documentation.md")[docs/contributing/documentation.md].

= Extending #mindtape

- A new CLI command is a module under `crates/mindtape-cli/src/cmd/` with an
  `Args` struct and a `run` function, wired into `cli/mod.rs`. Use `cmd/add/`
  as a model.
- A new domain capability goes in `crates/mindtape`, re-exported from its
  `lib.rs`. Keep task concepts out of `typst-harvest`, which is a general
  Typst crate other tools use.
- Record an architectural or tooling decision as a `decision` task under
  `truth/tasks/`. There are no ADRs.
