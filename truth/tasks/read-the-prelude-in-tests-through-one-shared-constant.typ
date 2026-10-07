#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read the prelude in tests through one shared constant",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  kind: "test",
  tags: ("tests",),
  estimate: (src-files: 0),
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
      sound-codebase: helps[a test file can move to another folder depth without its prelude path breaking],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7),
  links: related(
    "organize-each-crates-source-as-one-folder-per-area-with-a-mod-rs.typ",
  )[clears the way for its test-file moves],
)

== Problem

+ Run `rg -l 'include_str!\("\.\./\.\./\.\./lib/prelude.typ"\)' crates`.
+ 🔴 16 test files each spell the path to `lib/prelude.typ` relative to their own depth, so moving any of them into an area folder breaks its build.

== TO-BE

- 🟢 `mindtape::test_support::PRELUDE` holds the prelude, and every unit test in both crates reads it from there.
- 🟢 `rg 'lib/prelude.typ' crates/*/src` names `test_support.rs` and `crates/mindtape-cli/src/init.rs` only.

== Changes

- `crates/mindtape/src/test_support.rs`: `pub const PRELUDE`, and the `write_lib_package` doc names it.
- The 16 `*_tests*.rs` files: read `PRELUDE` from `test_support` (the CLI's through `mindtape::test_support`).

== Out of scope

- `crates/mindtape-cli/src/init.rs`: product code, which the `test-support` feature does not build.
- Integration tests under `crates/*/tests/`: they do not move.

== Dogfood

The area moves in `truth/tasks/organize-each-crates-source-as-one-folder-per-area-with-a-mod-rs.typ` move test files without touching a prelude path.
