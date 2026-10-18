#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Warn about the task files that fail to load in mt find",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: helps[mt find names a task file it could not read, as mt ls does],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer whose search skips a broken task file learns which file broke],
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

+ Counting the task files that failed to load in the `mt ls` footer
+ 🔴 #raw("crates/mindtape-cli/src/cmd/find.rs"): `mt find` opens the project with `util::open()`, not `util::open_warned()`, so it skips a task file that fails to load without a warning, while `mt ls` and `mt @set find` print one

```
$ mt ls >/dev/null
warning: tasks/setup-ci.typ:3:3: unexpected argument: titel
$ mt find a >/dev/null
$ echo $?
0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/find.rs").

== TO-BE

- 🟢 `mt find` prints a warning for each task file that fails to load, the same lines `mt ls` prints.
- 🟢 `mt find` still lists the rest and exits 0.


== Changes

- `crates/mindtape-cli/src/cmd/find.rs`: open the project with `util::open_warned()`.
- `crates/mindtape-cli/tests/it/ls_broken_e2e.rs`: assert the warning in `find_counts_the_broken_file_too`.


== Rollback

- Revert both files.


== Out of scope

- `mt @set find`, which already warns through `util::open_set_warned`.
- The footer, which already counts the broken files.


== Dogfood

When an agent breaks a task file in this repo, `mt find` names it.
