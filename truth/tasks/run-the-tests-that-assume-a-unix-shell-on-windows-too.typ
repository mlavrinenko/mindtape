#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run the tests that assume a Unix shell on Windows too",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 5),
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
      sound-codebase: helps[the Windows job tests the hooks, the editor and the completion hint instead of the test's own shell],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 27),
)

== Problem

+ Run the Windows CI job (run 40576881551).
+ 🔴 Eight tests fail on their own assumptions, not on `mt`. The flip hook tests chain with `;` and read `$VAR`, but `cmd /C` chains with `&&` and reads `%VAR%`. The editor test runs `bash`, which on Windows can be WSL's launcher. The ENOTDIR test expects an error that Windows reports as not found. The completion hint test spells the path it expects with `/`.


== Environment

- `fix-windows-ci` at `bb1c59d5`, on the `windows-latest` GitHub runner.


== TO-BE

- 🟢 Each of these tests passes on Linux and Windows, or says why it is Unix only.


== Changes

- `crates/mindtape-cli/src/cmd/flip/hooks_tests.rs`, `crates/mindtape-cli/tests/it/flip_hooks_e2e.rs`: chain with `&&`, and spell the env var commands per platform.
- `crates/mindtape-cli/src/cmd/add/interactive_tests_editor.rs`: a `sh` editor on Unix and a `cmd` one on Windows.
- `crates/mindtape-cli/src/cmd/at_tests.rs`: the ENOTDIR test is Unix only.
- `crates/mindtape-cli/src/cmd/install/tests.rs`: build the expected path with `join`.


== Out of scope

- The shipped presets whose gates are `sh` only. That needs a decision on what a shipped gate runs on Windows.


== Dogfood

Not applicable. These are tests.

