#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot("title", prompt: "Task title, imperative verb first"),
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  tags: slot("tags", kind: "list", default: ()),
  estimate: slot("estimate", kind: "text", default: "(files: 2)"),
)

== Problem

#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
)

== Goals

#goals(
  product: (
    readable-tasks: slot("readable-tasks", kind: "raw", default: "neutral"),
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
    docs-match-code: slot("docs-match-code", kind: "raw", default: "neutral"),
  ),
)

== TO-BE

#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it should work, behave and feel once this lands",
)

== Changes

#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: `path`, then what changes there; the red test first",
)

== Out of scope

#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "Bullets: what not to touch, and why",
)

== Dogfood

#slot(
  "dogfood",
  kind: "raw",
  prompt: "How this repo will use the change, or Not applicable and why",
)
