#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Give a failed hook action advice that matches the write",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a warning states what happened to the write, and the reader knows whether to re-run],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
)

== AS-IS

+ Declare a non-gating `[[on.rm]]` action whose command fails, and allow it with `mt allow`.
+ Remove a task with `mt rm --yes`.
+ 🔴 The removal goes ahead, and the warning says to run the command again:

```
$ mt rm fix-login-bug --yes
$ scripts/archive
  archive: no bucket configured
warning: scripts/archive (exit 1) — fix it, then run mt rm again
deleted 1 task:
  - tasks/fix-login-bug.typ
```

+ A non-gating `[[on.flip]]` action fails the same way, and its warning offers `--force` for a flip that already landed.

== Environment

- `mt` built from `main` at `978533e3`, on Linux.

== TO-BE

- 🟢 A failed action's advice says what happened to the write, and what to do next.
- 🟢 A gate that fails refuses the write, and its advice keeps the way past it: for flip, `fix it, or re-run with --force`. `mt rm` has no `--force`, and its gate refusal states the gate and the exit code.
- 🟢 A non-gating action that fails lets the write through, and its advice says so: `fix it, then run the command yourself; the removal is not blocked`.
- 🟢 A `message` on the action still replaces the advice, as it does today.

```
$ mt rm fix-login-bug --yes
$ scripts/archive
  archive: no bucket configured
warning: scripts/archive (exit 1) — fix it, then run the command yourself; the removal is not blocked
```

== Changes

- `crates/mindtape-cli/src/cmd/rm/hooks.rs`: `DEFAULT_ADVICE` is already the non-gate advice (`rm/hooks.rs:55`), so only its text changes. The gate path (`rm/hooks.rs:162`) keeps its `gate failed` refusal, which carries the project's `message` and no advice.
- `crates/mindtape-cli/src/cmd/flip/hooks.rs`: `DEFAULT_ADVICE` splits, because `runless_gate` (`flip/hooks.rs:245`) and the non-gate path (`flip/hooks.rs:206`) share it today. The gate keeps `--force`, and the non-gate path stops offering it.
- `crates/mindtape-cli/src/cmd/rm/hooks_tests.rs`, `crates/mindtape-cli/src/cmd/flip/hooks_tests.rs`: each path asserts its own advice.
- `crates/mindtape-cli/tests/it/rm_e2e/mod.rs`: the end-to-end warning text.
- `www/content/hooks.typ`: what a failed action says, for a gate and for a non-gating run.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Fixed`.

== Out of scope

- The `message` an action declares. It still replaces the advice, and a project with its own words keeps them.
- The exit code and the output a failed action prints. Both stay as they are.

== Dogfood

This repo's `[[on.flip]]` hooks are all gating, and its own flips keep working. The change shows up in the `examples/` fixtures and in any project that runs a non-gating action.
