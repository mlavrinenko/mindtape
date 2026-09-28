// docs/src/contributing.typ — source for the repo-root CONTRIBUTING.md.
// Rendered by `docs/generate-md.sh` via typlite; see docs/src/lib.typ.
//
// The Quality Gates table is derived. `www/generated/gates.json` carries the
// sub-gates `just check` declares, in order, each with its doc comment and a
// `command` read back out of the recipe's own body (www/gates.jq argues it),
// so the hand-quoted clippy invocation cannot drift again and a gate dropped
// from `just check` drops off the table on its own. The Notes column has no
// machine source: it stays hand-written in `www/utils/gate-notes.typ`, the
// `www/utils/*-notes.typ` house pattern, keyed by gate name — a gate missing
// there fails `just docs::md` naming it. gates.json also carries the
// Justfile's `_FMT_PATHS` assignments, selected by that suffix so a future
// formatter's list lands without an edit; the block below the table LOOPS
// that dict rather than naming each key, because a hand-listed `.at("...")`
// per key once let a fifth list land in the artifact and never render, and
// nothing gates assignments the way check-doc-facts gates gate names.
#import "lib.typ": doc-title, fact, just
#import "/www/utils/gate-notes.typ": gate-notes

#metadata((
  output: "CONTRIBUTING.md",
)) <mindtape-md>

#doc-title[Contributing to MindTape]

= Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the full toolchain (installs the
                     # pre-commit hook: asserts the gate is fresh, checks
                     # task files)
just check           # run before every commit
```

Use `just` recipes rather than raw `cargo`: they carry the right flags and
environment, and `just` with no arguments lists them. `flake.nix` owns the tools.
In the dev shell `mt` is `target/mt-dev/debug/mt` on `PATH`, not a wrapper that
rebuilds: #just.build-mt() builds it, #just.fork() builds a fresh worktree's,
and #just.integrate() rebuilds this checkout's after the fast-forward. An edit
between those events leaves `mt` older than the code beside it until the next
build; entering the shell with no binary there names #just.build-mt().

This is the single source for how work is done here; a process rule belongs
here and nowhere else. Deep dives sit in `docs/contributing/`, and `AGENTS.md`
covers only what MindTape _is_.

= Code Style

All clippy lints are `deny` (workspace `Cargo.toml`); the project will not
build with violations. Key restrictions:

- No `unwrap()`/`expect()` outside tests — use `?` with `anyhow`/`thiserror`
- No `todo!()`/`unimplemented!()`/`unreachable!()`/`unsafe`/wildcard imports
- Functions: ≤70 lines, ≤5 args, cognitive complexity ≤20

= Error Handling

- `anyhow::Result` for application code (the CLI); `thiserror::Error` for
  library error types callers match on (`HarvestError`, `ConfigError`, `FilterError`)
- Keep `main.rs` a thin entry point; logic lives in library modules

= Testing

- Unit tests inline in `#[cfg(test)] mod tests` (or a sibling `_tests.rs` via
  `#[path]`) next to the code they exercise.
- Tests that shell out to the `mt` binary (`assert_cmd::cargo_bin`) belong in
  `tests/`, not inline: Cargo exports `CARGO_BIN_EXE_mt` before integration
  tests, a guarantee `--lib` unit tests lack.
- As a file nears the linecop limit, #just.fix-check() ejects its inline tests
  into a `_tests.rs` sibling with
  #link("https://github.com/mlavrinenko/ejectest")[ejectest].
- Run #just.test()\; count with #just.count-tests().
- Anything that shells out or joins paths needs a thought for the Windows CI
  job. What runs there, the #just.check-windows() cross-check and the
  Unix-only pty tests:
  #link("docs/contributing/windows.md")[docs/contributing/windows.md].
- Mutation testing (`cargo-mutants`) proves whether tests pin behaviour, not
  just execute it. Run #just.mutants-diff() before a PR and triage every survivor:
  #link("docs/contributing/mutation-testing.md")[docs/contributing/mutation-testing.md].

= Quality Gates (run by #just.check())

#let gate-rows = fact("gates").at("gates")
#let gate-assignments = fact("gates").at("assignments")
#table(
  columns: 3,
  table.header([Gate], [Command], [Notes]),
  ..gate-rows
    .map(g => (
      raw(g.name),
      if g.command == none { emph(g.doc) } else { raw(g.command) },
      gate-notes.at(g.name),
    ))
    .flatten(),
)

`fmt-check`'s non-Rust path lists:

#for (name, value) in gate-assignments [
  - #raw(name): #value.split(" ").map(raw).join(", ")
]

Every arm runs through `mmz`, so a re-run skips each one whose declared inputs
are unchanged and the pre-commit hook can assert the whole set fresh without
re-running it. That memoization and the locked formatter gates are in
#link("docs/contributing/gates.md")[docs/contributing/gates.md]; the gated
example projects, in
#link("docs/contributing/example-projects.md")[docs/contributing/example-projects.md].

#just.check-full() is #just.check() plus #just.cover() and #just.crap(): always
correct about coverage and CRAP, at the price of an instrumented rebuild and a
full suite pass. Run it before opening a PR, plain #just.check() while iterating.

#just.fork("<slug>", "<base>") stands one of those worktrees up and
#just.integrate("<branch>") takes it back down. `fork` branches off `<base>` (default
`HEAD`) at `../mindtape-<slug>`, `direnv allow`-ed, refusing a dirty `HEAD` —
`git worktree add` copies commits, never the working tree — and reporting, without
refusing, a divergent base. Fork first, before flipping anything: `main` is
still clean at that point, so the worktree never needs a status flip
committed there just to clear the refusal — `mt wip <stem>` runs inside the
new worktree instead, and its flip lands in the worktree's first work commit
(see Commit Messages). `integrate` refuses unless that worktree reports
`mmz --is-fresh --tag gate`, naming the stale arm, then fast-forwards and removes it: a
fast-forward writes no commit, so the pre-commit hook never fires on the merge.
`fork` also builds the new worktree's `mt`, once, after loading its shell, and
`integrate` rebuilds this checkout's after the fast-forward, so the binary each
`PATH` serves matches the tree it sits in. What each refuses and deliberately
leaves undone — no seeded `target/`, no re-run gate, no waiver flag — is argued in
`.just/scripts/fork.sh` and `.just/scripts/integrate.sh`.

A delegated worktree whose full aggregate runs at integration can scope the
gate with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which demands a matching
`Gate-Scope:` trailer; dodging a red gate is never legitimate. When scoping
is, and what the trailer buys, are in #link("docs/contributing/gates.md")[docs/contributing/gates.md].

Coverage and CRAP run outside #just.check(): run #just.cover() first (#just.crap() refuses a
stale lcov), and #just.crap-baseline() in the same commit as any deliberate complexity. The
thresholds and every argument behind them, to read before changing either recipe's flags, are
in #link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md]; the
baseline's own paths and guards, in
#link("docs/contributing/crap-baseline.md")[docs/contributing/crap-baseline.md].

#just.nix-build() verifies `nix build` in a clean sandbox (no dev shell, no
sccache). Run it when touching `flake.nix` or `.cargo/config.toml`;
#just.release() calls it automatically.

= Signal Sweep (#just.ask-refactoring())

The sweep reads the _advisory_ quality signals and files a backlog task per
finding. It is not a gate: a gate stops you at the wall, the sweep names the
wall you are walking toward. Run it dry (#just.ask-refactoring()) to see what
it would file; the arms, the scoring and the design rules are in
#link("docs/contributing/signal-sweep.md")[docs/contributing/signal-sweep.md].

A delegated agent that notices something real outside its own task's scope
files it the same way — `mt add --template finding`, tagged `found` — before
writing the closing report, never in prose alone: the report reaches one
reader once, a filed task does not.

= Dependency Drift (outdatty)

#link("outdatty.yaml")[`outdatty.yaml`] couples source files to the
dependents that must stay in sync (e.g. the CLI surface to `README.md`).
After editing a source, update the listed dependents, re-record the group you
touched with #just.outdatty-update("<group>") and commit `outdatty.lock`. A
recorded hash is a review watermark, so blanket re-recording —
#just.outdatty-update-all() — claims a review that never happened. It stays
the explicit choice, never the default.

Edit a dependent alone, and record just that file in the same commit —
#just.outdatty-update("<group> <path>") leaves the group's other pending
dependents pending. Only a source change is a trigger, so the hash would
otherwise ride along on an unrelated commit; the reasoning and the
pending-groups one-liner are in `outdatty.yaml`'s header.

= Version Management

- No backward compatibility before the first public release: a rename lands
  whole, callers and configs included, with no alias or migration path left.
- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates. Several files
  repeat it and all have to move together; the `typst-pkg-version` outdatty group
  enumerates them and fails the build otherwise. Trust the group, not a list here.
- `mindtape-cli`'s dependency on `mindtape` carries a version requirement
  (not path-only), because crates.io rejects path-only deps on publish; bump
  it with the workspace version.
- A release (#just.release("VERSION")) is gated twice: an `[arg(...)]` pattern rejects a
  `v`-prefixed or non-semver `VERSION`, then the body requires the tag to equal the
  workspace version, a clean tree, and no non-empty `[Unreleased]` section in
  `CHANGELOG.md` (it moves into a dated section at release time). `main` is pushed
  before the tag, so the docs site (`pages.yml`) and CI run on the release tree.

= Commit Messages

Conventional Commits in the language of prior commits, with a `Refs: <stem>`
footer naming the driving task; a commit that only files or edits task
artifacts carries none, and a status flip rides with the work it claims. The
rule, the hooks that check it and the two setups are in
#link("docs/contributing/commit-messages.md")[docs/contributing/commit-messages.md].

= Command Semantics (read/write separation)

Read commands never mutate the authored tree; only explicit write commands do.
Which verb is which, the per-command read test, and the CLI command list it is
read from are in
#link("docs/contributing/command-semantics.md")[docs/contributing/command-semantics.md].

= Documentation

Docs live in `www/content/*.typ` and the `docs/src/*.typ` Markdown sources.
A touched file is refactored whole, one home per fact, checked examples over
described ones; the rules and the #just.dup-prose() audit are in
#link("docs/contributing/documentation.md")[docs/contributing/documentation.md].
Why the site build always rebuilds clean is in
#link("docs/contributing/docs-site-build.md")[docs/contributing/docs-site-build.md].

= Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args`
  struct and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
