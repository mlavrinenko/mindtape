# Contributing to MindTape

## Quick Start

```bash
git clone https://github.com/mlavrinenko/mindtape.git
cd mindtape
nix develop          # dev shell with the full toolchain
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
  the `guard` checks and `event_loop`/`draw`.
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
| Format | `just fmt-check` — `cargo fmt` over Rust, `tombi format` over the project config TOML (`.mindtape/config.toml` and the per-example configs), and `typstyle` over the Typst sources (`lib/`, `truth/tasks/`, the learn-piano example, and `www/`) |
| Lints | `cargo clippy --all-targets -D warnings` |
| Tests | `cargo test --workspace` |
| Unused deps | `cargo machete` |
| File size | `linecop` (Rust 500, Markdown 200; 250 README, 300 `AGENTS.md`/`SKILL.md`). `linecop --baseline 85` lists what is nearing its cap — run it before adding to an existing file |
| Copy-paste | `jscpd` (`just check-dry`) — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper |
| Doc drift | `outdatty check` |
| Fixtures | `just check-examples` — `mt check` against each in-tree fixture project: `examples/learn-piano/` (the docs generator's input) and `skills/mindtape/evals/files/sandbox/` (the mindtape skill's eval input). A fixture no gate validates rots silently |
| Docs site | `just docs-check` — `docs-gen` first captures live `mt` output (`www/generate.sh`), then `tola build`, then `pagefind` indexes the built HTML for client-side search, then `tola validate` checks links + assets |

Coverage runs separately (`just cover`, CI `coverage` job) via `cargo-llvm-cov`,
which merges each process's profraw — so the subprocess-based e2e tests credit
the command code they exercise (tarpaulin, which it replaced, could not see past
`exec` and read every command entry point as dead). The recipe writes
`target/coverage/lcov.info` for `just crap` and fails under 90% lines.
`main.rs` is excluded — keep it thin.

`just crap` runs right after it, in the same CI job and over the same
`lcov.info`: the CRAP metric (`CC² × uncovered³ + CC`) fails above 30, flagging
functions that are both complex and undertested. cargo-crap is not in nixpkgs,
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
