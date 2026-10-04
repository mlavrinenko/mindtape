#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Offer config values for a template's own enum slot",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a template names a field's example without copying the field's allowed values, and the caller still sees the list],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the template check no longer depends on the order of a config list],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 4),
)

== Problem

+ `.mindtape/templates/task.typ` leaves `kind` out, so `mt add` splices a choice over `[tasks.fields.kind] values`, and the template check files it with the first value.

+ `.mindtape/config.toml` lists `test` first only so that check passes, as its comment admits.

+ Declaring `kind: slot("kind", example: "test")` in the template instead loses the list:

  ```
  $ mt add -t task "x" -- kind=chore
  error: x: kind: `chore` is not an allowed value
  ```

+ 🔴 `--describe` and the editor stop offering the values, and the refusal stops naming them.


== TO-BE

- 🟢 A template slot that is the value of a `task.with` field with config `values` (`kind`, an `x-<ns>` field), and declares no `choices` of its own, is a choice over those values:

  ```typ
  kind: slot("kind", example: "test"),
  ```

  ```
  $ mt add -t task "x" -- kind=chore
  Error: slot "kind" value "chore" is not one of: feat, fix, refactor, docs, test, dev, milestone
  ```

- 🟢 `--describe` lists the choices, and the template check, `mt add` and the editor all see the same spec.

- 🟢 `task.typ` declares `kind` with `example: "test"`, and `[tasks.fields.kind] values` returns to its own order with the ordering comment gone.


== Changes

- `crates/mindtape-cli/tests/add_inherited_choices_e2e.rs`: red tests first — a template's own `kind` slot refuses a value outside config and lists the choices; `--describe` shows them. `add_config_scaffold_e2e.rs` sits at its line cap, so they get a file of their own.

- `crates/mindtape/src/add_slots.rs`: `inherit_field_choices` resolves the field a slot sits in and inherits the field's config values as its choices.

- `crates/mindtape-cli/src/add.rs`, `crates/mindtape-cli/src/check_templates.rs`: `splice_config_slots` inherits the choices and splices the required slots, the one step `mt add` and the template check share.

- `.mindtape/templates/task.typ`, `.mindtape/config.toml`: the declared slot, and the list in its own order.

- `www/content/templates.typ`, `CHANGELOG.md`: the inherited choices.


== Alternatives

- Copy the values into the slot's `choices`: a second list that drifts.
- A plain string slot: config still validates, but the caller loses the list.
- Keep the config order: the template check depends on list order.


== Out of scope

- Inheriting a numeric scale or framework inputs.


== Dogfood

`task.typ` declares its `kind` slot, and `just check` runs the template check on it.
