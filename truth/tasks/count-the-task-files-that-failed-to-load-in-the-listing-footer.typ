#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count the task files that failed to load in the listing footer",
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
      readable-tasks: helps[the footer accounts for every task file under the roots],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer whose task file broke sees it in the count, not only in a warning that scrolled away],
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

+ Listing a project in which one task file has a misspelled field
+ 🔴 #raw("crates/mindtape-cli/src/cmd/ls/mod.rs"): `mt ls` prints a broken file's error as a warning above the table, then leaves the file out of the footer's count, so the footer reads as if every task file loaded

```
$ ls tasks | wc -l
3
$ mt ls
warning: tasks/set-up-ci.typ:4:3: unexpected argument: titel
SLUG#  STATUS    TITLE                       FILE
gbz    proposed  Write the onboarding guide  tasks/write-the-onboarding-guide.typ

default view · 1 of 2 tasks
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/ls/mod.rs").

== TO-BE

- 🟢 When some task files fail to load, the footer ends with their count: `default view · 1 of 2 tasks · 1 broken`.
- 🟢 When every file loads, the footer is unchanged.
- 🟢 `mt ls` still exits 0 and still prints each load error as a warning; `mt check` stays the gate that fails on them.


== Changes

- `crates/mindtape-cli/src/cmd/ls/mod.rs`: pass the load-error count to the footer and append `· N broken` when it is above zero.
- `crates/mindtape-cli/tests/it/`: an end-to-end test listing a project with one broken file.


== Rollback

- Revert both files.


== Out of scope

- The JSON envelope, unless it already carries load problems, which stays as it is.
- `mt ls`'s exit code, which stays 0, as read commands tolerate a broken file.


== Dogfood

When an agent breaks a task file in this repo, the next `mt ls` footer says so.
