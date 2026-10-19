#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a task that leaves a question open",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent must ask before it files, so a task starts with every decision made],
      enforced-practice: helps[a settled spec is a rule mt check holds, not a habit someone has to remember],
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
  status: proposed(2026, 10, 19),
)

== Problem

+ Run `mt add` with a TO-BE bullet `- 🟢 Should it also cover the board?`.
+ Run `mt check`.
+ 🔴 It prints `ok`. A task can carry an open question into the work, and the agent decides it alone halfway through.


== TO-BE

- 🟢 A question mark that ends a sentence in TO-BE, Changes, Out of scope or Dogfood fails `mt check` with `ask the user and write the answer in, since a task leaves no question open`.
- 🟢 A `?` inside a code span or a code block, and anywhere in Problem, passes.
- 🟢 The rule ships in the `mindtape-tasks` pack, so a project drops or overrides it by name.


== Changes

- `crates/mindtape/src/packs/mindtape-tasks.toml`: the `no-open-questions` content rule, below.
- `crates/mindtape-cli/tests/it/`: the red test first, one task with a question in TO-BE and one with a `?` in a code span.

```toml
[[tasks.body.content.rules]]
name = "no-open-questions"
path = [{ match = "TO-BE|Changes|Out of scope|Dogfood" }]
assert = { must-not-match = '\?(\s|$)' }
when = "!terminal"
message = "ask the user and write the answer in, since a task leaves no question open"
```


== Alternatives

- A banned-phrase list such as `open question` or `TBD`. It misses a question asked in plain words, and a project can add phrases with `must-not-match` already.
- A rule in this repo's config only. Open questions are what every agent leaves behind, so the pack is the place.


== Out of scope

- Problem, which quotes commands and output where a `?` is literal.
- A new assertion kind. If content rules see code spans as text, strip them in the regex first, and file a mechanism task only when the regex can't.


== Dogfood

Every task an agent files in this repo has to settle its questions with the owner first.
