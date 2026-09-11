#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Look a value up in a dictionary from a filter expression",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("mt",),
  estimate: (files: 9),
  assignee: "agent",
  status: wip(
    2026,
    9,
    11,
  )[teaching the filter walker string-valued arguments: word(), matches(), in\_dict()],
  links: depends-on("add-named-dictionaries-to-the-configuration.typ"),
)

== Summary

The filter language compares a field against a literal and nothing else. Every
function it has returns a boolean, and every argument it accepts is a field
name or a literal — so there is no way to say "the first word of the title",
and therefore no way to ask a dictionary about it.

This is the piece that makes `[dicts]` reachable from an expression:
`in_dict("verbs", word(title, 1))`, evaluated wherever a filter already runs —
`mt ls`, a view, a limit, a `required` rule, a `when` gate.


== Scope

+ Teach the expression walker string-valued arguments, so a function may take
  the result of another call rather than only an identifier or a literal. This
  is the load-bearing change; the three functions below are what it buys.
+ Add `word(field, n)`: the n-th whitespace-separated word of a string field,
  1-based, with surrounding punctuation trimmed. It takes no position on Typst
  markup or code spans — a title opening with a backticked command yields that
  span verbatim, and exempting it is the rule author's business.
+ Add `matches(field, "regex")`, the escape hatch a closed vocabulary needs.
+ Add `in_dict("name", <string expr>)`: membership in a configured or built-in
  dict, honouring that dict's `case`. An unknown dict name is an error naming
  it, raised at parse time like every other unknown name in the language.
+ Document all three on the filter page, and cover the cross product that
  matters: a missing field, a word index past the end, a `fold` dict against a
  capitalised word, and an `exact` one against the same.

