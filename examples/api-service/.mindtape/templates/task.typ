#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: slot(
    "title",
    prompt: "Starts with a verb: Throttle failed logins per IP",
  ),
  estimate: (
    src-files: slot(
      "src-files",
      kind: "int",
      prompt: "How many source files the change touches",
    ),
  ),
)

== Problem
#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps that reproduce it. Open the failing step with 🔴.",
)

== TO-BE
#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it works once this lands",
)

== Changes
#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: its `path`, then what changes there",
)

== Out of scope
#slot("out-of-scope", kind: "raw", prompt: "What this task leaves alone")
