#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "split or re-cap <path>, or rewrite <path> under its cap for prose",
    example: "Split crates/mindtape-cli/src/cmd/add/mod.rs",
  ),
  kind: "refactor",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises as headroom falls",
      example: 5,
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: linecop measures, it does not guess",
      example: 0.9,
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how much code must move",
      example: 6,
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
    example: 2,
  ),
  estimate: slot(
    "estimate",
    kind: "text",
    prompt: "(src-files: N)",
    example: "(src-files: 2)",
  ),
  assignee: "agent",
  tags: slot(
    "tags",
    kind: "list",
    prompt: "swept, size, plus tests for a test module",
    example: ("swept", "size"),
  ),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a file under its cap changes without reading its
        neighbors],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Run `linecop --format json`.
#slot(
  "measured",
  kind: "raw",
  prompt: "one numbered 🔴 line per measured file: a doc source and its render share a finding",
  example: ```typ
  + 🔴 #raw("crates/mindtape-cli/src/cmd/add/mod.rs") stands at 494 of its 500-line budget: 6 lines of headroom left, 98.8% of the cap spent.
  ```,
)

#slot(
  "seam",
  kind: "text",
  prompt: "one plain sentence on where the seam question stands",
  example: "The fill path and the header injection read as two seams; nothing settles which to carve first.",
)

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just lint::size` passes and `linecop --baseline 90` no longer names
  this file.

== Changes

#slot(
  "remedy",
  kind: "raw",
  prompt: "the two language-specific bullets: what a remedy is here, and what a split has to be",
  example: ```typ
  - Carve the header injection into its own module, or re-cap the file in `.linecop.yaml`.
  - A split moves whole functions with their doc comments, so each file reads on its own.
  ```,
)
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Behavior changes: a split moves code and changes nothing it does.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
