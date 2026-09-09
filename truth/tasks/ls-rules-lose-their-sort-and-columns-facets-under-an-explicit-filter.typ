#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "ls rules lose their sort and columns facets under an explicit filter",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: proposed(2026, 9, 8),
)

== Summary

`crates/mindtape-cli/src/ls_rules.rs:110-116` routes any filter argument to
`explicit_bundle`, which skips rule matching entirely. That is documented
intent for the filter facet — an explicit filter should win over an
auto-activating rule. It also silently takes the rule's `sort` and
`columns` down with it, which is a different question: a rule saying "on
branch X, show these columns" stops applying the moment the caller types a
filter, and nothing in the output says so.


== Scope

Decide whether the `sort` and `columns` facets of a matching `[[ls.rules]]`
entry should survive an explicit filter argument, then make
`crates/mindtape-cli/src/ls_rules.rs` do that.

The case for splitting them: filter is selection and answers "which tasks",
while sort and columns are presentation and answer "how do I want to read
any listing". A caller narrowing the set has said nothing about
presentation. The case against: one rule is currently one bundle with one
`applied:` attribution line, and splitting the facets means a listing can
be attributed to two sources at once, which the trailer and the JSON
envelope's `applied` block both have to express.

Whichever way it lands, the `applied:` trailer must name what happened —
today a rule that matched but contributed nothing is indistinguishable
from a rule that never matched. Cover the chosen behaviour in
`crates/mindtape-cli/src/ls_rules_tests.rs` and the attribution in
`ls_rules_tests_applied.rs`.

= Open question

Should an explicit `--sort` or `--columns` flag suppress a matching rule's
corresponding facet only, or does passing either signal that the caller has
taken over presentation wholesale?

