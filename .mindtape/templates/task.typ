#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  kind: "task",
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  tags: slot("tags", kind: "list", default: ()),
  estimate: slot("estimate", kind: "text", default: "(files: 2)"),
)

== Summary

#slot("summary", kind: "raw", prompt: "One-paragraph summary")

== Scope

#slot("scope", kind: "raw", default: "- TODO")
