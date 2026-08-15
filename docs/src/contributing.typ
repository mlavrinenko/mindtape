// docs/src/contributing.typ — source for the repo-root CONTRIBUTING.md.
// Rendered by `docs/generate-md.sh` via typlite; see docs/src/lib.typ.
//
// Content-preserving port, except for the Quality Gates table:
// `www/generated/gates.json` carries the sub-gates `just check` declares, in
// order, each with its Justfile doc comment and a derived `command` — the
// recipe's own body, read back rather than retyped, so it cannot drift the
// way the hand-quoted clippy invocation once did (www/gates.jq argues the
// derivation). Gate and Command are both machine-derived; the Notes column's
// prose has no machine source, so it stays hand-written in
// `www/utils/gate-notes.typ` — the same `www/utils/*-notes.typ` house
// pattern as `crate-notes.typ`, `dsl-field-notes.typ` and `cli-notes.typ` —
// keyed by gate name via `.at()`. A gate missing there fails `just docs::md`
// naming it, and one removed from `just check` drops out on its own.
// `.linecop.yaml`'s caps land in `www/generated/linecop-caps.json` too, but
// this file quotes no literal cap to derive from it. `gates.json` also
// carries the Justfile's fmt path assignments (TOML_FMT_PATHS, TYPST_FMT_PATHS,
// JUST_FMT_PATHS — selected by their `_FMT_PATHS` suffix in www/gates.jq, so
// a future formatter's list lands without an edit) `fmt-check`'s note used
// to only name; the block below the table renders their real contents.
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

Use `just` recipes instead of raw `cargo` — they carry the right flags and
environment. `just` (no args) lists them. Tool inventory is `flake.nix`'s
subject, commented in place; this file covers the workflow around them.

This document is the single source for how work is done here — a rule about
process belongs here and nowhere else. `AGENTS.md` pulls it into agent context
and describes only what MindTape _is_. Deep dives sit in `docs/contributing/`.

= Code Style

All clippy lints are `deny` (workspace `Cargo.toml`); the project will not
build with violations. Key restrictions:

- No `unwrap()`/`expect()` outside tests — use `?` with `anyhow`/`thiserror`
- No `todo!()`/`unimplemented!()`/`unreachable!()`/`unsafe`/wildcard imports
- Functions: ≤70 lines, ≤5 args, cognitive complexity ≤20

= Error Handling

- `anyhow::Result` for application code (the CLI)
- `thiserror::Error` for library error types callers match on
  (`HarvestError`, `ConfigError`, `FilterError`)
- Keep `main.rs` a thin entry point; logic lives in library modules

= Testing

- Unit tests inline in `#[cfg(test)] mod tests` (or a sibling `_tests.rs` via
  `#[path]`) next to the code they exercise.
- Tests that shell out to the `mt` binary (`assert_cmd::cargo_bin`) belong in
  `tests/` (integration tests), not inline: Cargo builds the binary and
  exports `CARGO_BIN_EXE_mt` before integration tests, a guarantee `--lib`
  unit tests lack.
- As a file nears the linecop limit, #just.fix-check() ejects its inline tests
  into a `_tests.rs` sibling with
  #link("https://github.com/mlavrinenko/ejectest")[ejectest].
- Run #just.test()\; count with #just.count-tests().
- Windows and the pty-driven terminal tests have their own seams — CI job,
  #just.check-windows() cross-check, why `tests/interactive_e2e.rs` is
  `#![cfg(unix)]`:
  #link("docs/contributing/windows.md")[docs/contributing/windows.md]. Anything
  you write that shells out or joins paths needs a thought for that job.
- Mutation testing (`cargo-mutants`) proves whether tests pin behaviour, not
  just execute it. Run #just.mutants-diff() before a PR and triage every
  survivor:
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

`fmt-check`'s non-Rust path lists — one list feeds both `fmt` and
`fmt-check`, so a path is added in one place, never to a single recipe:

- `TOML_FMT_PATHS`: #gate-assignments.at("TOML_FMT_PATHS").split(" ").map(raw).join(", ")
- `TYPST_FMT_PATHS`: #gate-assignments.at("TYPST_FMT_PATHS").split(" ").map(raw).join(", ")
- `JUST_FMT_PATHS`: #gate-assignments.at("JUST_FMT_PATHS").split(" ").map(raw).join(", ")

Every arm runs through `mmz`, so a re-run skips each one whose declared inputs
are unchanged and the pre-commit hook can assert the whole set fresh without
re-running it. That memoization, why the example projects are gated, and how
the formatter gates are themselves locked are in
#link("docs/contributing/gates.md")[docs/contributing/gates.md].

#just.check-full() is #just.check() plus #just.cover() and #just.crap() — always
correct about coverage and CRAP, at the price of an instrumented rebuild and a
full suite pass. Use it before opening a PR, plain #just.check() while
iterating.

A delegated worktree whose full aggregate runs at integration can scope the
gate with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which requires a
matching `Gate-Scope:` trailer so the commit never reads as fully gated:
#link("docs/contributing/gates.md")[docs/contributing/gates.md]. Never
legitimate to dodge a gate that is actually red.

Coverage and CRAP run outside #just.check(): #just.cover() fails under 90%
lines, then #just.crap() fails above 30 and on any regression against
`crap-baseline.json`. Run `cover` first — `crap` refuses a stale lcov — and
#just.crap-baseline() in the same commit as any deliberate complexity. Every
one of those numbers is argued in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md]
; read it before changing either recipe's flags.

#just.nix-build() verifies `nix build` in a clean sandbox (no dev shell, no
sccache). Run it when touching `flake.nix` or `.cargo/config.toml`;
#just.release() calls it automatically.

= Signal Sweep (#just.ask-refactoring())

`contrib/ask-refactoring.sh` reads the _advisory_ quality signals and files a
backlog task per finding. It is not a gate and never will be — a gate stops
you at the wall, the sweep names the wall you are walking toward. Run
#just.ask-refactoring() to see what it would file today; the arms, the scoring
and the design rules are in
#link("docs/contributing/signal-sweep.md")[docs/contributing/signal-sweep.md].

= Dependency Drift (outdatty)

#link("outdatty.yaml")[`outdatty.yaml`] couples source files to the
dependents that must stay in sync (e.g. the CLI surface to `README.md`).
After editing a source, update the listed dependents, then
#just.outdatty-update() and commit `outdatty.lock`.

= Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates.
  Several files repeat it and all have to move together; the
  `typst-pkg-version` outdatty group enumerates them and fails the build
  otherwise. Trust the group, not a list here.
- `mindtape-cli`'s dependency on `mindtape` carries a version requirement
  (not path-only), because crates.io rejects path-only deps on publish; bump
  it with the workspace version.
- A release (#just.release("VERSION")) is gated twice. `VERSION` itself must
  be a bare semver with no leading `v` (an `[arg(...)]` pattern rejects
  anything else before the recipe body ever runs, so a typo like `v0.3.0`
  fails fast with a pattern-mismatch message instead of a confusing "Cargo.toml
  is 0.3.0" error from deep inside the script). Past that, the recipe body
  itself gates: the tag must equal the workspace version, the working tree
  must be clean, and `CHANGELOG.md` may not have a non-empty `[Unreleased]`
  section (it moves into a dated `[VERSION]` section at release time). The
  recipe pushes `main` before the tag, so the docs site (`pages.yml`) and CI
  run on the release tree.

= Commit Messages

#link("https://www.conventionalcommits.org/")[Conventional Commits]. Match
the language of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: truth/tasks/<stem>.typ
```

MindTape tracks its own backlog as per-file task artifacts under
`truth/tasks/` (MindTape dogfooding itself — see `.mindtape/config.toml`).
Point `Refs:` at the driving task file (`truth/tasks/<stem>.typ`).
Architectural and tooling decisions are tracked as task artifacts too; the
ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config
in `.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs
nothing.

= Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never
mutate the working tree; only explicit write commands do.

- Read (pure): `ls`, `check`, `show`, `export`, `config` queries — leave the
  tree byte-identical (enforced by test).
- Write (explicit): `add`, `recur`, `done`, `flip`, `rm`, `init`.

Recurrence never materializes as a side effect of a read. An instance is
spawned or refreshed only by an explicit `mt recur`; `mt done` closes a task
and never creates or advances anything.

"Byte-identical" is the _authored_ tree — task files and config. A read may
still populate the git-ignored `.mindtape/cache/` scratch, which is derived
state the way `target/` is, not authored state.

= Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args`
  struct and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
