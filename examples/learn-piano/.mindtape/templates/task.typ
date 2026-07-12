#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Practice task title"),
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot("summary", kind: "text", prompt: "One-paragraph summary")

== Scope

#slot("scope", kind: "text", default: "- TODO")
