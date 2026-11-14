#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "isolate global state in tests",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  tags: ("cli", "tests"),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that added these tests
  ],
  status: done(
    2026,
    7,
    21,
  )[cwd/XDG/tmp-path dependencies removed. Suite green at `--test-threads=1` and each suspect test alone. 71 mutants 65 caught 0 missed],
)

== Summary

Tests added during the mutation burn-down reach outside their own process
state. They set process-global `OnceLock`s that every sibling test then
inherits, and they assert on globals being *unset* (true only if they run
first). Some drive the developer's real terminal into raw mode, and one reads
the repo's own backlog from the current working directory. Each one passes today, and each is a latent flake that will fire on the next test
added nearby.

== Scope

- `crates/mindtape-cli/src/ls_interactive/tui_tests.rs` `suspend_executes_closure`
  builds a `Tui` directly and calls `suspend`, which runs `enable_raw_mode()`
  and writes `EnterAlternateScreen` to the test process's stdout. Under `cargo
  test -- --nocapture` on a real terminal, that hijacks the developer's screen.
  `enter_screen_fails_without_tty` does the same when a TTY is present, and its
  body is two conditional `panic!`s, so half of it is dead on a TTY-less run.
  Drive both through the PTY harness (`tests/tty_e2e.rs`) or drop them.
- `crates/mindtape-cli/src/util.rs` `init_worktree_and_override` and
  `crates/mindtape-cli/src/render/color.rs` `init_then_is_color_active` each
  assert a global is `None`/`false` *before* init, which makes them order-dependent,
  and then set it permanently for every other test in the binary. Move to a child-process
  test (the pattern `worktrees.rs` `run_child` already uses correctly).
- `crates/mindtape-cli/src/ls_interactive/tests.rs`
  `refresh_loads_tasks_from_project` resolves the root from the *current working
  directory*, Typst-evaluating the real `truth/tasks/` backlog, then asserts
  `!tasks.is_empty()`. Use a temporary-directory fixture.
- `crates/mindtape-cli/src/init_tests.rs`
  `with_schema_directive_prints_note_when_schema_missing` spawns a child without
  setting `XDG_CONFIG_HOME`, and asserts the schema is *missing*. It fails on any machine where the developer
  has run `mt install --schema`.
- `crates/mindtape-cli/src/go.rs` `spawn_editor_succeeds_with_true` spawns a real
  process against a hardcoded `/tmp/x` instead of a temporary directory.

== Verification

`just check`, plus `cargo test -p mindtape-cli -- --test-threads=1` and a run in
a random order (or with the suspect tests filtered to run alone) to confirm no
ordering dependency remains.
