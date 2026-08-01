#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "stop tty sensitive tests from flipping with the caller",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tests",),
  estimate: (files: 4),
  status: done(2026, 8, 1)[just test now passes piped and on a real tty],
)

== Summary

`ls_interactive::guard` read `is_terminal()` inside itself, so `guard_table_fails_on_tty_not_format` asserted a property of the caller, not the code. Piped — CI, and `just check`, which runs the test arm under `parallel` — the guard refused and the test passed. Run straight from a terminal, `cargo test` inherits the tty, the guard allowed, and the same test panicked on `unwrap_err()`. That is the whole of the reported "just check is green but just test fails": nothing to do with mmz freshness, just stdout.

== Scope

- `guard` splits into `guard_with(args, tty)`; the ambient `is_terminal()` read stays in the one-line production wrapper, and the tests pass the answer in. Adds the missing third case, table + tty allows
- same fix for `render::table::glyph_mode`, which had already hit this and worked around it by re-deriving its expectation from `is_terminal()` — that made the test re-implement the function it covered, so it could not fail either way
- production behaviour is unchanged in both; this is test reachability only
- the pattern to keep: production reads the ambient terminal, the unit under test takes it as an argument. `add.rs`, `add_interactive.rs` and `init.rs` already did this
