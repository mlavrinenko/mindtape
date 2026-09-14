#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Retire more than one task in one mt rm",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

`mt rm [TARGET]` takes a single optional target, so retiring two tasks by name costs two commands. A filter expression already retires every match, so the capability is there and only the ergonomic is missing — and `mt flip` already takes a list of refs, which is the shape a reader expects here too.

== Scope

Make TARGET variadic, resolving each entry as a reference or a filter and retiring the union, with the confirmation prompt and the SWHID tombstone rewrite covering the whole set in one pass. Red test: two refs in one invocation retire both and rewrite the inbound links for each.
