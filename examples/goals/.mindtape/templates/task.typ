#import "@local/mindtape:0.5.0": *
#import "/goal-calls.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "What changes, starting with a verb",
    example: "Ship the search index",
  ),
)

== Goals

#slot(
  "goals",
  kind: "raw",
  default: "+ #Neutral(answer-fast, avoid-lock-in).",
  prompt: "One verdict per goal: #Helps(to: <goal>)[why] or #Neutral(<goal>).",
  example: "+ #Helps(to: answer-fast)[results appear while you type].\n  + #Neutral(avoid-lock-in).",
)

= Summary

#slot(
  "summary",
  kind: "raw",
  prompt: "Why the task exists, in a paragraph",
  example: ```typ
  Search scans every note on every keystroke.
  ```,
)
