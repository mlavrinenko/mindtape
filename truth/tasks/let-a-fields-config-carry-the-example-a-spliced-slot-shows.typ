#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

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
      implementable-tasks: helps[--describe shows a sample the project wrote, not one Rust made up],
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
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing the examples cf85120b gives slots that mt add splices
+ 🔴 #raw("crates/mindtape/src/add_slots.rs"): a spliced slot's example is invented in Rust (a scale midpoint, the first value, the field name without \`x-\`, \`someone\`), so a rule on the field's shape fails the template check with no config to fix it

```
# [tasks.fields.x-ticket] required = true; rule assert = 'matches(x-ticket, "^[A-Z]+-[0-9]+$")'
$ mt check
warning: <built-in>/task.typ:3: rules.ticket-shape: must satisfy `matches(x-ticket, "^[A-Z]+-[0-9]+$")`
$ mt add --describe
x-ticket  string  required  # x-ticket
    e.g. ticket
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/add_slots.rs").

== TO-BE

- 🟢 `[tasks.fields.<name>]`, `[tasks.assignee]`, `[tasks.estimate]` and the numeric scales take `example`, and a spliced slot shows it.
- 🟢 A spliced slot with no configured example is an unfillable slot naming its config path, as an authored slot is.


== Changes

- `crates/mindtape-cli/tests/check_templates_e2e.rs`: red tests first: a configured example fills, a missing one names the config path.
- `crates/mindtape/src/config_tasks.rs`: the `example` keys.
- `crates/mindtape/src/add_slots.rs`: read them; drop `midpoint`, `PLACEHOLDER_ASSIGNEE` and the name fallback.


== Rollback

- Revert the commit; Rust invents the examples again.


== Out of scope

- Examples of slots a template declares itself.


== Dogfood

This repo's `.mindtape/config.toml` gives each required field an example, and `just check` fills them.
