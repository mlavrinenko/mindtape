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
  just execute it. A surviving `MISSED` mutant is a real assertion gap. Not a
  `just check` gate — deliberately no CI job either (see below). Two commands:
  `just mutants-diff` sweeps only mutants touched by this branch's diff against
  `main` — the pre-PR check, minutes not hours, no state file, and it covers
  new files for free (every line of a new file counts as touched). `just
  mutants -f <file>` forces a deliberate full-file sweep. Triage each survivor
  into a real gap (write a regression test) or an equivalent mutant (add it to
  `.cargo/mutants.toml` `exclude_re` with its reason). `.cargo/mutants.toml`
  also holds the excluded files (`main.rs`, `recur_rrule_off.rs`) and the
  `rrule` feature; `just crap` picks where a pass pays off.
- A `TIMEOUT` verdict is not a verdict in either direction — under parallel
  load it can hide a real pass or a real survivor, so a final-verdict sweep
  runs at `CARGO_MUTANTS_JOBS=2` or lower with an extended timeout, not the
  default parallelism. When a `TIMEOUT` is reproducible, it found a real
  unbounded loop (`MAX_CATCH_UP_STEPS` in `recur_dates.rs`,
  `MAX_DISAMBIGUATE_ATTEMPTS` in `naming.rs`) — bound it, and always by a `for`
  over a fixed range, never by the counter a mutant can freeze (`while n <=
  CEILING` still hangs once the mutant stops incrementing `n`). A survivor
  inside `#[cfg(not(feature = …))]` code is unreachable under the sweep's own
  feature set — move the twin into its own file and `exclude_globs` it
  (`recur_rrule_off.rs`) so the live side stays one always-compiled function.
- No CI mutation job, on purpose: every mutant re-runs a suite that includes
  subprocess and PTY e2e tests, and `TIMEOUT` verdicts are untrustworthy under
  parallel load — exactly the environment a shared runner is. An advisory
  check with unreliable verdicts trains people to ignore it, and a cold build
  on two runner cores turns "advisory" into the slowest job in the pipeline.
  Run `just mutants-diff` locally instead, where a timeout means something.

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

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the pre-commit freshness
assertion (`mmz --is-fresh --tag gate`); `typstyle --check truth/tasks` and
`mt check` still run, and `.githooks/commit-msg` refuses the commit unless the
message carries a matching `Gate-Scope: <reason>` trailer — a scoped commit
never reads like a fully gated one in `git log`. Legitimate for a delegated
worktree whose full aggregate runs at integration; never legitimate to dodge a
gate that is actually red.

Coverage runs separately (`just cover`, CI `coverage` job) via `cargo-llvm-cov`,
which merges each process's profraw — so the subprocess-based e2e tests credit
the command code they exercise (tarpaulin, which it replaced, could not see past
`exec` and read every command entry point as dead). The recipe writes
`target/coverage/lcov.info` for `just crap` and fails under 90% lines.
`main.rs` is excluded — keep it thin. `just cover` builds default features only,
so `recur_rrule.rs` (the `rrule` cargo feature's expander) never compiles and
`just crap` excludes it too, rather than scoring it as phantom 0%-covered;
`just check-rrule` covers it separately, under the `rrule` feature.

`just crap` runs right after it, in the same CI job and over the same
`lcov.info`: the CRAP metric (`CC² × uncovered³ + CC`) fails above 30, flagging
functions that are both complex and undertested. It reports only the functions
that fail (`--min` tracks `--threshold`) and skips the inline `*_tests.rs`
modules, and it refuses to run at all when `lcov.info` is older than a source
file — a function the lcov never saw is scored as 0% covered, not as unknown,
so a stale lcov invents failures (a 10-day-old one reported 28, of which 27 were
phantom). Run `just cover` first; CI already does. cargo-crap is not in nixpkgs,
so `flake.nix` builds it from the crates.io tarball with naersk `singleStep`
(crates.io 403s the user agent nixpkgs' `fetch-cargo-vendor` sends, so
`buildRustPackage` cannot compute a vendor hash). A flat dispatcher sits near
the limit by construction — `filter.rs` `function` scores 28 at full coverage —
so adding an arm there means covering it in the same change.

Since nixpkgs' `rustc` ships no `llvm-tools-preview`, the dev shell exports
`LLVM_COV`/`LLVM_PROFDATA` from `llvmPackages_21` — the same LLVM major as
`rustc` 1.92. Bumping the toolchain across an LLVM major means bumping that
`llvmPackages_*` in `flake.nix` too, or `llvm-profdata` rejects the profraw.

`just nix-build` verifies that `nix build` succeeds in a clean sandbox (no dev
shell, no sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; it
is called automatically by `just release`.

## Signal Sweep (`just ask-refactoring`)

`contrib/ask-refactoring.sh` reads the *advisory* quality signals and files a
backlog task per finding. It is not a gate and never will be — it is the other
half of the gates: a gate stops you when the wall is already in front of you, the
sweep says which wall you are walking toward.

The entry script holds the argument parsing, the re-run guard, the shared
`stem_for` and the driver; each arm is one file under `contrib/ask-refactoring/`
defining `score_<arm>`/`signal_<arm>`/`emit_<arm>` next to the reasoning that
justifies its numbers. Adding an arm is a new file there, its name in `ARMS`,
and a template — still no config change.

```
just ask-refactoring                    # dry: print the tasks it would file
just ask-refactoring --write            # file them
just ask-refactoring --baseline 85      # widen the linecop threshold (default 90)
just ask-refactoring --coverage-floor 85 # widen the coverage threshold (default 88)
```

The mutants arm has no threshold: it reads whatever `just mutants-diff` (or
`just mutants -f FILE`) last wrote to `mutants.out/`, and skips when that is
missing or older than the sources.

Five rules the design turns on, in the order they matter:

- **Only advisory signals qualify.** jscpd, outdatty, machete and clippy already
  block a commit; a task from one of those would be a note about a wall you are
  standing at. All three arms are live: `linecop --format json` for file size,
  `cargo llvm-cov report --json` (over the lcov `just cover` writes, refreshed
  via `just mmz cover`) for per-file line coverage — advisory because `just
  cover` gates only the workspace total, never a single file — and
  `mutants.out/outcomes.json` for surviving mutants. There is deliberately no
  CRAP arm: above ~90% coverage CRAP degenerates into cyclomatic complexity, so
  no band below the `just crap` gate separates signal from noise here. Measured,
  not assumed — the numbers are in the `crap` recipe comment.
- **A signal never fakes a measurement it does not have.** Coverage refreshes
  its own lcov and *refuses* a stale one. The mutants arm cannot: a full
  `cargo mutants` run is hours, so it *skips* — loudly, naming
  `just mutants-diff` — rather than blocking the other two arms or, worse,
  printing "no survivors" when nobody has measured. Absent and clean must never
  look alike.
- **One task per finding, never one per sweep.** Each is dispatched to its own
  agent and has to carry its own scope. Two axes shape it, kept apart on
  purpose. *Language* decides the remedy — linecop caps Rust and Markdown, and
  shedding a section into `references/` is not carving a module, so the filed
  task says the one that applies; it comes from linecop's own `language` field,
  never re-derived from the extension. *Test versus production* decides tone and
  score, and is a Rust distinction only: both need a real seam decision, but an
  already-ejected `*_tests.rs` is different work from the file it came out of,
  and `just eject` has no second move on it.
- **Idempotency comes from the signal, not a ledger.** Nothing records what was
  filed. The one deliberate suppression is an override with a *written reason* in
  the signal's own config — a `.linecop.yaml` entry for size, in the style the
  `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` ones already use; an `--exclude`
  entry (with its reason in the comment beside it) in the `just cover` /
  `just crap` recipes for coverage; an `exclude_re` line in `.cargo/mutants.toml`
  for an equivalent mutant. There is no ignore list anywhere else, so silencing a
  finding always leaves a sentence behind saying why. Scope is derived, not
  recorded: a mutants task names what that sweep actually covered, read out of
  `outcomes.json` itself, so a one-file run never reads as a workspace verdict.
- **The remedy is the refactorer's call.** Split thoughtfully, or raise the limit
  with an override that names a new sane number and the reason for it. A split
  for its own sake is not the goal; the cap pushes back on scope growth, not on
  accuracy.

The re-run guard is `[views.ask-refactoring]` in `.mindtape/config.toml`
(`has_tag("swept") && !terminal`). While anything the sweep filed is still open,
the sweep refuses and names the count — one sprint of mechanical work is emptied
before the next is filed. Every filed task carries the provenance tag `swept`
plus one signal tag (`size`, `mutants`, `coverage`); the guard selects on `swept`
alone, so a new signal joins it for free.

Scoring is honest per finding, not a constant triple wearing a framework's
clothes: impact rises as headroom falls or as the unreached fraction grows, ease
falls with how much has to be written or moved, and confidence is high (0.9)
because both tools *measure* rather than guess — 0.7 only for a test module,
where the remedy is genuinely open. The formulas and their justification live in
the script, next to the code that applies them.

Adding a signal is a scoring function plus a `.mindtape/templates/sweep-*.typ`
template, never a config change: the tags and the guard are already declared. The
three templates share one shape, kept in step by the `sweep-templates` outdatty
group rather than by a Typst `#import` — an import would survive into every task
the sweep writes and bind a tracked artifact to a path outside the tasks root.

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
