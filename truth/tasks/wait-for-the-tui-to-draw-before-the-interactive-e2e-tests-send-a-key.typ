#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Wait for the TUI to draw before the interactive e2e tests send a key",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
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
      sound-codebase: helps[a gate no longer fails on machine load alone],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Running \`just check\` on a machine under load while landing the doc gates
+ 🔴 #raw("crates/mindtape-cli/tests/interactive_e2e.rs"): quitting\_restores\_the\_terminal sends ESC after a fixed 300 ms sleep, so it fails when the machine is loaded and passes alone

```
FAIL mindtape-cli::interactive_e2e quitting_restores_the_terminal
mt did not exit within 5s   (load average 41; two of two runs)
$ just test quitting_restores_the_terminal   # load 14
1 test run: 1 passed   (three of three runs)
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/tests/interactive_e2e.rs").

== TO-BE

- 🟢 The test sends ESC after the screen-enter bytes appear in the captured output, and passes under load.

== Changes

- `crates/mindtape-cli/tests/interactive_e2e.rs`: replace the fixed sleep with a read that waits for the alternate-screen bytes, bounded by the existing timeout.

== Rollback

- Revert the test change.

== Out of scope

- The other tests in the file, unless they share the sleep.

== Dogfood

Every parallel `just check` in this repo stops failing this test on a busy machine.
