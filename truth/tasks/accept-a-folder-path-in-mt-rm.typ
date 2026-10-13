#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept a folder path in mt rm",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: helps[a finished batch of tasks goes with one command naming its folder],
      implementable-tasks: neutral,
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
  status: proposed(2026, 10, 13),
)

== Problem

+ Planning to file each `just ask-refactoring` batch into a folder and remove it once done
+ 🔴 #raw("crates/mindtape-cli/src/cmd/rm/mod.rs"): `mt rm` reads a folder path as a filter and refuses it, so removing a folder of tasks takes its derived tag instead

```
$ mt rm tasks/sw1 --dry-run
error: unsupported in filter: 'tasks/sw1': '/' isn't an operator the filter language has
$ mt rm 'has_tag("sw1")' --dry-run
would delete 2 task(s):
  - tasks/sw1/fix-one.typ
  - tasks/sw1/fix-two.typ
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/rm/mod.rs").

== TO-BE

- 🟢 `mt rm tasks/sw1` deletes every task under that folder, as `mt rm tasks/sw1/fix-one.typ` deletes one, and `--dry-run` lists them.

== Changes

- `crates/mindtape-cli/src/cmd/rm/mod.rs`: resolve a target that names a directory under a root to the tasks inside it.
- `crates/mindtape-cli/tests/it/`: the red e2e test first, a folder of two tasks removed by its path.
- `truth/tasks/swept/`: removed whole by the new command, as Dogfood says.

== Rollback

- Revert the commit. No file format changes.

== Out of scope

- Other commands that take task targets. Each follows once someone asks for it.

== Dogfood

The closing commit runs `mt rm truth/tasks/swept --dry-run`, checks it lists only
the closed sweep tasks (250 when filed), then runs it for real and commits the
removal with this task. From then on a finished sweep batch goes the same way.
