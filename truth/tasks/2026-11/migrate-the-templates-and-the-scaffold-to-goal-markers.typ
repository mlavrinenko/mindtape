#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Migrate the templates and the scaffold to goal markers",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "tests", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 21),
  links: parent("rewire-goals-through-typst-markers.typ")
    + depends-on(
      "migrate-the-open-backlog-and-switch-the-goal-rule.typ",
      "carry-body-markers-through-the-candidate-door.typ",
    ),
)

== AS-IS

+ 🔴 `.mindtape/templates/dev.typ` has ten `x-goals` slots. An author fills the whole scorecard in the header before the body.
+ Give a raw slot a multi-line default, then run `mt add`. 🔴 `scaffold.rs` writes `# default: {default}` on one line. The comment swallows the block.
+ Run `just advisory ask-refactoring --write`. 🔴 The five arms pass no goals value, and the sweep templates hardcode `x-goals`.

== TO-BE

- 🟢 Every active template has one `goals` raw slot after `Dogfood`, `required: true`, with no default: the author fills it by hand or with `--set`, and the block rates every goal. Its `example:` shows the complete ten-goal block, one helps and nine neutrals. The slot's value carries the `== Goals` heading, so the template body has no literal one.
- 🟢 The scaffold's `# default:` line uses `push_comment_lines`, and a `splices_verbatim` default shows the dedented `default_string`. A multi-line default or `--set` value stays a block scalar.
- 🟢 The sweep templates take the same slot, and each arm passes its own rated value through one `goals_with_one_helps <audience> <goal> <reason>` shell helper.
- 🟢 `tweak.typ` stays unchanged. A tweak lands as done, so no goal check reaches it.

```typ
== Goals

+ #Helps(adopter, to: guide-the-implementer)[because one template slot carries the whole goal block].
+ #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
```

== Changes

- `.mindtape/templates/*.typ`: one `goals` raw slot after `Dogfood`, `required: true`, no default, and an `example:` holding the complete ten-goal block with one helps. `required` is true because an optional raw slot with no default fills as the literal `none`. The slot's value carries the `== Goals` heading, so the body has no literal one. Import generation 2. `tweak.typ` unchanged.
- `crates/mindtape-cli/src/cmd/add/scaffold.rs`: `# default:` uses `push_comment_lines`, and a `splices_verbatim` default shows the dedented `default_string`. A multi-line `splices_verbatim` seed renders a block. Today a multi-line raw default's lines are not comment-prefixed. The scaffold then stops parsing as YAML: a correctness bug.
- `crates/mindtape/src/template/mod.rs`: `SlotKind::splices_verbatim` becomes `pub`. The scaffold then reads the same `Text | Raw` pair the fill does. No second predicate.
- `crates/mindtape-cli/src/cmd/add/scaffold_tests.rs`, `crates/mindtape-cli/tests/it/add_edit_rule_e2e.rs`: the scaffold parses as YAML with a multi-line raw default, comments on and off. A `--set` newline renders a block.
- `.mindtape/templates/sweep-*.typ`: the hardcoded `x-goals` becomes the same `goals` slot, and each arm passes its rated value. `contrib/ask-refactoring.sh` gains `goals_with_one_helps <audience> <goal> <reason>`. Each arm's `args` in `contrib/ask-refactoring/*.sh` gains `--set "goals=$(...)"`. The arms pass no goals value today.
- `crates/mindtape-cli/tests/it/add_config_scaffold_e2e.rs`: copy generation 2's chain and pass a goals value.
- `www/content/templates.typ`, `skills/mindtape/references/hand-editing.md`: the `goals` slot, its example and the new form.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- The backlog, the registries and the rule switch. The migration leaf.
- A redesign of the interactive prompt. The goals slot has no default, so `mt add` refuses a blank one and the author rates every goal. `[templates] check-defaults` stays false: the slot has no default, so the default variant is the example variant.
- Migrating the example projects. They keep the adopter `x-goals` form.

== Dogfood

The next `mt add` writes the new form, and `just advisory ask-refactoring --write` files a swept task that passes `mt check`.

== Goals

+ #Helps(adopter, to: guide-the-implementer)[one template slot carries the whole goal block instead of ten fields].
+ #Helps(adopter, to: run-checks-at-the-write)[`mt add` writes a task the marker check reads].
+ #Helps(maintainer, to: match-docs-to-code)[the templates page and the hand-editing reference show the new form].
+ #Neutral(read-tasks-at-a-glance, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).
