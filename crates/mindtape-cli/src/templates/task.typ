#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title"),
  priority: slot("priority", kind: "int", default: 5),
  tags: slot("tags", kind: "list", default: ()),
)

== Summary

#slot("summary", kind: "text", prompt: "One-paragraph summary")

== Scope

#slot("scope", kind: "text", default: "- TODO")
