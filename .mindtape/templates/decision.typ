#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

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
  estimate: slot("estimate", kind: "text", default: "(src-files: 0)"),
  x-goals: (
    product: (
      readable-tasks: slot(
        "readable-tasks",
        kind: "raw",
        default: "neutral",
        example: "helps[a listing sorted by due shows what is due first]",
      ),
      implementable-tasks: slot(
        "implementable-tasks",
        kind: "raw",
        default: "neutral",
      ),
      enforced-practice: slot(
        "enforced-practice",
        kind: "raw",
        default: "neutral",
      ),
      cheap-adoption: slot("cheap-adoption", kind: "raw", default: "neutral"),
      typst-over-markdown: slot(
        "typst-over-markdown",
        kind: "raw",
        default: "neutral",
      ),
      plain-files: slot("plain-files", kind: "raw", default: "neutral"),
    ),
    process: (
      sound-codebase: slot("sound-codebase", kind: "raw", default: "neutral"),
      docs-match-code: slot(
        "docs-match-code",
        kind: "raw",
        default: "neutral",
      ),
    ),
  ),
)

== AS-IS

#slot(
  "as-is",
  kind: "raw",
  prompt: "Numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
  + Run `mt rules --for add` and read which kinds may carry Alternatives.
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
