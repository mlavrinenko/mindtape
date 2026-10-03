#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let the default task template file a dev task",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a rule the default template cannot satisfy stops pushing filers past `mt add`],
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
  status: done(2026, 10, 3),
)

== Problem

+ Filing a \`dev\` task with plain \`mt add\`.
+ 🔴 #raw(".mindtape/templates/task.typ"): The default template offers \`kind=dev\` but has no Rollback slot, which \`rollback-for-dev\` requires.

```
$ mt add -f - <<<'{title: Bump linecop, kind: dev, ...}'
error: ...: body.headers.rules.rollback-for-dev: a dev chore says how to undo it, in one Rollback section
```
Only `mt add --list` reveals the `dev` template.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".mindtape/templates/task.typ").

== TO-BE

- 🟢 When `mt add` refuses a task, and the chosen template left a field to a slot that the caller filled, every other template that writes that field as the same literal gets a `hint:` line under the error, as #raw("hint: template `dev` files kind=dev: mt add -t dev ...").
- 🟢 Several matching templates are each named. No match, no hint.
- 🟢 Nothing that is accepted today is refused. Only the failure text changes.


== Changes

- `crates/mindtape-cli/src/add_hint.rs`: reads each template's `task.with(…)` arguments and compares their literals with the caller's slot-filled values. It names no kind, template, field value or section.
- `crates/mindtape-cli/src/add.rs`: appends the hint to the error of a refused preview or write.
- `crates/mindtape-cli/tests/add_template_hint_e2e.rs`: a refusal names the pinning template, and a value no template pins gets no hint.
- `www/content/templates.typ`: the `mt add` page describes the hint.
- `mt import` takes full records and no template, so it needs no hint.


== Rollback

Revert the commit; a refused `mt add` goes back to the bare validation error.


== Out of scope

- The default template's slots, the `rollback-for-dev` rule and the config: the rule still holds, and the hint points at `-t dev`.
- Filing the task under another template automatically.


== Dogfood

The next `dev` task here is filed with plain `mt add`, or refused with a pointer to `-t dev`.
