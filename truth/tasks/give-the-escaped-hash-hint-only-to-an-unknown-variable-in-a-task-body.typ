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
  estimate: (src-files: 2),
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
  status: proposed(2026, 10, 19),
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

- 🟢 An unknown variable inside the `task.with(...)` header gets no escaped-hash hint.
- 🟢 An unknown name used as a status helper gets a hint that lists the configured statuses, as `mt flip --list` does.


== Changes

- `crates/mindtape-cli/src/util.rs`: give the hash hint only when the error's span falls in the body, and the status hint when it falls in the header.
- `crates/mindtape-cli/src/cmd/install/hint.rs`: add the status hint beside `escaped_hash_hint`.


== Rollback

- Revert both files.


== Out of scope

- A did-you-mean for misspelled field names, which is its own finding.


== Dogfood

Every agent that misspells a status in this repo's tasks gets the list of statuses instead of advice about `#`.
