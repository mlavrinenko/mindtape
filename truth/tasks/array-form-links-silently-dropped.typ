#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "A links: field in array form is silently dropped",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 8, 20),
)

== Summary

A `links:` field written as a Typst array is parsed as no links at all:

```typ
links: (
  depends-on("a.typ"),
  depends-on("b.typ"),
),
```

Both edges vanish and `mt check` reports the task valid. The spelling that
works is `+` concatenation — `depends-on("a.typ") + depends-on("b.typ")` —
but nothing says so at the point of failure and nothing complains.

== Why

The array form is the shape a Typst author would reach for first, since it is
what every other multi-valued field in a task takes: `tags: ("a", "b")` is an
array, and so is `estimate`. Guessing the same for `links:` produces a task
that validates clean while its dependency graph is quietly empty — so
`mt ls ready` lists work that is actually blocked, which is the one question
the graph exists to answer.

Silent is the problem, not the syntax. An error would have cost one attempt.

== Scope

- Reject the array form with an error naming the field and showing the `+`
  spelling, or accept it as an alias. Either is fine; silence is not.
- Audit the other fields taking a composite value for the same class of
  silent drop.
- Regression test: a task whose `links:` is an array must either fail
  `mt check` or resolve its edges — never validate carrying zero links.

== Notes

Hit while filing the first tasks in a new project: two of four tasks were
authored with array-form links, `mt check` passed, and the mistake surfaced
only because `mt ls blocked` came back empty when it should have listed both.
