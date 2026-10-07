#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Write rule messages as the next step",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Run `mt check` on a task whose Environment section sits before its Problem.
+ 🔴 It prints `rules.environment-follows-problem: Environment follows Problem`, a fact where the reader needs the fix.

== TO-BE

- 🟢 Every `message` in the shipped pack and this repo's config is the next step in the imperative, lower case, with no trailing period or em dash: `move Environment after Problem`.
- 🟢 The config page states that rule for authors.

== Changes

- `crates/mindtape/src/packs/mindtape-tasks.toml`: reword the 24 messages the inventory lists.
- `.mindtape/config.toml`: the gate message at line 570 drops the `--force` advice `mt` now prints, and `Dogfood follows Out of scope` becomes `move Dogfood after Out of scope`.
- `examples/api-service/.mindtape/config.toml`: its two messages.
- `www/content/config.typ`: the authoring rule for `message`.
- Regenerate README and the site with the docs recipe.

== Out of scope

- Rust messages: the other fan-out tasks.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
