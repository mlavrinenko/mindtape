#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count a tests_-prefixed file as a test in the size sweep",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "size"),
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
      sound-codebase: helps[the size sweep scores a test module as a test, as crap and coverage already do],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Moving library areas into folders, where test modules of a mod.rs are named \`tests\_\*.rs\`
+ 🔴 #raw("contrib/ask-refactoring/size.sh"): The size arm's \`\$is\_test\` regex misses \`tests\_\*.rs\`, so a themed test module of a \`mod.rs\` is scored as production code.

`contrib/ask-refactoring/size.sh:62`: `and ($path | test("_tests\\.rs$|(^|/)tests\\.rs$|(^|/)tests/"))) as $is_test` — no `tests_` branch, while `CRAP_EXCLUDES` gained `**/tests_*.rs` in the area moves.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring/size.sh").

== TO-BE

- 🟢 `just ask-refactoring` scores `config/tests_id.rs` with the test confidence and ease, like `config/tests.rs`.

== Changes

- The size-arm fixture check (`.just/scripts/size-fixture-check.sh`): red first, a `tests_*.rs` path read as a test.
- `contrib/ask-refactoring/size.sh`: add `(^|/)tests_[^/]*\\.rs$` to `$is_test`.

== Rollback

- Revert the commit.

== Out of scope

- The other arms: `coverage.sh` already matches `tests_`.

== Dogfood

This repo's next sweep stops proposing a production split for a test module.
