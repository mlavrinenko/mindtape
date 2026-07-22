#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "drop redundant and vacuous tests",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  tags: ("tests",),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that added these tests
  ],
  status: done(
    2026,
    7,
    21,
  )[duplicates dropped, vacuous tests made deterministic, 92 coordinate comments rewritten; 328 mutants across 12 pruned sources, 302 caught, 0 missed],
)

== Summary

The burn-down optimised for "a mutant dies", not "a reader learns something", so
it left duplicates, tests that cannot fail, names that describe the mutation
operator instead of the behaviour, and comments that hard-code source line
numbers. Prune.

== Scope

Delete or merge:

- `crates/mindtape/src/naming_tests.rs` —
  `disambiguate_line_155_not_deletion_regression` is byte-equivalent to
  `disambiguate_appends_dash_two_on_first_collision`, and the latter was
  *rewritten* into that duplicate, so single-collision → `foo-2` is now
  uncovered and the name is a lie. Restore the original body, drop the
  duplicate.
- `crates/mindtape-cli/src/rm_cli_tests.rs` — `confirm_rejects_n` and
  `run_aborts_on_declined_confirm` are the same test with different comments,
  each spawning `mt`. Keep the latter.
- `crates/mindtape-cli/src/cli.rs` — `builtin_names_not_empty` is subsumed by
  `builtin_names_contains_check` three lines above. `verbosity_3_is_trace` and
  `verbosity_high_is_trace` hit the same `_ =>` arm; fold all five
  `verbosity_*` tests into one table test.
- `crates/mindtape-cli/src/render/color.rs` — 12 one-line `parse_*` tests
  asserting only `is_some()`, which never pins the *mapping* (a mutant sending
  `"cyan"` to `Color::Red` survives all twelve). One table test asserting exact
  `Color` values replaces them; saves ~45 lines.
- `crates/mindtape-cli/tests/cli_dump_e2e.rs` — asserts exactly what
  `cli_dump.rs` `dump_json_is_valid` and `dump_json_contains_bin_name` already
  assert in-process. Keep only what needs the binary.
- `crates/mindtape-cli/src/graph_tests.rs` — `run_prints_graph_from_disk`
  subsumes `run_with_prints_graph`'s `"digraph mindtape"` assertion; two spawns
  where one would do.

Fix, do not delete:

- `crates/mindtape-cli/src/go.rs` `editor_command_rejects_blank_config` — the
  `Err` branch asserts nothing, so the "rejects" it names is never checked.
  `editor_command_falls_through_to_env` wraps its whole body in `if VISUAL or
  EDITOR is set`, so on a clean runner it asserts nothing.
- `crates/mindtape/src/framework_tests.rs`
  `validate_rejects_scalar_with_replace_minus_with_divide` — name encodes the
  mutation operator and says "rejects" over an `is_ok()` body. Rename to what it
  checks. (The rest of the file is good work.)
- `crates/mindtape/src/test_support.rs`
  `write_lib_package_writes_typst_toml_and_prelude` — byte-for-byte manifest
  assertion for a *test helper*, and its `prelude` fixture is a task source.
- `crates/mindtape-cli/src/handle.rs` `check_chars_second_letter_differs` —
  asserts a magic literal (`"op"`), not what the name says.
- `crates/mindtape-cli/src/render/table_tests_layout.rs`
  `glyph_mode_is_none_in_test_environment` asserts a property of the harness,
  not of the code.
- `crates/mindtape-cli/src/render/columns.rs` `due_yesterday_is_overdue` uses
  `"2000-01-01"`, not yesterday.
- Repo-wide: comments hard-coding source coordinates (`// Kills line 413:`,
  `// validate_scalar line 205:32:`, `// rm.rs:293:22`) — already stale in
  `ls_rules_tests.rs`. Rewrite each to name the predicate, not the coordinate.

== Verification

`just check`, then `cargo mutants -f <file>` without `--iterate` on each source
file whose tests were pruned — a delete is only safe if the file still sweeps
clean.
