#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check x- extras from the field config",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a project's x- field is held to its configured values and required rule by mt check and every write command, not by a Typst function someone has to ship],
      cheap-adoption: helps[an adopting repo declares an x- enum in one [tasks.fields] table instead of writing and maintaining a Typst check],
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

+ In a scratch project, declare a field the task DSL never declared:

  ```toml
  [tasks.fields.x-machine]
  values = ["alpha"]
  required = true
  ```

+ File one task without `x-machine` and one with `x-machine: "desktop"`, then run `mt check`.

+ 🔴 It prints `ok: 2 task(s) valid`. The table parses (`crates/mindtape/src/config_tasks.rs:251`) but `check_enum_field` (`crates/mindtape/src/validate.rs:231`) reads only `kind`, and no load-time check refuses the name, so the declaration is silently inert.


== TO-BE

- 🟢 `[tasks.fields.<name>]` constrains an `x-<ns>` extra exactly as it constrains a core enum field: `values` takes literals or `@dict` references, and `required` takes `true`, `false` or a filter expression.

- 🟢 The shape:

  ```toml
  [tasks.fields.x-machine]
  values = ["@machines"]
  required = "!terminal"
  ```

- 🟢 `mt check` and every write command report a declared extra that is missing or outside `values`, at the rule's severity, like any core field.

- 🟢 One table, one `required` grammar and one `@dict` expansion serve core fields and extras alike; the Typst `check` stays for shapes `values` cannot compare.

- 🟢 A `[tasks.fields.<name>]` key that is neither a core field nor an `x-<ns>` name fails config load naming the key, so the silent no-op is gone.


== Changes

- `crates/mindtape/src/validate_tests_fields.rs`: red tests first — a declared `x-` extra that is missing, one outside `values`, an `@dict`-expanded list, and an unknown field name refused at load.

- `crates/mindtape/src/validate.rs`: `check_enum_field` reads an `x-<ns>` name from `task.extras`, compares an `HVal::Str` against `values`, and reports the `required` miss like a core field; a non-string extra with `values` set is a breach.

- `crates/mindtape/src/config_tasks.rs`: refuse every `[tasks.fields.<name>]` key that is neither a core field nor an `x-<ns>` name, at config load.

- `examples/named-sets/.mindtape/config.toml` and its tasks: a declared `x-` extra backed by `@dict`, so `check-examples` runs the path.

- `schema/config.schema.json`: regenerate; the `tasks.fields` description stops saying core-only.

- `www/content/config.typ`, `www/content/concepts.typ`, `www/content/rules.typ`: document the extras enum and required path; correct "mt never checks it".

- `CHANGELOG.md`: an Added entry.


== Alternatives

- Keep the Typst `check` as the only path: basic enum and required validation would still need a shipped function per project.
- Add a separate `[tasks.extras.<name>]` section: a second table for the same job, against the one-mechanism request.
- Only refuse `x-` names in `[tasks.fields]`: removes the silence, not the need.
- Promote every custom field to a core field: core growth for each consumer's field, which extras exist to avoid.


== Out of scope

- Writing extras from the CLI (`mt set x-machine=…`, an `mt add` slot): this task checks authored values only.
- Filter and sort access to extras, which stay `has`/`miss` in the filter language.
- Constraints `values` cannot express: nested dicts and regexes stay on a Typst `check`.


== Dogfood

`examples/named-sets` gains a declared `x-` extra backed by a `@dict`, so `just check` runs the path through `check-examples` on every gate.

