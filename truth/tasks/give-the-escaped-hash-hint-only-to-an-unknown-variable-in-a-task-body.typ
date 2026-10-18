#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give the escaped-hash hint only to an unknown variable in a task body",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer who misspells a status is told the statuses, not sent to look for a stray hash],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 19),
)

== Problem

+ Misspelling a status helper in a task header and running `mt check`
+ 🔴 #raw("crates/mindtape-cli/src/util.rs"): `load_problems` attaches the escaped-hash hint to every first `unknown variable` error, so a misspelled helper in the `task.with(...)` header gets advice about `#` in the body

```
status: finished(2026, 10, 19)
error: tasks/set-up-ci.typ:5:11: unknown variable: finished
hint: a literal # in a task body opens a Typst expression: write \# or put the prose in a code block
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/util.rs").

== TO-BE

- 🟢 An unknown variable in the body keeps the escaped-hash hint, once per run.
- 🟢 An unknown variable inside the `status:` argument gets `hint: run mt flip --list to see the statuses`, worded as `mt flip` words its own refusal, and no escaped-hash hint.
- 🟢 Any other unknown variable in the `task.with(...)` header, such as a misspelled `ice(`, gets neither hint.


== Changes

- `crates/mindtape/src/model/task_spans.rs`: keep the byte ranges of the call and of each named argument's value, and say whether a line and column fall outside the call, in it, or in one argument's value.
- `crates/mindtape/src/loader/errors.rs`: record that region on `LoadError` when the file is evaluated.
- `crates/mindtape-cli/src/util.rs`: give the escaped-hash hint only outside the header, and the status hint only inside `status:`.
- `crates/mindtape-cli/src/cmd/install/hint.rs`: add the status hint beside `escaped_hash_hint`.


== Rollback

- Revert the four files.


== Out of scope

- A did-you-mean for misspelled field names, which is its own finding.


== Dogfood

Every agent that misspells a status in this repo's tasks gets the list of statuses instead of advice about `#`.
