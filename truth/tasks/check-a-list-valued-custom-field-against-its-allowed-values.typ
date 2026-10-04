#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check a list-valued custom field against its allowed values",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the machine vocabulary is held by the same [tasks.fields] mechanism as kind, not by a project's Typst function],
      cheap-adoption: helps[a project with a list-valued field deletes its check function and declares the vocabulary in config],
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

+ In a dotfiles repo, `[tasks.fields.x-machine]` declares the machine vocabulary, and six tasks carry `x-machine: ("alpha", "gamma")`.

+ With `values = ["alpha", "gamma", "beta", "delta"]`, `mt check` is run.

+ 🔴 It prints `error: tasks/rework-secrets.typ:5: x-machine: a field with allowed values takes a string`: a non-empty `values` refuses a list extra outright, so the project cannot replace its Typst `check` with config.

+ The filter language already reads a list extra (`has(x-machine, "alpha")`), so validation is the only layer that cannot.


== TO-BE

- 🟢 `[tasks.fields.x-<ns>] values` accepts a string extra or a list extra; every element must be a string in `values`.

- 🟢 A non-string element and an empty list are breaches, alongside the scalar case's `is not an allowed value`.

- 🟢 The scalar path and `@dict` references are unchanged.

- 🟢 The shape:

  ```toml
  [tasks.fields.x-machine]
  values = ["alpha", "gamma", "beta", "delta"]
  required = "!terminal"
  ```

  ```typst
  x-machine: "alpha",
  x-machine: ("alpha", "gamma"),
  ```

  ```bash
  mt check    # ok
  ```


== Changes

- `crates/mindtape/src/validate_tests_extras.rs`: red tests first — a list with every element allowed passes; one unknown element, one non-string element and an empty list each breach.

- `crates/mindtape/src/validate.rs`: `check_enum_field` accepts `HVal::Array` and checks each element, keeping the scalar path.

- `crates/mindtape/src/config_tasks.rs`: the `FieldConfig.values` doc stops saying "takes a string only".

- `www/content/concepts.typ`: a declared extension field takes a string or a list.

- `CHANGELOG.md`: the addition.


== Alternatives

- Keep string-only and let each project ship a Typst check: rejected — the filter already reads list extras, and config is the declared-field path.
- Accept any element type: rejected — `values` is a string set; a number cannot match an entry.
- Allow an empty list: rejected — it satisfies `required` while naming no value, and every `has(field, value)` is false.


== Out of scope

- `values` over dict-valued extras.
- `mt set` writing an extra.
- A scalar/list cross-check: a field that is a string on one task and a list on another stays legal.


== Dogfood

A dotfiles repo deletes `.mindtape/machines.typ` and its `[[tasks.rules]]` check for the two-line `[tasks.fields.x-machine]`.

