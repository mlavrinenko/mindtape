#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the trailing period from an OS error reason",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 7),
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
      sound-codebase: helps[the not-found tests pass on Windows, where the OS words the reason differently],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 27),
)

== Problem

+ On Windows, run `mt import none.json`.
+ 🔴 It prints `error: can't read: 'none.json': the system cannot find the file specified.`, with a period mid-sentence, because Windows ends its reasons with one.
+ Six tests expect the Linux wording, `no such file or directory`, so they fail on Windows (CI run 40576881551).


== Environment

- `fix-windows-ci` at `bb1c59d5`, on the `windows-latest` GitHub runner.


== TO-BE

- 🟢 The reason drops a trailing period, so it reads the same as on Linux.
- 🟢 The tests expect each platform's own wording.


== Changes

- `crates/mindtape-cli/src/diag/io.rs`: the red test, a reason with a trailing period, then drop it. Add `NOT_FOUND` and `PATH_NOT_FOUND`, each platform's wording, for the tests.
- `crates/mindtape-cli/src/diag/tests.rs`, `crates/mindtape-cli/src/cmd/rm/gitless_retire_tests.rs`, `crates/mindtape-cli/src/cmd/add/tokens_tests.rs`, `crates/mindtape-cli/tests/it/import_error_format_e2e.rs`, `crates/mindtape-cli/tests/it/error_format_e2e.rs`: expect the platform's wording.


== Out of scope

- Rewording the OS reasons themselves. They stay the OS's words.


== Dogfood

Not applicable. This repo is developed on Linux.

