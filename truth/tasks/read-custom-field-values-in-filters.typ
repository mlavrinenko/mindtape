#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read custom field values in filters",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent answers which tasks touch a host from the filter language instead of parsing mt ls --json with jq],
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
)

== Problem

+ In a dotfiles repo, `x-machine` marks each open task's host(s); `mt ls 'x-machine == "alpha"'` should list alpha's tasks.

+ 🔴 It prints `Error: unknown field in filter: x-machine`: an `x-<ns>` slot supports presence only (`has()` / `miss()`), so no expression reads its value.

+ `mt ls 'has(x-machine)'` lists all 20 tasks and cannot separate alpha's 8 from the rest.

+ The accepted ADR 002 promised otherwise: "The filter language flattens extras to path-addressed properties."


== TO-BE

- 🟢 Any `x-<ns>` value is readable, declared in `[tasks.fields.x-<ns>]` or not; the declaration adds validation and the `mt add` slot, not readability.

- 🟢 A scalar value compares like a core field: `x-machine == "alpha"`, `!=`, `<`, `>`, `matches(...)`, `in_dict(...)`, `--sort`.

- 🟢 A list value is membership through `has(x-machine, "alpha")` — true when the list holds the element, a scalar equals it, a dict has the key.

- 🟢 `==` against a list is false: `x-machine == "alpha"` selects alpha-only, `has(x-machine, "alpha")` every task touching alpha.

- 🟢 The shape:

  ```bash
  mt ls 'x-machine == "alpha"'      # alpha only
  mt ls 'has(x-machine, "alpha")'   # touches alpha
  ```

  ```typst
  x-machine: "alpha",
  x-machine: ("alpha", "gamma"),
  ```

- 🟢 A dict-valued extra stays presence-only.


== Changes

- `crates/mindtape/src/filter_extras_tests.rs`, a child of `filter_tests.rs` to keep that file under its linecop cap: red tests first — scalar extra equality, list membership through `has`, a missing extra under `!=`, a dict extra presence-only, `--sort x-machine`.

- `crates/mindtape/src/filter.rs`: `field_value` resolves an `x-<ns>` name from `task.extras`, and the module doc stops saying presence-only.

- `crates/mindtape/src/fields.rs`: `FieldValue` gains the list shape membership compares.

- `crates/mindtape/src/filter_compare.rs`: `==` / `!=` on a list; an ordering against a list is false.

- `crates/mindtape/src/filter_predicates.rs`: the two-argument `has()`, which `crates/mindtape/src/filter_eval.rs` dispatches to.

- `crates/mindtape/src/filter_strings.rs`, `crates/mindtape/src/sort.rs`: the new value shape's arms. `commit_expr.rs` matches no `FieldValue` and only calls `filter_compare`, so it needs none.

- `www/content/filters.typ` and `skills/mindtape/references/filters.md`: extras are readable, and `has()` gains its value form. `www/content/rules.typ` stops saying a filter cannot name an extra.

- `CHANGELOG.md`: the addition.


== Alternatives

- Infix `"alpha" in x-machine`: evalexpr 12.0.3 has no `in`, and a pre-lex rewrite needs operand boundaries, a mini expression parser — rejected for one operator.
- `==` as membership for a list: `==` would mean two things by value shape; the exact `==` / membership `has` split is kept.
- Declared-only readability: `[tasks.fields.x-<ns>]` gates validation, not reading; `has()` already reads undeclared names.
- `contains("alpha", x-machine)` as membership: substring, not exact; left to a separate decision.


== Out of scope

- Path-addressed flattening of dict extras, ADR 002's fuller promise.
- `--sort` for list-valued extras: no order is defined for a list.
- Extending `contains()` to extras.
- Writing extras from `mt set`.


== Dogfood

The `.nix` backlog drops its jq workaround and filters `x-machine`; MindTape's own `x-goals` stays presence-only because it is a dict.

