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
environment. `just` (no args) lists them. What is in the shell and why each tool
is there is `flake.nix`'s subject, commented in place; this file covers the
workflow around the tools, not their inventory.

This document is the single source for how work is done here. `AGENTS.md` pulls
it into agent context with `@CONTRIBUTING.md` and describes only what MindTape
*is* — architecture and the task DSL. The two must not overlap: a rule about
process belongs here, and nowhere else.

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
- `tests/interactive_e2e.rs` drives `mt ls --interactive` under a real PTY via
  `script(1)` (util-linux, in the flake devShell) — it needs a TTY to exercise
  the `guard` checks and `event_loop`/`draw`. It and `tests/tty_e2e.rs` are
  `#![cfg(unix)]`: `script(1)` has no Windows equivalent, so on Windows they
  compile to nothing rather than being skipped at runtime.
- CI runs the suite on Windows too (`.github/workflows/ci.yml`, job `windows`),
  which is where the platform seams get exercised — `cmd /C` vs `sh -c` for
  `[[on.flip]]` `run`, the `dirs` config/data paths, path separators. It runs
  plain `cargo build`/`cargo test`, not `just check`: there is no Nix there, and
  the remaining gates are platform-independent. Anything you write that shells
  out or joins paths needs a thought for that job.
- `just check-windows` cross-checks the workspace (tests included) against
  `x86_64-pc-windows-gnu` from Linux, so compile breakage surfaces before the
  push rather than in CI. It runs in its own dev shell (`nix develop .#windows`)
  because the default shell's rustc carries no Windows std; `pkgsCross.mingwW64`
  supplies a rustc — the same pinned 1.92.0 — that has std for the target and
  the host both, so no rustup and no second toolchain manager is involved. Not
  a `just check` arm: it pulls a separate toolchain closure and `target/` tree.
  Windows CI still owns runtime behaviour and the `-msvc` target; this only
  moves the compile half earlier.
- Run `just test`; count with `just count-tests`.
- Mutation testing (`cargo-mutants`) proves whether tests pin behavior, not
  just execute it. A surviving `MISSED` mutant is a real assertion gap. Run
  `just mutants-diff` before a PR — it sweeps only the mutants your branch's
  diff touches, so it is minutes not hours and covers new files for free.
  `just mutants -f <file>` forces a deliberate full-file sweep. Triage each
  survivor into a real gap (write a regression test) or an equivalent mutant
  (`.cargo/mutants.toml` `exclude_re`, with its reason). Neither command is a
  `just check` gate, and there is no CI job for it on purpose — the reasoning,
  the `TIMEOUT` caveat, and how to bound a loop a mutant can hang are in the
  `mutants` recipes' comments and in `.cargo/mutants.toml`.

## Quality Gates (run by `just check`)

| Gate | Tool |
|------|------|
| Format | `just fmt-check` — `cargo fmt` over Rust, `tombi format` over the project config TOML (`.mindtape/config.toml` and the per-example configs), and `typstyle` over the Typst sources (`lib/`, `truth/tasks/`, both template dirs, the learn-piano example, and `www/`). The non-Rust path lists live in the `TOML_FMT_PATHS`/`TYPST_FMT_PATHS` Justfile variables, one list feeding both `fmt` and `fmt-check` — add a path there, never to one recipe |
| Lints | `cargo clippy --all-targets -D warnings` |
| Tests | `cargo test --workspace` |
| Unused deps | `cargo machete` |
| File size | `linecop` (Rust 500, Markdown 200; 250 README, 300 `AGENTS.md`/`SKILL.md`/`CONTRIBUTING.md`). `linecop --baseline 85` lists what is nearing its cap — run it before adding to an existing file |
| Copy-paste | `jscpd` (`just check-dry`) — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper |
| Doc drift | `outdatty check` |
| Fixtures | `just check-examples` — `mt check` against each example project: `examples/learn-piano/` (full config surface; feeds the docs generator and the skill's triage evals) and `examples/minimal/` (the floor config; feeds the skill's trap evals). Keep the recipe and the mmz `examples` scope in step. A fixture no gate validates rots silently |
| Doc coverage | `just check-doc-coverage` — set-differences `mt _cli --json`'s reflected command names against `www/utils/cli-notes.typ`'s dict keys; fails naming any visible subcommand with no matching entry, so a new verb can no longer ship silently undocumented |
| Docs site | `just docs-check` — `docs-gen` first captures live `mt` output (`www/generate.sh`), then `tola build`, then `pagefind` indexes the built HTML for client-side search, then `tola validate` checks links + assets |
| CRAP (partial) | `just check-crap` — runs the CRAP gate when `target/coverage/lcov.info` is fresh, and otherwise SKIPS LOUDLY, saying so and exiting 0. Editing any source stales the lcov, so on the commits that matter it skips; it catches the free case only. `just check-full` is the arm that is always correct |

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the pre-commit freshness
assertion (`mmz --is-fresh --tag gate`); `typstyle --check truth/tasks` and
`mt check` still run, and `.githooks/commit-msg` refuses the commit unless the
message carries a matching `Gate-Scope: <reason>` trailer — a scoped commit
never reads like a fully gated one in `git log`. Legitimate for a delegated
worktree whose full aggregate runs at integration; never legitimate to dodge a
gate that is actually red.

`just check-full` is `just check` plus `just cover` and `just crap` — the pre-PR
run, always correct about coverage and CRAP, at the price of an instrumented
rebuild and a full suite pass. Use it before opening a PR; use plain
`just check` while iterating.

Coverage runs separately (`just cover`, CI `coverage` job) via `cargo-llvm-cov`,
which merges each process's profraw — so the subprocess-based e2e tests credit
the command code they exercise (tarpaulin, which it replaced, could not see past
`exec` and read every command entry point as dead). The recipe writes
`target/coverage/lcov.info` for `just crap` and fails under 90% lines.
`main.rs` is excluded — keep it thin. `just cover` builds default features only,
so `recur_rrule.rs` (the `rrule` cargo feature's expander) never compiles and
`just crap` excludes it too, rather than scoring it as phantom 0%-covered;
`just check-rrule` covers it separately, under the `rrule` feature.

It throws the previous run away first (`cargo llvm-cov clean --workspace`), so it
always pays a full instrumented rebuild (~1 min). Without that, `report` unions
every profraw and object file left since the last `cargo clean` — other feature
sets, other commits — and reports coverage this source cannot produce; it
inflated `crap-baseline.json` once already. Never trade that line for speed. The
recipe's comment carries the measurement.

`just crap` runs right after it, in the same CI job and over the same
`lcov.info`: the CRAP metric (`CC² × uncovered³ + CC`) fails above 30, flagging
functions that are both complex and undertested. It refuses to run against an
lcov older than a source file, so run `just cover` first — CI already does. Why
it refuses, what it excludes, and why there is deliberately no advisory band
below the threshold are argued in the `crap` recipe's own comment. Read that
before changing its flags.

One consequence is worth carrying in your head rather than looking up: above
~90% coverage the cubed term vanishes and a function scores its own cyclomatic
complexity, so a flat dispatcher sits near the threshold by construction, and
adding an arm to one means covering it in the same change.

That same collapse is why the threshold alone is not the whole gate. `just crap`
runs a second check against `crap-baseline.json`, a committed score for every
function: nothing may get WORSE than the baseline, at any absolute level. It
never reads the absolute score, only the delta, so the collapse does not apply —
a rise means complexity went up or coverage went down either way. It is what
catches a function that is complex and undertested but scores well under 30,
which the threshold is structurally blind to. When you deliberately add
complexity, run `just crap-baseline` in the same commit so the new number lands
next to the reason for it.

How the dev shell supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs
is `flake.nix`'s subject — both are non-obvious, both break loudly if changed
carelessly, and both are commented there.

`just nix-build` verifies that `nix build` succeeds in a clean sandbox (no dev
shell, no sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; it
is called automatically by `just release`.

## Signal Sweep (`just ask-refactoring`)

`contrib/ask-refactoring.sh` reads the *advisory* quality signals and files a
backlog task per finding. It is not a gate and never will be — it is the other
half of the gates: a gate stops you when the wall is already in front of you,
the sweep says which wall you are walking toward. How it works — what each arm
reads, how it scores, the six design rules, the re-run guard — lives in
[docs/contributing/signal-sweep.md](docs/contributing/signal-sweep.md); run
`just ask-refactoring` to see what it would file today.

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) couples source files to the dependents that must
stay in sync (e.g. the CLI surface to `README.md`). After editing a source,
review the listed dependents, update them, then run `just outdatty-update` to
record the new state into `outdatty.lock` and commit it.

## Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates. On bump,
  every place that repeats it has to move with it — the `TYPST_PKG_VERSION` const
  in `crates/mindtape-cli/src/init.rs`, the `PKG_VERSION` const in
  `www/utils/site.typ` (the docs render every `#import` example from it), and the
  import examples in `README.md` and `AGENTS.md`. The `typst-pkg-version`
  outdatty group is what enforces that; trust it over this list.
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
are tracked as task artifacts too; the ADR process is retired (`docs/decisions/`
is frozen history).

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

"Byte-identical" is the *authored* tree — task files and config. A read may
populate the git-ignored `.mindtape/cache/` scratch (the persistent content-hash
harvest cache); that derived artifact is never authored state, the way `target/`
is not, so it is exempt from the invariant.

## Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args` struct
  and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its
  `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR
  process is retired).
