#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title",
    example: "Retry failed invoice payments",
  ),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot(
  "summary",
  kind: "text",
  prompt: "The problem, stated once",
  default: "- TODO: what is wrong today, and why it matters",
)

== Acceptance criteria

#slot(
  "acceptance",
  kind: "text",
  default: "- TODO: a condition someone else can check without asking you",
)

== Non-goals

#slot(
  "non-goals",
  kind: "text",
  default: "- TODO: what this task deliberately will not touch",
)

== Files

#slot(
  "files",
  kind: "text",
  default: "- TODO: paths this task expects to change",
)
