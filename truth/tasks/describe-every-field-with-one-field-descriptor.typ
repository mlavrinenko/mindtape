#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Describe every field with one field descriptor",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 3,
  ),
  difficulty: 6,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project retires a field value as it retires a tag, by editing one list],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[allowed values, closed-task exemption, slot choices and membership live in one place instead of three],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing the x- field commits f12ca44d through 922148f6
+ 🔴 #raw("crates/mindtape/src/validate.rs"): \`\[tasks.fields\] values\`, \`\[tasks.assignee\] allowed\` and \`\[tasks.tags\] allowed\` each have their own validator, slot generator and closed-task rule, and they already disagree

```
# done task with tags: ("retired-tag",) and x-machine: "delta", both outside the allow-lists
$ mt check
error: tasks/setup-ci.typ:5: x-machine: `delta` is not an allowed value

# template: assignee: slot("assignee"), x-machine: slot("machine")
$ mt add --describe
assignee  string  required
machine  choice  required  one of: alpha, beta

$ mt ls 'has(tags, "frontend")'
Error: unknown field in filter: tags
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/fields.rs").

== TO-BE

- 🟢 One descriptor per field (`kind`, `assignee`, `tags`, each `x-` field) carries its accessor, allowed values, required, default and closed-task exemption.
- 🟢 The closed-task exemption is a config `when` filter, and tags default to `!terminal` as they skip closed tasks today:

  ```toml
  [tasks.fields.x-machine]
  values = ["alpha", "beta"]
  values-when = "!terminal"
  ```
- 🟢 Validation, the `mt add` splice and choice inheritance read the descriptor, so a template's `assignee` slot offers the allowed names.
- 🟢 `has(field, value)` reads any list or dict field: `has(tags, "frontend")`, `has(estimate, "days")`.


== Changes

- `crates/mindtape/src/validate_tests_fields.rs`, `crates/mindtape/src/filter_extras_tests.rs`: red tests first: the three outputs above.
- `crates/mindtape/src/fields.rs`: the descriptor, and list values for tags and estimate keys.
- `crates/mindtape/src/validate.rs`, `crates/mindtape/src/add_slots.rs`, `crates/mindtape/src/config_tasks.rs`: read it, dropping the `kind` and `x-` name cases.
- `crates/mindtape/src/filter_predicates.rs`: `has()` reads the descriptor.


== Rollback

- Revert the commit; each field keeps its own checks.


== Out of scope

- Numeric scales and priority frameworks.
- List cardinality, filed separately.


== Dogfood

This repo's `[tasks.tags]` and `[tasks.fields.kind]` keep their behaviour through the descriptor, and `just check` runs both.
