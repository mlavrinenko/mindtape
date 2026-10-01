#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail the gate when TOML_FMT_PATHS or a plain mt build drifts",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a drifted fmt list or a relinking build fails by name],
      docs-match-code: helps[two hand-kept facts become gate-checked],
    ),
  ),
  kind: "dev",
  assignee: "agent",
  status: done(2026, 10, 2),
)

== Problem

+ 🔴 Add `tmp-probe.toml` (untracked) at the repo root and run `just check`: it passes. `TOML_FMT_PATHS` in the `Justfile` is a hand-kept list, so `tombi format --check` never reads the new file and the list drifts from `git ls-files` unseen (`cd8a4e51` had to enumerate every tracked TOML by hand).
+ 🔴 Add `cargo build -p mindtape-cli -q` to any gate-reachable script: `just check` stays green, yet a plain build relinks `target/debug/mt` under the running test arm (`32d79fd9`, `docs/contributing/gates.md` "One mt binary, one feature set"); the race fires only on a stale tree, so it returns unseen.


== TO-BE

- 🟢 A TOML file git knows (cached or untracked, not ignored) that `TOML_FMT_PATHS` lacks, or a listed path git does not know, fails the gate naming each path.
- 🟢 A `cargo build` or `cargo run` of `mindtape-cli` into the shared `target/debug` without `--tests` fails the gate naming the file or recipe and line; `--release` and `--target-dir` are exempt by flag, never by file.


== Changes

- `.just/scripts/check-gate-drift.sh`, `_gate-drift-lib.sh` (TOML list against git; the `cargo build/run mindtape-cli` line matcher), `_gate-drift-control.sh`, new. The reach out of `check` is not walked here: `_script-graph.sh` is split into `_script-graph-lib.sh` (the shared walk) and a thin front-end, and `_script-reach.sh` is the second front-end. A control that must pass a clean fixture and name the offender in each known-bad one; red first. Split to stay under the 300-line shell cap.
- `Justfile`: `check-gate-drift` recipe, wired into `check`.
- `.mmz/config.yaml`: scopes and a gate-tagged rule for the arm.
- `outdatty.yaml`, `outdatty.lock`: the script joins `dev-docs`, re-recorded after reviewing its dependents.
- `www/utils/gate-notes.typ`, `docs/src/contributing/gates.typ`: the arm's prose; `CONTRIBUTING.md` and `docs/contributing/gates.md` regenerated.

== Rollback

Drop the recipe, its `check` entry, the script, the mmz scope and rule and the outdatty line, then regenerate the docs.


== Out of scope

- `just check-rrule`: it is not a `just check` arm, so the reach derivation leaves it out; it builds a different feature set into `target/debug` on purpose and CI runs it serially. `docs/src/contributing/gates.typ` said it belongs in another target directory, which was wrong and is corrected.
- Rewriting `TOML_FMT_PATHS` to derive from git: the explicit list is the review point, the gate only keeps it honest.


== Dogfood

Every `just check` in this repo, and every delegated worktree's gate.
