#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the compiler error when just test fails to build",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[a red test that does not compile says why, so nobody reruns cargo by hand to read it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Writing a red unit test that calls a function not yet written
+ 🔴 #raw("Justfile"): the test recipe passes --cargo-quiet twice, which drops rustc's diagnostics too, so a build failure prints only the cargo command line and its exit code

```
$ just test -p mindtape a_text_slot_whose_example
error: command `cargo test --no-run --message-format json-render-diagnostics --package mindtape --workspace --quiet` exited with code 101
error: Recipe `test` failed on line 165 with exit code 101
$ cargo test --no-run -p mindtape --quiet
error[E0425]: cannot find function `escaped_examples` in this scope
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 `just test` on a crate that does not compile prints rustc's errors, and a clean build still prints no `Compiling` lines.


== Changes

- `Justfile`: keep `--cargo-quiet` once, or pass the build through a filter that keeps the diagnostics, whichever nextest allows.


== Rollback

- Revert the `test` recipe line.


== Out of scope

- The run banner and summary line nextest prints on a pass. `just check` already hides them.


== Dogfood

Every red test in this repo that fails to compile shows its error under `just test`.
