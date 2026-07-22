#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "pin TUI teardown under the PTY harness",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 6,
  ),
  tags: ("cli", "tests"),
  links: related("collapse-branches-to-drop-mutant-exclusions.typ")[
    the other half of the exclusion burn-down
  ],
  status: done(
    2026,
    7,
    21,
  )[PTY teardown + suspend cases added; tui.rs 4 caught 1 unviable 0 missed at -j2; both exclusions dropped, exclude\_re 10 → 2],
)

== Summary

Two `exclude_re` entries claim the TUI teardown is unobservable: `Tui::drop`
"calls leave_screen which Err-s without a TTY", and `leave_screen` "succeeds
without a TTY, same as returning Ok(())". Both reasons assume no terminal. The
repo has had one since `tests/tty_e2e.rs` — `script(1)` from util-linux, already
in the devShell. Under a PTY, `leave_screen` writes `LeaveAlternateScreen` and
`cursor::Show` to a real terminal, which is directly observable in the captured
transcript. That is the same stale-assumption class already corrected six times
in the previous pass.

== Scope

- Extend `crates/mindtape-cli/tests/interactive_e2e.rs` (or `tty_e2e.rs`) with a
  case that runs `mt ls --interactive` under the PTY, quits, and asserts the
  captured stream contains the alternate-screen *leave* and cursor-show
  sequences after the enter sequences — i.e. the terminal was actually restored.
- Confirm the assertion fails for both mutants: `Tui::drop` with an empty body,
  and `leave_screen` replaced by `Ok(())`. `suspend`'s leave/enter pair gives a
  second observation point if the drop path proves awkward.
- Drop both `exclude_re` entries from `.cargo/mutants.toml`.
- Fold in the two `ls_interactive/tui_tests.rs` tests that currently drive the
  developer's real terminal (tracked in `isolate-global-state-in-tests.typ`) —
  once the PTY case exists, they have no reason to run in-process.

== Verification

`cargo mutants -f crates/mindtape-cli/src/ls_interactive/tui.rs` without
`--iterate`; both mutants must come back `caught`, not `unviable` and not
`timeout`. Run the final sweep at `-j2` or lower, per the TIMEOUT finding in the
parent task.
