#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Isolate the interactive pty tests from concurrent load",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  estimate: (files: 3),
  tags: ("found", "tests"),
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
      sound-codebase: helps[a red suite means a broken change rather than a busy machine],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 22),
)

== Problem

+ Running \`just check\` in one worktree while three delegated agents ran their own suites on the same machine.
+ 🔴 #raw("crates/mindtape-cli/tests/interactive_e2e.rs"): Two pty tests fail under concurrent machine load and pass in isolation, so a green suite depends on nothing else running.

```
FAIL [   2.371s] (1570/2592) mindtape-cli::interactive_e2e opening_a_task_still_works_with_hyperlinks_active
FAIL [   2.379s] (1574/2592) mindtape-cli::interactive_e2e opening_a_task_suspends_the_tui_and_runs_the_editor
panicked at crates/mindtape-cli/tests/interactive_e2e.rs:254:5:
the editor closure must run inside suspend
```
Re-running the same two by name on the same tree, unchanged: `2 tests run:
2 passed`. A third run of the full suite, once the other agents had
finished, passed all 2592.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/tests/interactive_e2e.rs").

== TO-BE

- 🟢 The two tests wait on the TUI reaching the state they assert instead of
  on wall-clock time, so a loaded machine makes them slower and not red.
- 🟢 Running the full suite beside three other builds passes, repeatedly.
- 🟢 If a bound is genuinely needed, it is a named constant with the reason
  beside it, not a literal buried in the assertion.


== Changes

- `crates/mindtape-cli/tests/interactive_e2e.rs` — replace whatever fixed
  wait these two depend on with a poll until the expected frame appears,
  bounded by a generous timeout that fails with the frame it last saw.
- `crates/mindtape-cli/tests/support/` — if the poll is worth sharing, it
  belongs there; `check-dry` fails on a second copy.
- `docs/src/contributing/windows.typ` — these are the Unix-only pty tests it
  already describes; say they must not depend on machine load.


== Rollback

Revert the test file. The change is test-only, so nothing shipping depends
on it and no consumer can observe the difference.


== Out of scope

- The TUI itself, which is not what failed; the assertions raced it.
- The other four tests in the same file, which did not flip.


== Dogfood

This repo runs several delegated worktrees at once on purpose, so the
concurrent case is the normal case here. The next parallel wave is the
test: a red arm must mean a real break.

