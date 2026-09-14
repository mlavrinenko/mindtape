#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "split or re-cap <path>"),
  kind: "task",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises as headroom falls",
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: linecop measures, it does not guess",
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how much code must move",
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
  ),
  estimate: slot("estimate", kind: "text", prompt: "(files: N)"),
  assignee: "agent",
  tags: slot(
    "tags",
    kind: "list",
    prompt: "swept, size, plus tests for a test module",
  ),
)

== Summary

#raw(slot("path", prompt: "repo-relative path of the oversized file")) stands at #slot("lines", kind: "text", prompt: "current line count") of its #slot("limit", kind: "text", prompt: "the linecop limit for this file")-line budget: #slot("headroom", kind: "text", prompt: "limit minus lines") lines of headroom left, #slot("percent", kind: "text", prompt: "lines over limit, as a percentage")% of the cap spent.

#slot(
  "seam",
  kind: "text",
  prompt: "one plain sentence on where the seam question stands",
)

`just ask-refactoring` filed this from `linecop --format json`, and nothing
recorded that it did. The sweep re-reads the signal every run, so this task is
worth exactly as much as the measurement behind it still is.

== Scope

#slot(
  "remedy",
  kind: "raw",
  prompt: "the two language-specific bullets: what a remedy is here, and what a split has to be",
)
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep — there is no ignore list anywhere else, deliberately.
- Done when `just check-file-size` passes and `linecop --baseline 90` no longer
  names this file.
