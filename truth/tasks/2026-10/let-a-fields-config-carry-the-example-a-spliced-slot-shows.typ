#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Let a field's config carry the example a spliced slot shows",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[`--describe` shows a sample the project wrote, not one Rust made up],
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
  status: done(2026, 10, 5),
)

== AS-IS

+ Reviewing the examples cf85120b gives slots that mt add splices
+ 🔴 #raw("crates/mindtape/src/add_slots.rs"): Rust invents a spliced slot's example (a scale midpoint, the first value, the field name without \`x-\`, \`someone\`). A rule on the field's shape then fails the template check, and no config can fix it

```
# [tasks.fields.x-ticket] required = true; rule assert = 'matches(x-ticket, "^[A-Z]+-[0-9]+$")'
$ mt check
warning: <built-in>/task.typ:3: rules.ticket-shape: must satisfy `matches(x-ticket, "^[A-Z]+-[0-9]+$")`
$ mt add --describe
x-ticket  string  required  # x-ticket
    e.g. ticket
```


This fell outside the task that surfaced it, so this task files it here rather
than leaving it in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/add_slots.rs").

== TO-BE

- 🟢 `[tasks.fields.<name>]`, `[tasks.assignee]`, `[tasks.estimate]` and the numeric scales take `example`, and a spliced slot shows it.
- 🟢 A spliced slot with no configured example fails as a slot that lacks a value, and gives its config path, as an authored slot does.


== Changes

- `crates/mindtape-cli/tests/check_templates_e2e.rs`: red tests first: a configured example fills, a missing one gives the config path.
- `crates/mindtape/src/config_tasks.rs`: the `example` keys.
- `crates/mindtape/src/add_slots.rs`: read them. Drop `midpoint`, `PLACEHOLDER_ASSIGNEE` and the name fallback.


== Rollback

- Revert the commit. Rust invents the examples again.


== Out of scope

- Examples of slots the template declares.


== Dogfood

This repo's `.mindtape/config.toml` gives each required field an example, and `just check` fills them.
