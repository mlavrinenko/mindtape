#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give a tasks.fields entry a cardinality so mt add files the lists validate accepts",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  estimate: (src-files: 4),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[mt add and mt check accept the same values for a declared field],
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
  links: related(
    "describe-every-field-with-one-field-descriptor.typ",
  )[the descriptor absorbs cardinality; either can land first],
)

== Problem

+ Reviewing the list-valued extension field check in c1882d4d
+ 🔴 #raw("crates/mindtape/src/add_slots.rs"): \`validate\` accepts a list in a declared \`x-\` field, but every slot \`mt add\` makes for it is a single-value \`choice\`, so a list cannot be filed

```
# [tasks.fields.x-machine] values = ["alpha", "beta"], required = "!terminal"
$ mt add "Two" -- 'x-machine=("alpha","beta")'
Error: slot "x-machine" value "(\"alpha\",\"beta\")" is not one of: alpha, beta
$ mt add "Three" -f - <<<'{x-machine: [alpha, beta]}'
Error: slot "x-machine" expects choice, got list
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config_tasks.rs").

== TO-BE

- 🟢 `[tasks.fields.<name>] multiple = true` declares a list-valued field.
- 🟢 `mt add` offers such a field as a multi-choice slot, spliced or inherited, and files a list.
- 🟢 `mt check` refuses a scalar in a `multiple` field and a list in any other.


== Changes

- `crates/mindtape-cli/tests/add_inherited_choices_e2e.rs`: red tests first — a list filed through a spliced and an inherited slot.
- `crates/mindtape/src/config_tasks.rs`: `FieldConfig.multiple`.
- `crates/mindtape/src/template_fill.rs`, `crates/mindtape/src/template.rs`: a multi-choice slot kind.
- `crates/mindtape/src/add_slots.rs`, `crates/mindtape/src/validate.rs`: read `multiple`.


== Rollback

- Revert the commit; a list field is filed by hand again.


== Out of scope

- `mt set` writing an extension field.
- Dict-valued fields.


== Dogfood

Not applicable: this repo declares no list-valued field; a dotfiles repo files `x-machine` lists with `mt add`.
