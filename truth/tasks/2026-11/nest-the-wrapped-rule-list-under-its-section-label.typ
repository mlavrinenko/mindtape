#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Nest the wrapped rule list under its section label",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "omit-the-absent-line-when-nothing-is-absent.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the index's sections stay distinguishable at a glance],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 19),
)

== AS-IS

+ Reviewing the bare `mt explain` index while removing its `absent: none` line
+ 🔴 #raw("crates/mindtape-cli/src/cmd/explain/index.rs"): a wrapped rule list continues at the same two-space indent as the section label. A continuation line reads as a new section.

```
$ mt explain | sed -n '/^rules:/,/^derived/p'
rules:
  tasks.rules: tag-task-stateful-or-stateless, start-title-with-verb, keep-one-milestone-in-progress,
  keep-one-task-in-progress, cap-judgment-work-at-max-source-files, keep-work-inside-changes-plan,
  file-tweak-done, give-every-goal-verdict, file-tasks-by-month, sweeps-live-in-their-folder,
  link-finding-to-what-surfaced-it
```

The continuation `  keep-one-task-in-progress` sits in the same column as `  tasks.rules:`. The `templates:` line in the same block already continues at four spaces:

```
  templates: decision, dev, dev-stateful, docs, feat, feat-stateful, finding, fix, fix-stateful,
    sweep-dry, sweep-mutants, sweep-repeats, sweep-size, sweep-undertested, task, tweak (default: task)
```

== TO-BE

- 🟢 A wrapped rule list continues under its section label, indented past it.
- 🟢 The `rules:` block reads as five sections, not fifteen lines.

```text
$ mt explain | sed -n '/^rules:/,/^derived/p'
rules:
  tasks.rules: tag-task-stateful-or-stateless, start-title-with-verb,
    keep-one-milestone-in-progress, keep-one-task-in-progress,
    cap-judgment-work-at-max-source-files, keep-work-inside-changes-plan,
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/index.rs`: pass a four-space continuation to the `rules:` sections' `labelled` call, as `templates:` already does.

== Out of scope

- The `absent:` line's own wrapping. It is a top-level label, so its two-space continuation has no section to nest under.

== Dogfood

This repo's `mt explain` prints 11 rules under `tasks.rules`, and the wrapped tail stops reading as a section.
