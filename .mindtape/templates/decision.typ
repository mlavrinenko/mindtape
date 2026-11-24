#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title, imperative verb first",
    example: "Record the retry policy in a decision task",
  ),
  kind: "decision",
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  tags: slot("tags", kind: "list", default: ("stateless",)),
)

== AS-IS

#slot(
  "as-is",
  kind: "raw",
  prompt: "One plain line that carries the essence first, then numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
  Only `feat` may carry Alternatives, so a decision is filed as a feature.

  + Run `mt config show --resolved` and read which kinds may carry Alternatives.
  + 🔴 Only `feat` may carry Alternatives, so a decision is filed as a feature.
  ```,
)

== TO-BE

#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: the call this task records, and how the repo works once it lands",
  example: ```typ
  - 🟢 `decision` is a kind, so a decision closes in a task-only commit.
  ```,
)

== Changes

#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: `path`, then what changes there; name this task file and say no code changes",
  example: ```typ
  - `truth/tasks/record-the-retry-policy-in-a-decision-task.typ`: records this decision; no code changes.
  ```,
)

== Alternatives

#slot(
  "alternatives",
  kind: "raw",
  prompt: "Bullets: each design considered, and why it lost",
  example: ```typ
  - Gate by commit type: a decision still needs a `feat` to carry Alternatives.
  ```,
)

== Out of scope

#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "Bullets: what not to touch, and why",
  example: ```typ
  - The implementation. A separate task makes this decision real.
  ```,
)

== Dogfood

#slot(
  "dogfood",
  kind: "raw",
  prompt: "How this repo will use the change, or Not applicable and why",
  example: ```typ
  This repo files its next architectural call as a `decision` and closes it with a task-only commit.
  ```,
)

#slot(
  "goals",
  kind: "raw",
  required: true,
  prompt: "Under `== Goals`, rate every goal: one `#Helps` with a reason, then `#Neutral` for the rest",
  example: ```typ
  == Goals

  + #Helps(adopter, to: guide-the-implementer)[because one template slot carries the whole goal block].
  + #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
  ```,
)
