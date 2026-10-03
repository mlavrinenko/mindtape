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
  status: proposed(2026, 10, 3),
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

- 🟢 `mt add` with the default template either files a valid `dev` task or refuses `kind=dev` up front, naming `-t dev`.


== Changes

- `.mindtape/templates/task.typ`: drop `dev` from the kinds it offers, or add the Rollback slot when the kind is `dev`.
- `.mindtape/config.toml`: let the `rollback-for-dev` message name `mt add -t dev`.


== Rollback

Revert the commit; the template and config return to offering `dev` without a Rollback slot.


== Out of scope

- The `rollback-for-dev` rule itself: a dev chore should still say how to undo it.


== Dogfood

The next `dev` task here is filed with plain `mt add`, or refused with a pointer to `-t dev`.

