#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add in-process CLI tests to harden coverage",
  status: done(2026, 6, 14),
  priority: "normal",
  estimate-hours: 4,
  tags: ("tests", "cli"),
)

== Summary

The CLI command runners (`check` / `init` / `eval` / `util` / `list`) are
effectively uncovered: `assert_cmd` subprocess tests don't count toward
`cargo-tarpaulin`, so coverage clears the 70% gate by only ~1 point.

== Scope

- Drive the command `run` functions in-process against a temp project fixture.
- Assert exit behaviour and output without spawning a subprocess.
- Raise the coverage margin so the gate isn't fragile.
