#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold on.flip commands to the same approval as waits-on checkers",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate the user never allowed refuses the flip instead of running or passing],
      cheap-adoption: hurts[a fresh clone asks once before `mt done` runs its gate],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
  links: depends-on(
    "run-no-waits-on-checker-until-the-user-allows-the-commands-the-config-runs.typ",
  ),
)

== Problem

+ Clone `examples/api-service`, whose `[[on.flip]]` runs `scripts/test` into `done`.
+ 🔴 Run `mt done reject-expired-sessions-on-token-refresh`. It runs `scripts/test` before you have read the config.


== TO-BE

- 🟢 `mt allow` covers every `run` in `[[on.flip]]` too, in the same hash, so one approval holds every command the config runs.
- 🟢 Un-allowed, a `run` action does not run: a gate refuses the flip with the allow hint, `--force` waives it as it waives a failing gate, and a non-gating `run` prints that it was skipped.
- 🟢 The skill tells an agent never to run `mt allow` itself, only to pass the warning on.


== Changes

- `crates/mindtape-cli/tests/it/flip_hooks_e2e.rs`: red first: an un-allowed gate refuses, `--force` passes, an allowed one runs.
- `crates/mindtape/src/allow.rs`: hash the `[[on.flip]]` `run` lines beside the checkers.
- `crates/mindtape-cli/src/cmd/flip/hooks.rs`: check the approval before `run_command`.


== Alternatives

- Skipping an un-allowed gate: the flip would pass a check that never ran.
- A second approval for flips: two prompts for one trust decision.


== Out of scope

- `[go] editor`, which runs the editor you configured yourself.


== Dogfood

This repo's `[[on.flip]]` gates run once `mt allow` records them; `.config/wt.toml` needs no change, since forks share the approval through the git common dir.
