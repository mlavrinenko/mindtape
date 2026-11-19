#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Reopen the mt add buffer when a rule rejects the filled task",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  links: (
    (
      task: "keep-the-mt-add-buffer-when-the-fill-rejects-it.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a rejected rule keeps the answers and the task gets filed instead of abandoned],
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: helps[one editor session instead of restarting the form],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
)

== AS-IS

+ Filling `mt add --edit -t fix` in this repo, then saving a buffer whose `AS-IS` has no 🔴 step
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/mod.rs"): a rule that refuses the filled task parks the rendered task and drops the buffer, so the form cannot come back

```
$ mt add --edit -t fix
error: <new task from template 'task'>:33: body.content.rules.mark-defect-step-with-red: open the step that shows the defect with 🔴
note: section `AS-IS` must match `(?m)^\s*(\+|\d+\.)\s+🔴`
note: the filled task was parked at: '/tmp/mindtape-rejected-EDuSuO.typ'
```

`park()` writes `source`, the filled task, so the parked file is the rendered Typst rather than the YAML buffer. The answers are readable there, and `mt add --edit` again starts from the scaffold.

== TO-BE

- 🟢 A rule that refuses the filled task re-opens the buffer in the same editor, with the breach as a `#` comment above the values, until the filled task passes.
- 🟢 The parked file stays for a rejection the loop cannot fix, and the error still names it.
- 🟢 `[add] interactive = never` and a non-terminal run keep today's behaviour: one error, no editor.

```
$ mt add --edit -t fix
# error: body.content.rules.mark-defect-step-with-red: open the step that shows the defect with 🔴
title: "Sort tasks without a due date last"
as-is: |-
  + Run `mt ls --sort due:asc`.
  + The undated tasks print first.
```

== Changes

- `crates/mindtape-cli/src/cmd/add/pipeline.rs`: the loop renders the filled task, runs the candidate check, and stops on an unchanged buffer or the round bound.
- `crates/mindtape-cli/src/cmd/add/seed.rs`: the value seeding split out of `pipeline.rs`. The render loop then fits the file cap.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: the candidate check moves into the loop, so a rule breach re-opens instead of parking.
- `crates/mindtape-cli/src/cmd/add/interactive.rs`: the error comment carries every breach, one `#` per line.
- `crates/mindtape-cli/src/cmd/add/pipeline_tests.rs`: the red test first, a buffer whose `AS-IS` has no 🔴 coming back with the breach. Also the unchanged-buffer and round-bound stops.
- `crates/mindtape-cli/tests/it/add_edit_rule_e2e.rs`: the same loop end to end, under a pseudo-terminal.
- `crates/mindtape-cli/src/task_edit.rs`: `candidate_problems` returns every fatal breach without printing, for the loop.
- `www/content/templates.typ`, `CHANGELOG.md`: the loop covers a rule breach.

== Out of scope

- A `[[on.add]]` gate that refuses the draft. It runs inside `render`, after the loop, and its message is about a command rather than a value. It gets its own finding once someone hits it.
- A write failure after the loop. The file is on disk by then, and the error already says so.

== Dogfood

This repo's `mt add --edit -t fix` re-opens on a missing 🔴 step instead of parking the task.
