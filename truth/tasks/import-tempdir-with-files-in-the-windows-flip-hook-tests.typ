#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Import tempdir_with_files in the Windows flip hook tests",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 0),
  tags: ("found", "tests"),
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
      sound-codebase: helps[the Windows cross-check compiles again],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17)[just check-windows compiles],
)

== Problem

+ Running `just check-windows` on the at.toml registry branch
+ 🔴 #raw("crates/mindtape-cli/src/cmd/flip/hooks_cmd_tests.rs"): the `windows` test module calls `tempdir_with_files` but imports only `tempdir`, so `just check-windows` fails to compile

```
error[E0425]: cannot find function `tempdir_with_files` in this scope
   --> crates/mindtape-cli/src/cmd/flip/hooks_cmd_tests.rs:141:19
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/flip/hooks_cmd_tests.rs").

== TO-BE

- 🟢 `just check-windows` compiles every target with no error.


== Changes

- `crates/mindtape-cli/src/cmd/flip/hooks_cmd_tests.rs`: import `tempdir_with_files` in `mod windows`.


== Rollback

- Revert the import line.


== Out of scope

- The unused-import warnings the same run prints, which are their own noise.


== Dogfood

Every `just check-windows` in this repo compiles again.
