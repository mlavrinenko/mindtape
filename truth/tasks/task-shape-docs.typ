#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document the reshaped task sections and the tag index",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: helps[the docs and the skill name the same shape the rules enforce],
      implementable-tasks: neutral,
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
  status: proposed(2026, 11, 1),
  links: depends-on("add-mt-tags.typ", "task-templates.typ"),
)



== Problem

+ Run `mt rules --for add` and read the section names the docs still use.
+ 🔴 `www/content/require-a-task-shape.typ` and `packs.typ` still say `Problem` and a `Rollback` for every `dev` task.
+ 🔴 The `mt tags` command and the opt-out tags are absent from the skill, and the eval still names the `problem` slot.

== TO-BE

- 🟢 The docs name `AS-IS`, the statefulness tags and the `mt tags` command.
- 🟢 The api-service fixture, its templates and evals show a fenced `TO-BE` example and the JWT task's `stateful` Rollback.
- 🟢 The skill tells an agent to run `mt tags --for add` before filing.

== Source of truth

- `crates/mindtape/src/packs/tasks.toml` and `dogfood.toml`: the section names and rules the pages quote.
- `mt tags --for add` and `mt add --describe`: the live prompts the skill quotes.

== Changes

- `www/content/require-a-task-shape.typ`, `www/content/packs.typ`: the section names, the kind list and the tag index.
- `docs/src/contributing.typ`, `docs/src/contributing/commit-messages.typ`: the `decision` kind and the retired tag.
- `docs/src/skill.typ`: the filing steps and the `mt tags` command.
- `skills/mindtape/evals/evals.json`: the slot names and the ruling examples.
- `examples/api-service/tasks/*.typ` and `examples/api-service/.mindtape/templates/*.typ`: the fenced examples and the `stateful` JWT task.

== Out of scope

- The pack and template sources in `task-section-rules` and `task-templates`, and the corpus in `as-is-sweep`.

== Dogfood

The docs site is this repo's user-facing surface for the practice.
