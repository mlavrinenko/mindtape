#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fix the dir refusal hint",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  status: done(2026, 11, 12),
  assignee: "agent",
)

== AS-IS

+ Reviewing the on-add branch after `just check`.
+ 🔴 #raw("crates/mindtape/src/model/writable.rs"): The `dir` hint says `mt mv` is the filed-task answer, but `mt mv` only renames and keeps the folder.

The hint read `use mt mv <task> <new-stem> for a filed task`, while `mt mv --help` says it keeps the task's folder.


== TO-BE

- 🟢 The hint says `--dir` chooses the folder when adding and `mt mv` only renames a filed task.
- 🟢 The `mt set` page and skill reference repeat the same fact.

```
hint: a directory is not a field; --dir chooses it when adding, and mt mv only renames a filed task
```


== Changes

- `crates/mindtape/src/model/writable.rs`: correct the `dir` hint.
- `crates/mindtape/src/model/writable_tests.rs`, `crates/mindtape-cli/tests/it/set_e2e.rs`, `add_fields_e2e.rs`: pin it.
- `crates/mindtape-cli/src/cmd/set.rs`, `www/content/cli/set.typ`, `www/utils/cli-notes.typ`: match the wording.


== Out of scope

- Adding a command that moves a filed task between folders.


== Dogfood

A user is not sent to a command that cannot do what the hint promises.

