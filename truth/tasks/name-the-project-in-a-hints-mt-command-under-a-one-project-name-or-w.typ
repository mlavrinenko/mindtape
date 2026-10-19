#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the project in a hint's mt command under a one-project @name or -w",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a hint under mt @blog can be copied and run as printed],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 19),
)

== Problem

+ Running `mt @blog check` from the directory that holds the set
+ 🔴 #raw("crates/mindtape-cli/src/cmd/at.rs"): under `mt @blog` or `-w`, a hint names a bare `mt` command, which fails run as printed from the same directory

```
$ mt @blog check
hint: run mt flip --list to see the statuses
$ mt flip --list
error: no .mindtape/ directory here or above
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/diag/hint.rs").

== TO-BE

- 🟢 Under `mt @blog check`, the hint reads `hint: run mt @blog flip --list to see the statuses`.
- 🟢 Under `mt -w projects/blog check`, the hint carries `-w projects/blog`, so it runs from where the user stood.
- 🟢 A run inside the project, or one that names it through `$MINDTAPE_WORKTREE`, keeps its hints bare.


== Changes

- `crates/mindtape-cli/src/diag/hint.rs`: build a project `Command` under the `@name` or `-w <dir>` that `crate::selector` already records for `mt ls`'s next-page line, and let `Diag::in_member` put a set member's `@name` in its place.


== Rollback

- Revert the file.


== Out of scope

- A set member's hints, which `Diag::in_member` already scopes.
- Hints that name a global command such as `mt install`.


== Dogfood

Not applicable, this repo runs `mt` from inside its own tree, where hints stay bare.
