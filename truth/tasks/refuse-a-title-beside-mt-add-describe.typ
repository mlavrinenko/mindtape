#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a title beside mt add --describe",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent that types `mt add --describe finding` learns the right slots instead of filling the wrong template],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Filing review findings with the \`finding\` template
+ 🔴 #raw("crates/mindtape-cli/src/add.rs"): \`mt add --describe finding\` takes \`finding\` as a title and describes the default template, with no warning

```
$ mt add --describe finding | head -2
title  string  required  # Task title, imperative verb first
    e.g. Test that a task without a due date sorts last
```
The `finding` template's title prompt is `what should change, imperative verb first`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/add.rs").

== TO-BE

- 🟢 `mt add --describe finding` fails with `--describe takes no title; did you mean -t finding?` when `finding` names a template, and with `--describe takes no title` otherwise.


== Changes

- `crates/mindtape-cli/tests/`: the red e2e test first.
- `crates/mindtape-cli/src/add.rs`: refuse title words beside `--describe`, naming the template when one matches.


== Rollback

- Revert the `add.rs` check.


== Out of scope

- `--list` and `--print`. `--print` takes a title on purpose.


== Dogfood

An agent here that misreads the skill's `mt add --describe` line gets corrected on the first try.
