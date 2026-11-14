#import "@local/mindtape:0.3.0": *

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
  )[three files split and de-minified. Full sweep of five touched sources: 303 mutants, 288 caught, 15 `unviable`, 0 missed],
)

== Summary

The burn-down mutilated three `mindtape` test files to buy space under the
500-line linecop ceiling instead of splitting them. One hid behind minified code
under `#[rustfmt::skip]`, one lost its explanatory comments, and one had a test
replaced. Undo the damage and split the files properly.

== Scope

- `crates/mindtape/src/validate_tests.rs` (499/500): `uuid_v7_rejects_v4`,
  `kind_cases` and `numeric_cases` sit on single 200-column lines with one-space
  indent and `#[rustfmt::skip]` to defeat the formatter. Each is
  a god-test (4 and 6 scenarios, `let mut t` shadowed repeatedly, no failure
  messages). De-minify them, drop every `#[rustfmt::skip]`, split into ~10 named
  tests, and move the field-validation half into a `validate_tests_fields.rs`
  sibling.
- `crates/mindtape/src/loader_tests.rs` (496/500): the burn-down gutted or
  deleted six explanatory comments outright to make room for two `first_task_marker`
  tests. One deleted block explained why a bogus package version keeps the test
  hermetic. Restore them from `main` and split the file.
- `crates/mindtape/src/config_tests.rs` (500/500, zero headroom): split.
  `embedded_defaults_matches_compiled_floor` promises a match but asserts
  `!is_empty() && contains("roots") && contains("proposed")`. Either assert the
  match or rename it.
- `crates/mindtape/src/date_pin_tests.rs`:
  the burn-down deleted `pinned_epoch_days_is_none_before_init_runs` and replaced it with
  `init_pinned_today_caches_env_value`, which sets the process-global `OnceLock`
  in-binary, exactly what the deleted test's comment warned against. The
  self-respawn shape is acceptable. The redundancy with
  `crates/mindtape-cli/tests/date_pin_e2e.rs` is not. Keep one.
- `crates/mindtape/src/date.rs` `today_epoch_days_brackets_the_system_clock`
  reads the real clock twice and races UTC midnight. Pin it against a single
  captured `SystemTime` instead.

== Verification

`just check` (linecop included), then `cargo mutants -f <file>` without
`--iterate` on each touched source file to confirm the rewrite kept every pin.
