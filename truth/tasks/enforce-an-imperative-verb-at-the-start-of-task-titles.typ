#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Enforce an imperative verb at the start of task titles",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: (),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 11),
)

== Summary

`mt check` validates statuses, links, tags and body headers, but nothing
constrains the grammatical shape of a title. blustroke has adopted
verb-first titles ("Wire the cross target", not "cross target wiring") and
wants the tracker to hold the convention instead of a person's memory.


== Scope

+ Add a title rule to the configuration (e.g. `[tasks.title] mood =
  "imperative"`, backed by a first-word verb list or allow-list) enforced
  by `mt check` with a message that names the offending title.
+ Default to `warn` so existing backlogs survive; let a project raise it
  to `error`.
+ Cover it: a verb-first title passes, a noun phrase fails, an allow-listed
  or unlisted verb behaves as configured, and a first word that is code or
  a proper noun is handled deliberately.

