#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Decide the goal set for an agnostic engine",
  kind: "decision",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a goal name says who it serves without a glossary.],
      implementable-tasks: helps[the chosen set tells a task author which goal to rate and why.],
      enforced-practice: neutral,
      config-over-code: helps[the policy goal describes the engine boundary, not the file format.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[AGENTS and the docs render the chosen set from one source.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
  links: parent("rewire-goals-through-typst-markers.typ"),
)

== AS-IS

+ 🔴 `config-over-code` says policy is config, while this repo's goal DSL will be project Typst code.
+ Read `enforced-practice`. 🔴 Its title, "Checks run where work happens," describes this repo's CI as easily as an adopter's `mt` rule.
+ Read `typst-over-markdown`. 🔴 It only claims the docs first screen, not the references, functions and metadata the marker DSL uses.
+ Read any goal name. 🔴 It is a noun phrase. A task line reads `#Helps(adopter, to: cheap-adoption)`, which is not a sentence.
+ A task that changes only `.just/` can rate an adopter goal and pass.

== TO-BE

- 🟢 Every name is the infinitive of its title, so `#Helps(adopter, to: <name>)` reads as a sentence. The registry carries the infinitive and the gerund.
- 🟢 `config-over-code` becomes `keep-the-engine-agnostic`: a project's values live in config or its own Typst, never in Rust.
- 🟢 `enforced-practice` becomes `run-checks-at-the-write`: a project's rules run at the write.
- 🟢 `typst-over-markdown` becomes `beat-markdown`: a field, slot or marker reaches a rule as a value, never by parsing prose.
- 🟢 `sound-codebase` becomes `keep-changes-safe`. Closed tasks keep old keys. Eight product and two process goals.

== Changes

- `truth/tasks/2026-11/decide-the-goal-set-for-an-agnostic-engine.typ`: records the chosen set. This task changes only its own file.

```typ
read-tasks-at-a-glance: (
  to: "read tasks at a glance",
  from: "reading tasks at a glance",
  title: "Tasks read at a glance",
  acid-test: [Anyone opening an open task file can say within 30 seconds what is broken, why it matters, and how it should work instead.],
)
guide-the-implementer: (
  to: "guide the implementer",
  from: "guiding the implementer",
  title: "Tasks guide the implementer",
  acid-test: [An implementer in a fresh session, given only the task file, lands the change without asking where to start or what done looks like.],
)
run-checks-at-the-write: (
  to: "run checks at the write",
  from: "running checks at the write",
  title: "A project's rules run at the write",
  acid-test: [`mt add` and `mt flip` refuse what a rule rejects. A git-reading rule reports `skipped`.],
)
keep-the-engine-agnostic: (
  to: "keep the engine agnostic",
  from: "keeping the engine agnostic",
  title: "A project owns its policy",
  acid-test: [Every status, tag, rule, path and threshold is a config key, and a rule's function is the project's own Typst. Rust owns the vocabulary the schema spells as an enum, the canonical field names and the reserved `markers` input.],
)
adopt-in-one-line: (
  to: "adopt in one line",
  from: "adopting in one line",
  title: "Best practice adopts in one line",
  acid-test: [A repo takes the flagship's practice with one config line, overrides what it disagrees with, and gets later fixes without copying.],
)
beat-markdown: (
  to: "beat Markdown",
  from: "beating Markdown",
  title: "Typst beats Markdown with frontmatter",
  acid-test: [A field, slot or marker reaches a rule as a value, never by parsing prose. A collector may derive one; a rule reads the value. Prose rules may match markup and headings, but none re-derives a field. A misspelled helper is a compile error.],
)
keep-plain-files: (
  to: "keep plain files",
  from: "keeping plain files",
  title: "Plain files, no infrastructure",
  acid-test: [`mt` binary is the whole setup: no daemon, no database, no index, and nothing outside the repo to lose.],
)
explain-itself: (
  to: "explain itself",
  from: "explaining itself",
  title: "The CLI explains itself",
  acid-test: [A newcomer in a fresh session, given a project and `mt` but no external documentation, can list, file and flip a task, and every refusal names the command that fixes it.],
)
keep-changes-safe: (
  to: "keep changes safe",
  from: "keeping changes safe",
  title: "Changes stay safe",
  acid-test: [A change that breaks documented behavior fails `just check` before it can be committed, and every file fits its size cap.],
)
match-docs-to-code: (
  to: "match docs to code",
  from: "matching docs to code",
  title: "Docs match the code",
  acid-test: [Every doc claim about the CLI, the config or the task DSL is generated from its source, or fails a gate once the two drift.],
)
```

The DSL and migration leaves copy the exact `(to, from, title, acid-test)` strings into the registry. The name is the kebab of `to`, lowercased, so `beat-markdown` keeps its capital in the render and loses it in the key. They reserve `markers` as the one Rust-owned check input. The exception list is the filter functions, canonical field names, header actions, content asserts, slot kinds and that input. Every other input is a config key or an `x-` slot. They are follow-ons, not this task's changes.

== Alternatives

- Keep `config-over-code` and widen only its acid test. Cheapest, but the name still says TOML.
- Split it into `keep-the-engine-agnostic` and `own-the-policy`. More precise, but the two goals move together on almost every task.
- Name the engine goal `policy-in-project` or `mechanism-not-policy`. Those name the rule, not the property.
- Keep the noun-phrase names and let the DSL add the verb. Rejected: `#Helps(adopter, to: cheap-adoption)` is not a sentence.
- Add `consistent-tree` as a third process goal. Rejected: it is a property of the result, not a purpose, and every new goal re-migrates the backlog.

== Out of scope

- The marker capability and the DSL implementation.
- Migrating closed tasks.
- Adding `consistent-tree` after this decision.

== Dogfood

The next open task rates `keep-the-engine-agnostic` and `run-checks-at-the-write`, and a maintainer-only gate has no adopter goal to hide behind.
