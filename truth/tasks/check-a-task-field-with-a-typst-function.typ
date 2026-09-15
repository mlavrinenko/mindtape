#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check a task field with a Typst function",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 7,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: wip(2026, 9, 16),
)

== Problem

+ Give a task the field `x-goals: (product: (cheap-adoption: "helps"),)`.
+ Add `[[tasks.rules]]` with `assert = 'has(x-goals)'` to `.mindtape/config.toml` and run `mt check`.
+ 🔴 The config fails to load with `invalid filter: expected a field, got Sub`. The filter language has no name for an `x-` field and no way into a nested dictionary, so no rule can check a structure a project brings.
+ Open `truth/lib.typ`.
+ 🔴 This repo checks goal verdicts by panicking inside `#goals(...)` in the task body: the breach has no rule name and no severity, and the verdicts cannot leave the body.


== Goals

#goals(
  product: (
    readable-tasks: helps[verdicts can sit in `task.with` beside status and links instead of filling the body],
    implementable-tasks: neutral,
    enforced-practice: helps[any structure a project invents gets a named rule, refused before `mt add` writes at `error` severity],
    cheap-adoption: neutral,
    typst-over-markdown: helps[the check is a Typst function in the repo, where frontmatter needs a separate linter],
    plain-files: neutral,
  ),
  process: (
    sound-codebase: neutral,
    docs-match-code: neutral,
  ),
)

== TO-BE

- 🟢 A rule names a Typst function instead of a filter expression and keeps `name`, `when`, `message` and `severity`:

```toml
[[tasks.rules]]
name = "goals"
when = "!terminal"
check = { typst = "truth/lib.typ", function = "check-goals", field = "x-goals" }
severity = "error"
```

- 🟢 The function receives the field's value, or `none` when the task omits it, so requiring a field needs no second rule. Without `field` it receives the whole task as `mt eval --json` prints it.
- 🟢 It returns an array of strings: empty passes, and each string is one breach, reported as `rules.goals: <string>`.
- 🟢 A panic inside the function is one breach carrying the panic message, not an eval error.


== Changes

- `crates/mindtape/src/config_rules.rs`: `RuleSpec` takes exactly one of `assert` and `check`; load-time validation evaluates the module once and refuses a missing file or an unknown function.
- `crates/mindtape/src/config_rules_typst.rs` (new): for each selected task, evaluate a synthetic source that imports the function and wraps its result in `#metadata(...) <mindtape.check>`, in a world rooted at the project through the source-override route `crates/mindtape/src/candidate.rs` uses. The value is written as a Typst literal from the task's `HVal`, so a content block arrives as its harvested plain text.
- `crates/mindtape/src/config_rules_typst_tests.rs` (new): red tests first: the Problem config, a pass, two breaches, a panic, `none` for a missing field, an unknown function at load.
- `schema/config.schema.json`: regenerate. `www/content/config.typ`: document `check` beside `assert`. `CHANGELOG.md`: an Unreleased entry.


== Alternatives

- JSON Schema per field: the goal list is copied into the schema, and "helps at least one goal" compares values across keys, which a schema says awkwardly.
- `x-` paths and `all`/`any` quantifiers in the filter language: it grows into a small programming language for one use.
- Keep panicking inside `#goals(...)`: no rule name, no severity, and the verdicts stay in the body.


== Out of scope

- `x-` field names in `mt ls` filters: no listing needs them yet.
- Moving this repo's verdicts into `x-goals`: `move-goal-verdicts-into-x-goals.typ`.


== Dogfood

Built in: `move-goal-verdicts-into-x-goals.typ` checks every open task's goals with this rule.

