#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read the misspelled-field span through typst-harvest, not a full compile",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tests",),
  estimate: (src-files: 2),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        sound-codebase: helps[one test stops compiling a task file through `typst` to read a span the harvest already reports],
      ),
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 28),
)

== AS-IS

+ Read `crates/mindtape/src/loader/tests_prelude.rs`, test `real_prelude_places_the_unknown_slot_error_on_the_name_in_the_task_file`.
+ 🔴 It runs `typst::compile::<PagedDocument>` to read the error's span, and `crates/mindtape/Cargo.toml` says why: "The harvest flattens an eval error to its message". That stopped being true: typst-harvest 0.6.0 returns `HarvestError::Eval` with each diagnostic's `Location`, whose `path` is root-relative for a project file and `{pkg-spec}/...` for a package file.


== TO-BE

- 🟢 The test reads the error's own location from `typst_harvest::harvest` and asserts it lies in `tasks/t.typ`, on `titel`.
- 🟢 The dev-dependency comment on `typst` and `typst-layout` names the one test that still needs them: the count estimate's card must lay out, and the harvest never lays out.


== Changes

- `crates/mindtape/src/loader/tests_prelude.rs`: assert the span through the harvest's `Diagnostic::location`.
- `crates/mindtape/Cargo.toml`: reword the dev-dependency comment.


== Rollback

- Revert the commit. Test code and a comment only.


== Out of scope

- Dropping `typst` and `typst-layout`: the count-estimate test lays the card out, which the eval-only harvest never does.


== Dogfood

Not applicable: test code only.

