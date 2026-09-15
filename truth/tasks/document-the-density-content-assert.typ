#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document the density content assert",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "docs"),
  estimate: (files: 3),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("add-a-density-content-assert.typ"),
)

== Problem

+ Open `www/content/config.typ` at "The five content assertions".
+ 🔴 It lists five. `density`, which landed in `add-a-density-content-assert.typ`, is missing, so no project can learn it exists.
+ Run `rg -n density .mindtape/config.toml`.
+ 🔴 It prints nothing: this repo polices no prose density at all.

== Goals

#goals(
  product: (
    readable-tasks: helps[a cap of one em-dash per paragraph keeps task prose plain],
    implementable-tasks: neutral,
    enforced-practice: helps[the density rule is refused by `mt check` instead of
      living in a style note],
    cheap-adoption: neutral,
    typst-over-markdown: neutral,
    plain-files: neutral,
  ),
  process: (
    sound-codebase: neutral,
    docs-match-code: neutral,
  ),
)

== TO-BE

- 🟢 `www/content/config.typ` documents `density` beside the other assertions:
  its `pattern`, `max` and `per` keys, the `section` default, and the unit-split
  rules.
- 🟢 `.mindtape/config.toml` carries this rule, and `mt check` passes:

```toml
[[tasks.body.content.rules]]
path = [{ match = ".*" }]
assert = { density = { pattern = "—", max = 1, per = "paragraph" } }
when = "!terminal"
message = "more than one em-dash in a paragraph reads as generated prose; use a comma, a colon or a second sentence"
```

== Source of truth

- `density_units` in `crates/mindtape/src/content_gate.rs`: the unit-split
  rules (fenced code dropped, blank-line paragraphs, each list item its own
  paragraph, sentences ending at `.`, `!` or `?`).
- `schema/config.schema.json`: the key names and defaults.

== Changes

- `www/content/config.typ`: the density entry beside the other assertions.
- `.mindtape/config.toml`: the rule above, after the other
  `[[tasks.body.content.rules]]` entries. If a task filed since trips it,
  rewrite that paragraph in a separate `task:` commit rather than raising `max`.
- `CHANGELOG.md`: an entry under Unreleased.

== Out of scope

- Any change to how the assert behaves.
- Closed tasks: `!terminal` leaves history alone.

== Dogfood

Built in: the rule polices this repo's open tasks.
