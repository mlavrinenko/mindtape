#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: slot("title", prompt: "Task title, imperative verb first"),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

#slot(
  "why",
  kind: "text",
  prompt: "Why the change, one line",
)
