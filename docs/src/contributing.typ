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
environment. `just` with no arguments lists them. The tools themselves are
`flake.nix`'s subject.

This document is the single source for how work is done here — a process rule
belongs here and nowhere else. `AGENTS.md` pulls it into agent context and
covers only what MindTape _is_. Deep dives sit in `docs/contributing/`.

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
re-running it. That memoization, the gated example projects and the locked
formatter gates are in
#link("docs/contributing/gates.md")[docs/contributing/gates.md].

#just.check-full() is #just.check() plus #just.cover() and #just.crap():
always correct about coverage and CRAP, at the price of an instrumented
rebuild and a full suite pass. Run it before opening a PR, plain #just.check()
while iterating.

A delegated worktree whose full aggregate runs at integration can scope the
gate with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which demands a matching
`Gate-Scope:` trailer so the commit never reads as fully gated. Dodging a gate
that is actually red is never legitimate — when scoping one is, and what the
trailer buys, are in #link("docs/contributing/gates.md")[docs/contributing/gates.md].

Coverage and CRAP run outside #just.check(). Run #just.cover() first —
#just.crap() refuses a stale lcov — and #just.crap-baseline() in the same
commit as any deliberate complexity. The thresholds and the argument behind
every number are in
#link("docs/contributing/coverage-crap.md")[docs/contributing/coverage-crap.md].
Read it before changing either recipe's flags.

#just.nix-build() verifies `nix build` in a clean sandbox (no dev shell, no
sccache). Run it when touching `flake.nix` or `.cargo/config.toml`;
#just.release() calls it automatically.

= Signal Sweep (#just.ask-refactoring())

The sweep reads the _advisory_ quality signals and files a backlog task per
finding. It is not a gate: a gate stops you at the wall, the sweep names the
wall you are walking toward. Run #just.ask-refactoring() to see what it would
file today. The arms, the scoring and the design rules are in
#link("docs/contributing/signal-sweep.md")[docs/contributing/signal-sweep.md].

= Dependency Drift (outdatty)

#link("outdatty.yaml")[`outdatty.yaml`] couples source files to the
dependents that must stay in sync (e.g. the CLI surface to `README.md`).
After editing a source, update the listed dependents, re-record the group you
touched with #just.outdatty-update("<group>") and commit `outdatty.lock`. A
recorded hash is a review watermark, so blanket re-recording —
#just.outdatty-update-all() — claims a review that never happened. It stays
the explicit choice, never the default.

Re-record the group when you edit a dependent on its own, too, in the same
commit. Only a source change is a trigger, so the gate stays green either way
— but the dependent's recorded hash then sits stale until some unrelated
source edit carries it along, putting the watermark on a commit that never
touched the file. It costs one lock line, and it is a convention rather than
a gate. `outdatty.yaml`'s header carries the measurement, the reasoning and a
one-liner listing groups with a dependent still pending.

= Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates.
  Several files repeat it and all have to move together; the
  `typst-pkg-version` outdatty group enumerates them and fails the build
  otherwise. Trust the group, not a list here.
- `mindtape-cli`'s dependency on `mindtape` carries a version requirement
  (not path-only), because crates.io rejects path-only deps on publish; bump
  it with the workspace version.
- A release (#just.release("VERSION")) is gated twice. `VERSION` must be a
  bare semver with no leading `v`: an `[arg(...)]` pattern rejects anything
  else before the recipe body runs, so `v0.3.0` fails fast with a pattern
  mismatch instead of a confusing "Cargo.toml is 0.3.0" error from inside the
  script. Then the body gates — the tag must equal the workspace version, the
  working tree must be clean, and `CHANGELOG.md` may not have a non-empty
  `[Unreleased]` section (it moves into a dated `[VERSION]` section at release
  time). The recipe pushes `main` before the tag, so the docs site
  (`pages.yml`) and CI run on the release tree.

= Commit Messages

#link("https://www.conventionalcommits.org/")[Conventional Commits]. Match
the language of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: truth/tasks/<stem>.typ
```

MindTape tracks its own backlog as per-file task artifacts under
`truth/tasks/` (dogfooding itself — see `.mindtape/config.toml`). Point
`Refs:` at the driving task file. Architectural and tooling decisions are
task artifacts too; the ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config
in `.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs
nothing.

= Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never
mutate the working tree; only explicit write commands do.

- Read (pure): `ls`, `check`, `show`, `export`, `config` queries — leave the
  tree byte-identical (enforced by test).
- Write (explicit): `add`, `recur`, `done`, `flip`, `rm`, `init`.

Recurrence never materializes as a side effect of a read: an instance is
spawned or refreshed only by an explicit `mt recur`, and `mt done` closes a
task without creating or advancing anything.

"Byte-identical" means the _authored_ tree — task files and config. A read may
still populate the git-ignored `.mindtape/cache/` scratch, derived state the
way `target/` is.

= Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args`
  struct and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
