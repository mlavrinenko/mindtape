#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check each template alone, without the backlog",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 4,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a backlog that satisfies its rules passes mt check, and a template breach is one the template can fix],
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
  status: proposed(2026, 10, 4),
  links: related(
    "locate-a-refused-writes-breach-on-the-file-on-disk-through-one-task-call-span-service.typ",
  )[template\_check::relocate moves into the span service],
)

== Problem

+ Reviewing the template check that cf85120b added to \`mt check\`
+ 🔴 #raw("crates/mindtape-cli/src/check_templates.rs"): \`mt check\` validates every template in one \`check\_candidates\` batch with the live backlog, at \`\<dest>/\<template-name>.typ\`, so backlog state and stem collisions fail the template pass

```
# 4 open tasks, rule assert = 'count(!terminal) <= 4'; the backlog holds it
$ mt check
error: .mindtape/templates/task.typ:3: rules.open-cap: must satisfy `count(!terminal) <= 4`
error: .mindtape/templates/task.typ:3: rules.open-cap: must satisfy `count(!terminal) <= 4` (reported on add-dark-mode)
Error: 5 violation(s), 0 load error(s)

# tasks/archive/fix.typ exists beside template fix.typ
$ mt check
error: .mindtape/templates/fix.typ:2: id: duplicate id `fix`; first seen in `tasks/archive/fix.typ`
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/check_templates.rs").

== TO-BE

- 🟢 Each template is checked alone against an empty backlog, so a count cap the backlog holds never fails the template pass.
- 🟢 Set-reading breaches (`count()` rules, acyclic link kinds) are dropped from a template's report.
- 🟢 The candidate is named by `output_stem` from the example title, where `mt add` would write it, so a task named like a template collides with nothing.


== Changes

- `crates/mindtape-cli/tests/check_templates_e2e.rs`: red tests first: a backlog at its `count()` cap, two templates that jointly pass a cap, a task stem equal to a template name.
- `crates/mindtape-cli/src/check_templates.rs`: one candidate per template against no existing tasks, named by `output_stem`.
- `crates/mindtape/src/candidate.rs`: expose the set-reading classification `elsewhere_violations` already uses, so the template pass drops those breaches.


== Rollback

- Revert the commit; templates are checked as one batch beside the backlog again.


== Out of scope

- Which templates are listed, and the fallback severity.
- The candidate check `mt add` runs, which must keep reading the backlog.


== Dogfood

This repo's `mt check` stops running the whole-project `validate` a second time for its eleven templates.
