#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the fields a breach reads on the Violation",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a refused mt add points at the template that fixes it, body rules included],
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
  status: done(2026, 10, 4),
  links: related(
    "carry-a-severity-on-every-violation-and-render-every-breach-in-one-place.typ",
  )[both reshape Violation; land one struct change],
)

== Problem

+ Reviewing the add hint after cc77f2b7
+ 🔴 #raw("crates/mindtape/src/config_rules.rs"): \`breach\_fields\` reads a body rule's breach as about \`body\`, and the hint matches slot names against field names, so neither refusal below hints at \`-t dev\`

```
$ mt add -f dev.yaml --set kind=dev
Error: …: body.headers.rules.rollback-for-dev: a dev chore says how to undo it, in one Rollback section
$ mt add "T" --set type=dev    # template: kind: slot("type")
error: tasks/t.typ:2: rules.dev-sets-difficulty: must satisfy `kind != "dev" || difficulty == 3`
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/violation.rs").

== TO-BE

- 🟢 A `Violation` carries the field names its check read: a rule's `when` and `assert` names, a body rule's `when` names, the field a field check is about.
- 🟢 The add hint reads those names and maps each field to the slot that fills it through the template's `task.with` arguments.
- 🟢 Both refusals above print the `dev` hint.


== Changes

- `crates/mindtape-cli/tests/add_template_hint_e2e.rs`: red tests first: a body header rule whose `when` reads `kind`, and a `kind: slot("type")` template.
- `crates/mindtape/src/violation.rs`: a field set on `Violation`.
- `crates/mindtape/src/config_rules.rs`, `crates/mindtape/src/body_gate.rs`: fill it where each breach is made; `breach_fields` goes.
- `crates/mindtape-cli/src/add_hint.rs`: read the set, and map field to slot.


== Rollback

- Revert the commit; the hint parses labels again.


== Out of scope

- The hint wording.


== Dogfood

The next `dev` task filed here with plain `mt add` is refused with a pointer to `-t dev`.
