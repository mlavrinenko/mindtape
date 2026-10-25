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
  status: done(2026, 10, 25),
  links: depends-on(
    "skip-inline-code-in-a-content-density-count.typ",
    "keep-a-content-rules-detail-under-its-custom-message.typ",
  ),
)

== Problem

+ Run `mt add` with a TO-BE bullet `- 🟢 Should it also cover the board?`.
+ Run `mt check`.
+ 🔴 It prints `ok`. A task can carry an open question into the work, and the agent decides it alone halfway through.


== TO-BE

- 🟢 A question mark that ends a sentence in TO-BE, Changes, Out of scope or Dogfood fails `mt check` with `ask the user and write the answer in, since a task leaves no question open`, and a `note:` under it quotes the sentence.
- 🟢 A `?` inside a code span or a code block, and anywhere in Problem, passes.
- 🟢 A terminal task is exempt, so the closed backlog needs no rewrite.
- 🟢 The rule ships in the `mindtape-tasks` pack, so a project drops or overrides it by name.


== Changes

- `crates/mindtape-cli/tests/it/`: the red test first. A TO-BE bullet `- Should it also cover the board?` fails with the message and the note. A `?` in a code span, in a fenced block and in Problem passes.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: the `no-open-questions` content rule, below.

```toml
[[tasks.body.content.rules]]
name = "no-open-questions"
path = [{ match = "TO-BE|Changes|Out of scope|Dogfood" }]
assert = { density = { pattern = '\?(\s|$)', max = 0, per = "sentence" } }
when = "!terminal"
message = "ask the user and write the answer in, since a task leaves no question open"
```


== Alternatives

- A `must-not-match` rule. It reads the raw section, code included, so every quoted command with a `?` would trip it.
- A banned-phrase list such as `open question` or `TBD`. It misses a question asked in plain words, and a project can add phrases with `must-not-match` already.
- A rule in this repo's config only. Open questions are what every agent leaves behind, so the pack is the place.


== Out of scope

- Problem, which quotes commands and output where a `?` is literal.
- A new assertion kind. `density` with `max = 0` per sentence already says "no sentence asks", once it skips code.


== Dogfood

Every task an agent files in this repo has to settle its questions with the owner first.
