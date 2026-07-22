#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "repair damaged tests in mindtape-cli",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  tags: ("cli", "tests"),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that damaged these files
  ],
  status: done(
    2026,
    7,
    21,
  )[comments restored, three god-tests fixed, three splits made honest; 112 CLI mutants 104 caught 0 missed, add.rs 37/37 at -j1],
)

== Summary

Same repair as the `mindtape` half, on the CLI crate: comments stripped from
pre-existing tests to buy line budget, one aggregate god-test, and a file split
whose stated rule the file itself violates.

== Scope

- `crates/mindtape-cli/src/flip_tests.rs` (474/500) — eight explanatory comments
  were stripped from pre-existing, passing transition tests (`// blocked → done
  is not an edge…`, `// wip → wip, even though wip is not in its own to list.`),
  blank lines removed inside bodies, and `→` rewritten to `->` in doc comments.
  None of it was required by any mutant. Restore from `main`.
- `flip_tests.rs` `flip_output_behavior` — one body, five tempdirs, single-char
  names, six scenarios, asserting on the *aggregate* stdout, so it can pass for
  the wrong reason. Its four assertions are each re-made in isolation by
  `restamp_prints_re_stamped`, `flip_prints_flipped_not_re_stamped`,
  `terminal_flip_prints_recur_notice` and `list_prints_verbs` directly below.
  Move its unique `err.contains("shadows")` check into its own test, then delete
  it (saves ~40 lines and a subprocess spawn).
- `flip_tests.rs` `run_delegates_to_run_with` — proves nothing about delegation
  (any `Err` from an unknown status passes), calls process-global
  `set_current_dir`, and binds `let ok = run(...).is_err()`. Rewrite or drop.
- `flip_tests.rs` `revalidate_catches_unreachable_status` — asserts only
  `is_err()`; cannot tell revalidation from any other failure. Assert the
  message.
- `crates/mindtape-cli/src/import_tests.rs` (495/500) — split.
- `add_tests.rs` / `add_tests_stdin.rs` — the split doc claims the criterion is
  "subprocess tests", but `add_tests.rs` kept two of its own
  (`warn_if_outside_roots_*`). Either move them and rename the sibling
  `add_tests_subprocess.rs`, or re-split by feature and fix the doc.

Splits that are already coherent and need no work: `render/table_tests*.rs`,
`ls_interactive/{mod,tui,tests,tui_tests}.rs`, `rm_tests.rs` +
`rm_cli_tests.rs`.

== Verification

`just check`, then `cargo mutants -f <file>` without `--iterate` on each touched
source file.
