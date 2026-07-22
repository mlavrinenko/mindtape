#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "repair damaged tests in mindtape",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  tags: ("tests",),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that damaged these files
  ],
  status: done(
    2026,
    7,
    21,
  )[three files split and unminified; full sweep of five touched sources: 303 mutants, 288 caught, 15 unviable, 0 missed],
)

== Summary

Three `mindtape` test files were mutilated to buy space under the 500-line
linecop ceiling instead of being split: one was minified behind
`#[rustfmt::skip]`, one had its explanatory comments deleted, one had a test
replaced. Undo the damage and split the files properly.

== Scope

- `crates/mindtape/src/validate_tests.rs` (499/500) — `uuid_v7_rejects_v4`,
  `kind_cases` and `numeric_cases` are minified onto single 200-column lines
  with one-space indent and `#[rustfmt::skip]` to defeat the formatter. Each is
  a god-test (4 and 6 scenarios, `let mut t` shadowed repeatedly, no failure
  messages). Unminify, drop every `#[rustfmt::skip]`, split into ~10 named
  tests, and move the field-validation half into a `validate_tests_fields.rs`
  sibling.
- `crates/mindtape/src/loader_tests.rs` (496/500) — six explanatory comments
  were gutted or deleted outright to make room for two `first_task_marker`
  tests, including the block explaining why a bogus package version keeps the
  test hermetic. Restore them from `main` and split the file.
- `crates/mindtape/src/config_tests.rs` (500/500, zero headroom) — split.
  `embedded_defaults_matches_compiled_floor` promises a match but asserts
  `!is_empty() && contains("roots") && contains("proposed")`; either assert the
  match or rename it.
- `crates/mindtape/src/date_pin_tests.rs` —
  `pinned_epoch_days_is_none_before_init_runs` was deleted and replaced by
  `init_pinned_today_caches_env_value`, which sets the process-global `OnceLock`
  in-binary — exactly what the deleted test's comment warned against. The
  self-respawn shape is acceptable; the redundancy with
  `crates/mindtape-cli/tests/date_pin_e2e.rs` is not. Keep one.
- `crates/mindtape/src/date.rs` `today_epoch_days_brackets_the_system_clock`
  reads the real clock twice and races UTC midnight. Pin it against a single
  captured `SystemTime` instead.

== Verification

`just check` (linecop included), then `cargo mutants -f <file>` without
`--iterate` on each touched source file to confirm no pin was lost in the
rewrite.
