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
| `fmt-check` | _Check formatting without writing changes (CI-friendly)_ | `cargo fmt` over Rust, `tombi format` over the tracked TOML (`.gitleaks.toml`, `.mindtape/config.toml` and the per-example configs), `typstyle` over the Typst sources, and `--fmt` over the justfiles — the exact path lists are below the table. One list feeds both `fmt` and `fmt-check`, so a path is added in one place, never to a single recipe. The justfiles need a list of their own because a bare `--fmt` formats the root `Justfile` only and silently skips every `mod` file; each file listed gets its own `--justfile <file> --fmt` pass and needs `set unstable := true` of its own, since `--fmt` reads that setting from the file it is pointed at, not from the `Justfile` declaring the `mod`. |
| `clippy` | `cargo clippy --workspace --all-targets -q -- -D warnings` | Every lint is workspace `deny` (see Code Style above); this recipe is only the enforcement point, not a separate lint surface. |
| `test` | `cargo nextest run --workspace --status-level fail --cargo-quiet --cargo-quiet "$@"` | The full workspace suite, run through nextest: one process per test, so a panicking or hanging test is isolated, and `--status-level fail` keeps a green run to a header and a summary — only a failure prints, per test, rather than every binary’s own result block. nextest does not run doctests, and there is no separate `cargo test --doc` pass: the tree has none (`cargo test --doc` reports 0 tests in both crates), so a doctest added later must come with a pass of its own. `just test <filter>` (a developer query, no `mmz`) runs this recipe directly with the filter as `ARGS`, so a filtered run never pollutes the cached full-suite record `just check` relies on. |
| `machete` | `cargo machete` | Flags a workspace dependency no crate actually imports. |
| `check-file-size` | `linecop --format paths` | Per-language caps — and the tighter ones on the always-loaded docs — are in [`.linecop.yaml`](.linecop.yaml), argued in place. `linecop --baseline 85` lists what is nearing its cap; run it before adding to an existing file. |
| `check-dry` | _Copy-paste gate: fail on any duplicated Rust block_ | `jscpd` — fails on any duplicated Rust block ≥70 tokens; fix by extracting a shared helper. |
| `check-secrets` | _Fail if the working tree carries a committed credential_ | gitleaks over the working tree under `.gitleaks.toml` — its default ruleset, a walk bound naming the gitignored roots (an unbounded scan walks `target/`: 1m44s over 2.09 GB, against 1.5s over 7.5 MB), and one allowlist for the `outdatty.lock` digests `generic-api-key` reads as auth secrets. History is out of scope: `gitleaks git` reports the absolute paths this repo has already committed and would need a `.gitleaksignore` entry per past mint. A control fixture is scanned first and must report an exact set of findings, because “no leaks found” is equally what an over-broad allowlist and a config that loaded no rules print — that one assertion pins the ruleset being live, both allowlists firing, each staying scoped to the rule and to the shape it silences rather than to the whole file, and none being wide enough to swallow the tree. A third arm set-differences `git ls-files` against mmz’s own expansion of this rule’s inputs, so the enumeration that stands in for “the whole tree” (`.mmz/config.yaml` has no negative glob, and `truth/tasks/**` may not be named by a gate-tagged rule) fails by name rather than going quietly false-fresh. A fourth arm (`.just/scripts/_gitleaks-bound-check.sh`, run against a synthesized control pair first for the same false-green reason) holds the walk bound to BOTH `.gitignore` files: every anchored entry either appears in the bound or carries a `# gitleaks-bound: exempt — <why>` line above it, and no bound pattern may match a file git tracks. Uncovered scratch has the walk descend into a tree a parallel arm is rewriting; an over-wide pattern silences a committed file, which is the only way an allowlist can cost coverage. Two entries are exempt and say so where they live — `/.cargo/*`, whose tracked `!/.cargo/mutants.toml` exception RE2 has no lookahead to spare, and `/result`, a nix symlink gitleaks never follows. The bound stops there — the `lstat` of the bounded directory itself precedes the allowlist check, so the scan holds the `target/www-generated.lock` flock across its walk rather than trusting the bound to stop the `ERR` a concurrent `rm -rf` logs on a run it still exits 0, noise that teaches the reader to skim a security gate. What it cannot do is catch a machine-local path — that is `just crap-baseline-check`’s job, argued there. See `.just/scripts/check-secrets.sh`. |
| `outdatty-check` | `outdatty check --format paths` | See Dependency Drift below for how `outdatty.yaml` couples a source to the dependents it must stay in sync with. |
| `check-examples` | _Validate each example project against itself_ | `mt check` against each example project: `examples/learn-piano/` (full config surface; feeds the docs generator and the skill’s triage evals) and `examples/minimal/` (the floor config; feeds the skill’s trap evals). Keep the recipe and the mmz `examples` scope in step. A fixture no gate validates rots silently. |
| `check-doc-coverage` | _Fail if a visible mt subcommand has no CLI doc entry_ | Set-differences `mt _cli --json`’s reflected command names against `www/utils/cli-notes.typ`’s dict keys; fails naming any visible subcommand with no matching entry, so a new verb can no longer ship silently undocumented. |
| `check-doc-facts` | _Fail if a derived crate, DSL field, gate, or sweep flag fact has no doc entry_ | Regenerates `www/generated/{crate-map,dsl-fields,gates,sweep-flags}.json` (`www/generate-facts.sh`) and set-differences each against its `www/utils/{crate-notes,dsl-field-notes,gate-notes,sweep-flag-notes}.typ` prose dict; fails naming any crate, DSL field, gate, or sweep flag with no matching entry — this file included. |
| `check-doc-recipes` | _Fail on a hand-typed just recipe mention outside the generated module_ | Fails on a hand-typed recipe mention in a doc source — literal `just` followed by a name, whether or not the name is real — that should have gone through the generated module (`www/generated/just.typ`, called as `#just.check()` and friends) instead. The three literal shapes it scans for (a prose backtick span, a fenced example’s leading command, a quoted config value) are argued in place in `.just/scripts/check-doc-recipes.sh`. |
| `docs-check` | _Build, index, and validate the docs site_ | `docs-gen` — a dependency — captures live `mt` output first (`www/generate.sh`), so this gate always sees fresh CLI examples before `pagefind` indexes the build for client-side search and `tola validate` checks links and assets. Both `tola` steps print only on failure. |
| `docs-md-check` | _Fail if generated Markdown has drifted from docs/src_ | Regenerates every `docs/src/*.typ` source with typlite into a temp directory and diffs each against the committed Markdown it declares; fails naming any drifted file rather than healing it in place. `just docs md` refreshes it — see [docs/contributing/generated-docs.md](docs/contributing/generated-docs.md). |
| `fmt-fixture-check` | _Lock the formatter gates against a mis-formatted fixture_ | Synthesizes a deliberately mis-formatted TOML and Typst fixture in a temp directory and asserts `tombi format --check` and `typstyle --check` both reject it (and accept a reformatted one); locks the formatter gates themselves, so a formatter silently accepting drift could not let `fmt-check` pass on an out-of-sync editor save. |
| `arg-pattern-check` | _Lock the arg-attribute patterns against known-bad and known-good input_ | Dispatch-only self-test for the `[arg(...)]` patterns on `just release VERSION`, `just bench N`, `just gen-bench N`, `just eject PCT` and the five `just ask-refactoring` thresholds: asserts each rejects a known-bad argument and accepts a known-good one, and that every declared default satisfies its own pattern (`fix-check` depends on bare `eject`; the sweep’s defaults are the only copy of its thresholds). One of the sweep’s cases is a value containing a space — the word-splitting that `[positional-arguments]` used to guard that recipe against, refused at dispatch now instead. Every assertion goes through the `--dry-run --yes` flag pair, so no recipe body ever actually runs — `release`’s included; see `.just/scripts/arg-pattern-check.sh`. |
| `positional-args-check` | _Lock positional-argument forwarding on the four passthrough recipes_ | Static self-test for the four passthrough recipes — `just test <filter>`, `just build <args>`, `just mutants <args>` and `just mutants-diff <args>` — asserting each carries `[positional-arguments]` and forwards `"$@"` rather than interpolating `{{ ARGS }}` into its body, which would re-split an argument containing a space and re-glob one containing a wildcard. Reads the `--dump --dump-format json` dump and greps the scripts the recipes forward into; no recipe body runs. See `.just/scripts/positional-args-check.sh`. |
| `script-attr-check` | _Lock the script attribute and each justfile's strict-mode interpreter_ | Strict-mode self-test in two arms. No recipe body may open with a `#!` shebang — every script body reaches the interpreter through `[script]`, so `set script-interpreter` applies to it — and every justfile must declare a setting that actually aborts a failed pipeline. `shebang` in the dump cannot tell the two apart (a `[script]` recipe reports `true` exactly as a `#!` body does, measured on just 1.46), so the first arm pairs that field with the absence of a `script` attribute. The second runs a `false | true` probe under a replica justfile carrying the file’s own setting line verbatim, rather than matching its text: `-euo pipefail` would fail a literal match while being strictly better, and `-e` alone would pass a looser one while dropping the exact flag that let `just count-tests` report a partial sum and exit 0. Every justfile is checked, not only those with `[script]` recipes today — a module does not inherit the root’s setting, so `.just/docs.just`’s currently userless one is load-bearing and cannot be tidied away; see `.just/scripts/script-attr-check.sh`. |
| `size-fixture-check` | _Lock the size gate's scan scope against an oversized hidden-dir fixture_ | Scan-scope self-test for `just check-file-size`: an oversized shell script at `.just/scripts/` inside a replica tree must be reported by `linecop` reading this repo’s own `.linecop.yaml`, and must go unreported with `include_hidden` deleted from it. The caps need no self-test — a wrong number is visible — but a directory the walker never enters reports the same “All files within size limits.” a clean tree does, which is how every recipe body extracted into `.just/scripts/` sat uncapped while the extraction’s stated reason was bringing them under a cap. Runs against a replica rather than the real directory because `just check` runs its arms in parallel and a probe in the tracked tree would fail the real size gate mid-run; see `.just/scripts/size-fixture-check.sh`. |
| `script-scope-check` | _Lock every gate script into a named mmz scope and outdatty's dev-docs group_ | Enumeration lock over the two manifests that list `.just/scripts/` file by file, in three arms. Every `.just/scripts/*.sh` must be named by some `.mmz/config.yaml` scope as a literal path; every memoized rule whose recipe body REACHES a script must declare a scope covering it — the second because a script listed in a scope no rule reads passes the first while still serving a stale hit; and every script must be an `outdatty.yaml` `dev-docs` source or carry an exemption naming the reason no doc restates it. It exists because those scopes replaced a `.just/scripts/**` glob every rule carried, and the glob had the property an enumeration cannot have: a recipe body extracted out of the Justfile was covered by arriving. A wildcard never counts as naming a script, so putting the glob back satisfies nothing; and the third arm demands a DECISION rather than membership, because `_nav-diff.sh` is outside that group on purpose and forcing it in would ask for a review of seven dependents no change to it can rot. Arm two walks a call graph (`.just/scripts/_script-graph.sh`), so a body reached one hop further counts: through another script (check-secrets.sh’s two gitleaks-bound scripts) and through a nested `just` (check-crap.sh shelling out to `just crap`) alike, both being the same edge into the next body. It claimed direct invocation only until four such scopes had been wired by hand, which is what a missing derivation looks like from the outside. A wrong edge fails a CORRECT manifest, so the reading is narrow where it can over-read — a `just` counts only at command position, never inside the `echo` hints these scripts use to name a recipe they do not run — and loud where it cannot read at all: an unclassified flag or a target that is not a bare recipe name fails rather than being skipped, since a skipped call is the silent miss the walk exists to remove. `.just/scripts/_script-graph-control.sh` drives it over a synthetic tree carrying every one of those shapes at once, both directions being invisible from the real tree, where a correct answer and a broken reader print the same sentence. See `.just/scripts/script-scope-check.sh`. |
| `site-pages-fixture-check` | _Lock the docs manifest gate against three known-bad fixtures_ | Locks the docs manifest gate itself: two pages declaring the same route, a page with no `<page-meta>` block, and a page missing from `www/utils/site.typ`’s NAV were each demonstrated by hand once, when they landed, and nothing since has proven they still fire. Runs a temp-tree replica’s own copy of `www/generate-site-pages.sh` for the first two (never the working tree — `just check`’s arms run in parallel, so a mutation there would be read as real by a concurrent gate), and pins the third by calling `.just/scripts/_nav-diff.sh` directly against a synthetic pair of route lists rather than driving a whole `just check-doc-facts` regenerate — that script is the extracted set-difference `just check-doc-facts` itself now calls, so the thing this gate pins is the thing that actually runs. A positive control asserts the unmutated replica passes, and every case asserts both a non-zero exit and that the failure names the offending page or route, never the exit code alone. It regenerates `www/generated/` itself first (`www/generate.sh`, inside the flock, the snapshot taken before the lock drops) rather than reading what the parallel `just docs gen` arm may have left: that directory is gitignored, so on a cold checkout the replica had no captures to compile against and every case failed for want of them — indistinguishable, on the run that decides, from the gate catching a real break. The replica is copied entry by entry so it can skip `www/public/` and `www/.tola/` — gitignored build outputs no page compile reads, both written by the parallel `just docs check` arm, so copying them made this gate a reader of a tree another arm rewrites underneath it for 8 MB of nothing. An assertion beside the copy fails the gate if either reappears. See `.just/scripts/site-pages-fixture-check.sh`. |
| `check-crap` | _Run the CRAP gate when the lcov is fresh, else skip loudly_ | Editing any source stales the lcov, so it skips on the commits that matter; it catches the free case only. `just check-full` is the arm that is always correct. |
| `crap-baseline-check` | _Fail if the CRAP baseline is empty or carries machine-local paths_ | The CRAP arm that never skips: it reads the committed baseline itself rather than the lcov, so no coverage run has to be fresh for it to be right. It fails on an entry carrying a machine-local absolute path — what `just crap-baseline` strips after minting, and what a re-mint by hand puts straight back — and on a baseline scoring no functions at all, which passes the ratchet by reporting every function as new. |

`fmt-check`’s non-Rust path lists:

- `JUST_FMT_PATHS`: `.just/docs.just`
- `TOML_FMT_PATHS`: `.gitleaks.toml`, `.mindtape/config.toml`, `examples/learn-piano/.mindtape/config.toml`, `examples/large/.mindtape/config.toml`
- `TYPST_FMT_PATHS`: `lib`, `truth`, `.mindtape/templates`, `examples/learn-piano/tasks`, `examples/learn-piano/.mindtape/templates`, `www/content`, `www/templates`, `www/utils`, `docs/src`

Every arm runs through `mmz`, so a re-run skips each one whose declared inputs are unchanged and the pre-commit hook can assert the whole set fresh without re-running it. That memoization, the gated example projects and the locked formatter gates are in [docs/contributing/gates.md](docs/contributing/gates.md).

`just check-full` is `just check` plus `just cover` and `just crap`: always correct about coverage and CRAP, at the price of an instrumented rebuild and a full suite pass. Run it before opening a PR, plain `just check` while iterating.

`just fork <slug> <base>` stands one of those worktrees up: a new branch off `<base>` (default `HEAD`) at `../mindtape-<slug>`, `direnv allow`-ed and with its dev shell already evaluated, and the base read out of the created worktree and compared to the sha it was created from. It refuses a dirty `HEAD` — `git worktree add` copies commits, never the working tree, so forking off one would hand the agent a base missing the changes you are looking at while typing the command — and it reports, without refusing, a base behind or diverged from this checkout. It warms no build cache, deliberately; what was measured about that, and what the base check can and cannot prove, are argued in `.just/scripts/fork.sh`.

A delegated worktree whose full aggregate runs at integration can scope the gate with `MINDTAPE_GATE_SCOPE=<reason> git commit`, which demands a matching `Gate-Scope:` trailer so the commit never reads as fully gated. Dodging a gate that is actually red is never legitimate — when scoping one is, and what the trailer buys, are in [docs/contributing/gates.md](docs/contributing/gates.md).

Coverage and CRAP run outside `just check`. Run `just cover` first — `just crap` refuses a stale lcov — and `just crap-baseline` in the same commit as any deliberate complexity. The thresholds and the argument behind every number are in [docs/contributing/coverage-crap.md](docs/contributing/coverage-crap.md). Read it before changing either recipe’s flags.

`just nix-build` verifies `nix build` in a clean sandbox (no dev shell, no sccache). Run it when touching `flake.nix` or `.cargo/config.toml`; `just release` calls it automatically.

## Signal Sweep (`just ask-refactoring`)

The sweep reads the _advisory_ quality signals and files a backlog task per finding. It is not a gate: a gate stops you at the wall, the sweep names the wall you are walking toward. Run `just ask-refactoring` to see what it would file today. The arms, the scoring and the design rules are in [docs/contributing/signal-sweep.md](docs/contributing/signal-sweep.md).

## Dependency Drift (outdatty)

[`outdatty.yaml`](outdatty.yaml) couples source files to the dependents that must stay in sync (e.g. the CLI surface to `README.md`). After editing a source, update the listed dependents, re-record the group you touched with `just outdatty-update <group>` and commit `outdatty.lock`. A recorded hash is a review watermark, so blanket re-recording — `just outdatty-update-all` — claims a review that never happened. It stays the explicit choice, never the default.

Re-record the group when you edit a dependent on its own, too, in the same commit. Only a source change is a trigger, so the gate stays green either way — but the dependent’s recorded hash then sits stale until some unrelated source edit carries it along, putting the watermark on a commit that never touched the file. It costs one lock line, and it is a convention rather than a gate. `outdatty.yaml`’s header carries the measurement, the reasoning and a one-liner listing groups with a dependent still pending.

## Version Management

- No backward compatibility before the first public release: a rename lands whole, callers and configs included, with no alias or migration path left.
- Workspace crates: `Cargo.toml` `[workspace.package] version` (semver).
- Typst library: `lib/typst.toml` `version`, independent of the crates. Several files repeat it and all have to move together; the `typst-pkg-version` outdatty group enumerates them and fails the build otherwise. Trust the group, not a list here.
- `mindtape-cli`’s dependency on `mindtape` carries a version requirement (not path-only), because crates.io rejects path-only deps on publish; bump it with the workspace version.
- A release (`just release VERSION`) is gated twice. `VERSION` must be a bare semver with no leading `v`: an `[arg(...)]` pattern rejects `v0.3.0` before the recipe body runs. Then the body gates — the tag must equal the workspace version, the working tree must be clean, and `CHANGELOG.md` may not have a non-empty `[Unreleased]` section (it moves into a dated `[VERSION]` section at release time). The recipe pushes `main` before the tag, so the docs site (`pages.yml`) and CI run on the release tree.

## Commit Messages

[Conventional Commits](https://www.conventionalcommits.org/). Match the language of prior commits. Footer with a task reference:

```
feat(filter): evaluate expressions over live tasks

Refs: truth/tasks/<stem>.typ
```

MindTape tracks its own backlog as per-file task artifacts under `truth/tasks/` (dogfooding itself — see `.mindtape/config.toml`). Point `Refs:` at the driving task file. Architectural and tooling decisions are task artifacts too; the ADR process is retired.

Commits that file or edit task artifacts themselves (and the backlog config in `.mindtape/config.toml`) carry no `Refs:` footer — a task commit refs nothing.

## Command Semantics (read/write separation)

MindTape follows a strict read/write split (CQRS). Read commands never mutate the working tree; only explicit write commands do. The split is declared once, in `crates/mindtape-cli/src/command_class.rs`, checked for exhaustiveness against `mt _cli --json`’s visible command list in `cli_dump.rs`’s own tests, and read here from `fact("cli")` rather than retyped.

- Read (pure): `ls`, `find`, `graph`, `views`, `worktrees`, `check`, `config show`, `config schema`, `preset ls`, `eval` — each leaves the tree byte-identical, asserted per command in `crates/mindtape-cli/tests/read_no_write_e2e.rs`. Add your command’s test there, not to this list.
- Write (explicit): `add`, `import`, `preset save`, `init`, `install`, `rm`, `review`, `flip`, `recur`, `link`. Every legend value is also a top-level verb over `flip` — `mt done` is `mt flip done`, generated from the legend rather than declared as a subcommand.
- `go` is neither: hands the file to the configured editor; mutates nothing itself.

Recurrence never materializes as a side effect of a read: an instance is spawned or refreshed only by an explicit `mt recur`, and `mt done` closes a task without creating or advancing anything.

“Byte-identical” means the _authored_ tree — task files and config. A read may still populate the git-ignored `.mindtape/cache/` scratch, derived state the way `target/` is.

## Extending MindTape

- New CLI command: add `crates/mindtape-cli/src/<name>.rs` with an `Args` struct and a `run` fn; wire it into `cli.rs`. See `ls.rs` for reference.
- New domain capability: add to `crates/mindtape` and re-export from its `lib.rs`. Keep `typst-harvest` consumer-blind — no task concepts there.
- Architectural/tooling decisions: file a task under `truth/tasks/` (the ADR process is retired).
