#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Locate a refused write's breach on the file on disk through one task-call span service",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a refused write points the editor at the line that is on disk],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one parse of the task.with call replaces three lookups that can disagree],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
  links: related(
    "carry-a-severity-on-every-violation-and-render-every-breach-in-one-place.typ",
  )[both reshape how a breach is located and printed; the renderer prints what the span service locates],
)

== Problem

+ Reviewing commits 05c9c425 and 3c1dca31 in a scratch project
+ 🔴 #raw("crates/mindtape-cli/src/pre_write.rs"): A refused write reports lines of the unwritten candidate, and loader.rs, model.rs and template\_check.rs each look up the task.with call on their own.

On disk, line 7 of `tasks/b.typ` is `)` and its second `= Problem` is line 17:
```
$ mt set b priority=55
error: tasks/b.typ:7: priority: 55 is outside the priority scale 0..9
error: tasks/b.typ:18: body.headers.rules.one-problem: one Problem
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/loader.rs").

== TO-BE

- 🟢 A refused `mt set`, flip or edit prints each breach at its line in the file on disk.
- 🟢 A refused `mt add` says the breach is in the task as filled, with no path to a missing file.
- 🟢 One `TaskCallSpans`, parsed once at load, answers every field and call line, including a call through `helper.with(`.


== Changes

- `crates/mindtape/src/loader.rs`: build `TaskCallSpans` once per file; move the stray doc comment back onto `marker_call`.
- `crates/mindtape/src/model.rs`: `field_location` reads the spans.
- `crates/mindtape/src/template_check.rs`: `relocate` maps lines through the shared spans.
- `crates/mindtape-cli/src/pre_write.rs`: remap a refused candidate's lines onto the authored file.
- `crates/mindtape-cli/tests/set_e2e.rs`: a red test for a refused `mt set` that inserts a field.


== Rollback

- Revert the commit; refused writes print candidate lines again.


== Out of scope

- Severity rendering and config-level locations, each filed on its own.


== Dogfood

Agents here hit refused writes daily, and the reported line now opens on the right text.
