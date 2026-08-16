<!-- Generated from docs/src/contributing.typ by `just docs-md`. Do not edit; edit the source. -->

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

Use `just` recipes rather than raw `cargo`: they carry the right flags and environment. `just` with no arguments lists them. The tools themselves are `flake.nix`’s subject.

This document is the single source for how work is done here — a process rule belongs here and nowhere else. `AGENTS.md` pulls it into agent context and covers only what MindTape _is_. Deep dives sit in `docs/contributing/`.

## Code Style

All clippy lints are `deny` (workspace `Cargo.toml`); the project will not build with violations. Key restrictions:

- No `unwrap()`/`expect()` outside tests — use `?` with `anyhow`/`thiserror`
- No `todo!()`/`unimplemented!()`/`unreachable!()`/`unsafe`/wildcard imports
- Functions: ≤70 lines, ≤5 args, cognitive complexity ≤20

## Error Handling

- `anyhow::Result` for application code (the CLI)
- `thiserror::Error` for library error types callers match on (`HarvestError`, `ConfigError`, `FilterError`)
- Keep `main.rs` a thin entry point; logic lives in library modules

## Testing

- Unit tests inline in `#[cfg(test)] mod tests` (or a sibling `_tests.rs` via `#[path]`) next to the code they exercise.
- Tests that shell out to the `mt` binary (`assert_cmd::cargo_bin`) belong in `tests/`, not inline: Cargo exports `CARGO_BIN_EXE_mt` before integration tests, a guarantee `--lib` unit tests lack.
- As a file nears the linecop limit, `just fix-check` ejects its inline tests into a `_tests.rs` sibling with [ejectest](https://github.com/mlavrinenko/ejectest).
- Run `just test`; count with `just count-tests`.
- Anything that shells out or joins paths needs a thought for the Windows CI job. What runs there, the `just check-windows` cross-check and the Unix-only pty tests: [docs/contributing/windows.md](docs/contributing/windows.md).
- Mutation testing (`cargo-mutants`) proves whether tests pin behaviour, not just execute it. Run `just mutants-diff` before a PR and triage every survivor: [docs/contributing/mutation-testing.md](docs/contributing/mutation-testing.md).

## Quality Gates (run by `just check`)

| Gate | Command | Notes |
| --- | --- | --- |
| `fmt-check` | _Check formatting without writing changes (CI-friendly)_ | `cargo fmt` over Rust, `tombi format` over the project config TOML (`.mindtape/config.toml` and the per-example configs), `typstyle` over the Typst sources, and `--fmt` over the justfiles — the exact path lists are below the table. One list feeds both `fmt` and `fmt-check`, so a path is added in one place, never to a single recipe. The justfiles need a list of their own because a bare `--fmt` formats the root `Justfile` only and silently skips every `mod` file; each file listed gets its own `--justfile <file> --fmt` pass and needs `set unstable := true` of its own, since `--fmt` reads that setting from the file it is pointed at, not from the `Justfile` declaring the `mod`. |
| `clippy` | `cargo clippy --workspace --all-targets -q -- -D warnings` | Every lint is workspace `deny` (see Code Style above); this recipe is only the enforcement point, not a separate lint surface. |
| `test` | `cargo test --workspace "$@"` | The full workspace suite. `just test <filter>` (a developer query, no `mmz`) runs this recipe directly with the filter as `ARGS`, so a filtered run never pollutes the cached full-suite record `just check` relies on. |
| `machete` | `cargo machete` | Flags a workspace dependency no crate actually imports. |
| `check-file-size` | `linecop` | Per-language caps — and the tighter ones on the always-loaded docs — are in [`.linecop.yaml`](.linecop.yaml), argued in place. `linecop --baseline 85` lists what is nearing its cap; run it before adding to an existing file. |
| `check-dry` | _Copy-paste gate: fail on any duplicated Rust block_ | `jscpd` — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper. |
| `outdatty-check` | `outdatty check` | See Dependency Drift below for how `outdatty.yaml` couples a source to the dependents it must stay in sync with. |
| `check-examples` | _Validate each example project against itself_ | `mt check` against each example project: `examples/learn-piano/` (full config surface; feeds the docs generator and the skill’s triage evals) and `examples/minimal/` (the floor config; feeds the skill’s trap evals). Keep the recipe and the mmz `examples` scope in step. A fixture no gate validates rots silently. |
| `check-doc-coverage` | _Fail if a visible mt subcommand has no CLI doc entry_ | Set-differences `mt _cli --json`’s reflected command names against `www/utils/cli-notes.typ`’s dict keys; fails naming any visible subcommand with no matching entry, so a new verb can no longer ship silently undocumented. |
| `check-doc-facts` | _Fail if a derived crate, DSL field, gate, or sweep flag fact has no doc entry_ | Regenerates `www/generated/{crate-map,dsl-fields,gates,sweep-flags}.json` (`www/generate-facts.sh`) and set-differences each against its `www/utils/{crate-notes,dsl-field-notes,gate-notes,sweep-flag-notes}.typ` prose dict; fails naming any crate, DSL field, gate, or sweep flag with no matching entry — this file included. |
| `check-doc-recipes` | _Fail on a hand-typed just recipe mention outside the generated module_ | Fails on a hand-typed recipe mention in a doc source — literal `just` followed by a name, whether or not the name is real — that should have gone through the generated module (`www/generated/just.typ`, called as `#just.check()` and friends) instead. The three literal shapes it scans for (a prose backtick span, a fenced example’s leading command, a quoted config value) are argued in place in `.just/scripts/check-doc-recipes.sh`. |
| `docs-check` | `tola build && pagefind --site public/mindtape --silent && tola validate` | `docs-gen` — a dependency, not part of the command shown — captures live `mt` output first (`www/generate.sh`), so this gate always sees fresh CLI examples before `pagefind` indexes the build for client-side search and `tola validate` checks links and assets. |
| `docs-md-check` | _Fail if generated Markdown has drifted from docs/src_ | Regenerates every `docs/src/*.typ` source with typlite into a temp directory and diffs each against the committed Markdown it declares; fails naming any drifted file rather than healing it in place. `just docs md` refreshes it — see [docs/contributing/generated-docs.md](docs/contributing/generated-docs.md). |
| `fmt-fixture-check` | _Lock the formatter gates against a mis-formatted fixture_ | Synthesizes a deliberately mis-formatted TOML and Typst fixture in a temp directory and asserts `tombi format --check` and `typstyle --check` both reject it (and accept a reformatted one); locks the formatter gates themselves, so a formatter silently accepting drift could not let `fmt-check` pass on an out-of-sync editor save. |
| `arg-pattern-check` | _Lock the arg-attribute patterns against known-bad and known-good input_ | Dispatch-only self-test for the `[arg(...)]` patterns on `just release VERSION`, `just bench N`, `just gen-bench N`, `just eject PCT` and the five `just ask-refactoring` thresholds: asserts each rejects a known-bad argument and accepts a known-good one, and that every declared default satisfies its own pattern (`fix-check` depends on bare `eject`; the sweep’s defaults are the only copy of its thresholds). One of the sweep’s cases is a value containing a space — the word-splitting that `[positional-arguments]` used to guard that recipe against, refused at dispatch now instead. Every assertion goes through the `--dry-run --yes` flag pair, so no recipe body ever actually runs — `release`’s included; see `.just/scripts/arg-pattern-check.sh`. |
| `positional-args-check` | _Lock positional-argument forwarding on the four passthrough recipes_ | Static self-test for the four passthrough recipes — `just test <filter>`, `just build <args>`, `just mutants <args>` and `just mutants-diff <args>` — asserting each carries `[positional-arguments]` and forwards `"$@"` rather than interpolating `{{ ARGS }}` into its body, which would re-split an argument containing a space and re-glob one containing a wildcard. Reads the `--dump --dump-format json` dump and greps the scripts the recipes forward into; no recipe body runs. See `.just/scripts/positional-args-check.sh`. |
| `script-attr-check` | _Lock the script attribute and each justfile's strict-mode interpreter_ | Strict-mode self-test in two arms. No recipe body may open with a `#!` shebang — every script body reaches the interpreter through `[script]`, so `set script-interpreter` applies to it — and every justfile must declare a setting that actually aborts a failed pipeline. `shebang` in the dump cannot tell the two apart (a `[script]` recipe reports `true` exactly as a `#!` body does, measured on just 1.46), so the first arm pairs that field with the absence of a `script` attribute. The second runs a `false | true` probe under a replica justfile carrying the file’s own setting line verbatim, rather than matching its text: `-euo pipefail` would fail a literal match while being strictly better, and `-e` alone would pass a looser one while dropping the exact flag that let `just count-tests` report a partial sum and exit 0. Every justfile is checked, not only those with `[script]` recipes today — a module does not inherit the root’s setting, so `.just/docs.just`’s currently userless one is load-bearing and cannot be tidied away; see `.just/scripts/script-attr-check.sh`. |
| `size-fixture-check` | _Lock the size gate's scan scope against an oversized hidden-dir fixture_ | Scan-scope self-test for `just check-file-size`: an oversized shell script at `.just/scripts/` inside a replica tree must be reported by `linecop` reading this repo’s own `.linecop.yaml`, and must go unreported with `include_hidden` deleted from it. The caps need no self-test — a wrong number is visible — but a directory the walker never enters reports the same “All files within size limits.” a clean tree does, which is how every recipe body extracted into `.just/scripts/` sat uncapped while the extraction’s stated reason was bringing them under a cap. Runs against a replica rather than the real directory because `just check` runs its arms in parallel and a probe in the tracked tree would fail the real size gate mid-run; see `.just/scripts/size-fixture-check.sh`. |
| `script-scope-check` | _Lock every gate script into a named mmz scope_ | Enumeration lock for `.mmz/config.yaml`’s per-script scopes, in two arms. Every `.just/scripts/*.sh` must be named by some scope as a literal path, and every memoized rule whose recipe body invokes a script must declare a scope covering it — the second because a script listed in a scope no rule reads passes the first while still serving a stale hit. It exists because those scopes replaced a `.just/scripts/**` glob every rule carried, and the glob had the property an enumeration cannot have: a recipe body extracted out of the Justfile was covered by arriving. A wildcard never counts as naming a script, so putting the glob back satisfies nothing; a body reached through a nested `just` (check-crap.sh’s two) is the case the second arm deliberately does not derive. See `.just/scripts/script-scope-check.sh`. |
| `check-crap` | _Run the CRAP gate when the lcov is fresh, else skip loudly_ | Editing any source stales the lcov, so it skips on the commits that matter; it catches the free case only. `just check-full` is the arm that is always correct. |

`fmt-check`’s non-Rust path lists:

- `JUST_FMT_PATHS`: `.just/docs.just`
- `TOML_FMT_PATHS`: `.mindtape/config.toml`, `examples/learn-piano/.mindtape/config.toml`, `examples/large/.mindtape/config.toml`
- `TYPST_FMT_PATHS`: `lib`, `truth/tasks`, `.mindtape/templates`, `examples/learn-piano/tasks`, `examples/learn-piano/.mindtape/templates`, `www/content`, `www/templates`, `www/utils`, `docs/src`

Every arm runs through `mmz`, so a re-run skips each one whose declared inputs are unchanged and the pre-commit hook can assert the whole set fresh without re-running it. That memoization, the gated example projects and the locked formatter gates are in [docs/contributing/gates.md](docs/contributing/gates.md).

`just check-full` is `just check` plus `just cover` and `just crap`: always correct about coverage and CRAP, at the price of an instrumented rebuild and a full suite pass. Run it before opening a PR, plain `just check` while iterating.

A delegated worktree whose full aggregate runs at integration can scope the gate with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which demands a matching `Gate-Scope:` trailer so the commit never reads as fully gated. Dodging a gate that is actually red is never legitimate — when scoping one is, and what the trailer buys, are in [docs/contributing/gates.md](docs/contributing/gates.md).

Coverage and CRAP run outside `just check`. Run `just cover` first — `just crap` refuses a stale lcov — and `just crap-baseline` in the same commit as any deliberate complexity. The thresholds and the argument behind every number are in [docs/contributing/coverage-crap.md](docs/contributing/coverage-crap.md). Read it before changing either recipe’s flags.

`just nix-build` verifies `nix build` in a clean sandbox (no dev shell, no sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; `just release` calls it automatically.

## Signal Sweep (`just ask-refactoring`)

The sweep reads the _advisory_ quality signals and files a backlog task per finding. It is not a gate: a gate stops you at the wall, the sweep names the wall you are walking toward. Run `just ask-refactoring` to see what it would file today. The arms, the scoring and the design rules are in [docs/contributing/signal-sweep.md](docs/contributing/signal-sweep.md).

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) couples source files to the dependents that must stay in sync (e.g. the CLI surface to `README.md`). After editing a source, update the listed dependents, re-record the group you touched with `just outdatty-update <group>` and commit `outdatty.lock`. A recorded hash is a review watermark, so blanket re-recording — `just outdatty-update-all` — claims a review that never happened. It stays the explicit choice, never the default.

Re-record the group when you edit a dependent on its own, too, in the same commit. Only a source change is a trigger, so the gate stays green either way — but the dependent’s recorded hash then sits stale until some unrelated source edit carries it along, putting the watermark on a commit that never touched the file. It costs one lock line, and it is a convention rather than a gate. `outdatty.yaml`’s header carries the measurement, the reasoning and a one-liner listing groups with a dependent still pending.

## Version Management

- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates. Several files repeat it and all have to move together; the `typst-pkg-version` outdatty group enumerates them and fails the build otherwise. Trust the group, not a list here.
- `mindtape-cli`’s dependency on `mindtape` carries a version requirement (not path-only), because crates.io rejects path-only deps on publish; bump it with the workspace version.
- A release (`just release VERSION`) is gated twice. `VERSION` must be a bare semver with no leading `v`: an `[arg(...)]` pattern rejects anything else before the recipe body runs, so `v0.3.0` fails fast with a pattern mismatch instead of a confusing “Cargo.toml is 0.3.0” error from inside the script. Then the body gates — the tag must equal the workspace version, the working tree must be clean, and `CHANGELOG.md` may not have a non-empty `[Unreleased]` section (it moves into a dated `[VERSION]` section at release time). The recipe pushes `main` before the tag, so the docs site (`pages.yml`) and CI run on the release tree.

## Commit Messages

[Conventional Commits](https://www.conventionalcommits.org/). Match the language of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: truth/tasks/<stem>.typ
```

MindTape tracks its own backlog as per-file task artifacts under `truth/tasks/` (dogfooding itself — see `.mindtape/config.toml`). Point `Refs:` at the driving task file. Architectural and tooling decisions are task artifacts too; the ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config in `.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs nothing.

## Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never mutate the working tree; only explicit write commands do.

- Read (pure): `ls`, `find`, `graph`, `views`, `worktrees`, `check`, `eval`, `config show`/`schema` — each leaves the tree byte-identical, asserted per command in `crates/mindtape-cli/tests/read_no_write_e2e.rs`. Add your command’s test there, not to this list.
- Write (explicit): `add`, `import`, `init`, `install`, `preset save`, `flip`, `recur`, `review`, `rm`. Every legend value is also a top-level verb over `flip` — `mt done` is `mt flip done`, generated from the legend rather than declared as a subcommand.
- `go` is neither: it hands the file to your editor and mutates nothing itself.

Recurrence never materializes as a side effect of a read: an instance is spawned or refreshed only by an explicit `mt recur`, and `mt done` closes a task without creating or advancing anything.

“Byte-identical” means the _authored_ tree — task files and config. A read may still populate the git-ignored `.mindtape/cache/` scratch, derived state the way `target/` is.

## Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args` struct and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR process is retired).
