#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the member project in a hint's mt command inside a set",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a hint inside a set can be copied and run as printed],
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

+ Running `mt @work check` over a set whose member has a misspelled status helper
+ 🔴 #raw("crates/mindtape-cli/src/diag/hint.rs"): Inside `mt @set check`, a hint that names an `mt` command prints it without the member's `@name`, so running it from the set's directory doesn't reach the project the error came from

```
$ mt @work check
error: tasks/draft-the-release-notes.typ:4:11: unknown variable: finishd
hint: run mt flip --list to see the statuses
note: project: blog
error: 1 of 2 projects failed
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/diag/hint.rs").

== TO-BE

- 🟢 Inside `mt @work check`, the hint for the blog member reads `hint: run mt @blog flip --list to see the statuses`.
- 🟢 Every hint that names a project-scoped `mt` command gets the member's `@name` the same way, from one place, not hint by hint.
- 🟢 Outside a set, hints are unchanged.


== Changes

- `crates/mindtape-cli/src/diag/hint.rs`: let a hint's command take the member's `@name`.
- The set runner that adds `note: project: …`: hand the member's name to the hints it prints.


== Rollback

- Revert both files.


== Out of scope

- Hints that name a global command such as `mt install`, which take no `@name`.


== Dogfood

Not applicable, this repo is one project, not a set.
