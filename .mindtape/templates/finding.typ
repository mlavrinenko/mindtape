#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot("title", prompt: "what should change, imperative verb first"),
  kind: "dev",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises with what staying unfiled costs",
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: the evidence below is quoted, not recalled",
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how far the fix reaches",
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
  ),
  estimate: slot("estimate", kind: "text", prompt: "(src-files: N)"),
  tags: slot("tags", kind: "list", prompt: "found, plus an area tag"),
  x-goals: (
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
      docs-match-code: slot(
        "docs-match-code",
        kind: "raw",
        default: "neutral",
      ),
    ),
  ),
)

== Problem

+ #slot(
    "context",
    kind: "text",
    prompt: "what you were doing when you noticed this, one line",
  )
+ 🔴 #raw(slot("path", prompt: "repo-relative path the finding is about")): #slot(
    "observation",
    kind: "text",
    prompt: "what is wrong, stated as one fact",
  )

#slot(
  "evidence",
  kind: "raw",
  prompt: "the evidence, quoted verbatim from what it printed — command output, a grep hit, a line count",
)

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(slot("carrier", prompt: "the file that would carry the fix")).

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
  prompt: "One bullet per file: `path`, then what changes there",
)

== Rollback

#slot(
  "rollback",
  kind: "raw",
  prompt: "How to undo this if it breaks something: what to revert or restore",
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
