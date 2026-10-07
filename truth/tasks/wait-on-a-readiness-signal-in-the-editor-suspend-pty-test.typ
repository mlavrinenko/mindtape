#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Wait on a readiness signal in the editor-suspend pty test",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tests"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a full just check stays green while worktrees build beside it],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
)

== Problem

+ Running a full \`just check\` while several worktrees built at once.
+ 🔴 #raw("crates/mindtape-cli/tests/interactive_e2e.rs"): \`opening\_a\_task\_suspends\_the\_tui\_and\_runs\_the\_editor\` fails under load on the fixed 5s wall-clock deadline \`WAIT\_TIMEOUT\` and passes alone; the done task \`isolate-the-interactive-pty-tests-from-concurrent-load\` fixed a different assertion in the same tests.

```
mt did not exit within 5s
```
`crates/mindtape-cli/tests/interactive_e2e.rs:32`: `const WAIT_TIMEOUT: Duration = Duration::from_secs(5);`
`crates/mindtape-cli/tests/interactive_e2e.rs:207`: `assert!(!timed_out, "mt did not exit within {WAIT_TIMEOUT:?}");`
Re-run alone, the test passes.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/tests/interactive_e2e.rs").

== TO-BE

- 🟢 The pty tests wait for the editor's marker or the child's exit with a generous ceiling, so machine load cannot fail them.

== Changes

- `crates/mindtape-cli/tests/interactive_e2e.rs`: replace the 5s `WAIT_TIMEOUT` poll with a wait on a readiness signal, or raise the ceiling.

== Rollback

- Revert the test change.

== Out of scope

- The product code the test drives. It is not at fault.

== Dogfood

Every full `just check` here runs these tests beside other builds.
