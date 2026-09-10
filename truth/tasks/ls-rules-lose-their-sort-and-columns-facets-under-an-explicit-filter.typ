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
  status: done(2026, 9, 10),
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

= Decision

Split them. `resolve` runs the cascade first and layers the explicit filter
over the `filter` facet alone, so a rule (or the default view, which had the
same hole) goes on supplying the sort and columns of a listing the caller
narrowed. A bare `@view` argument still overrides every facet that view sets:
naming a view is choosing a reading, not just a set of rows.

The case against turned out to cost nothing — `Applied` was already per-facet
and `trailer` already grouped by source, so two sources on one listing needed
no new shape. What it did need was a label: a rule's own overrides used to be
attributed to `manual`, which is also the CLI's label, so a filter from the
command line and a sort from a rule override would have grouped into one
`manual (filter, sort)`. A rule's overrides now carry the rule's label, and
`rule_label` falls back to `rule` rather than the empty string, which is no
longer printable as a source.

The open question answers itself from the same principle: a flag suppresses
its own facet only, which is what `apply_cli_overrides` already did.

`Applied::rule` is now set whenever a rule matched, contribution or not, and
`rule_superseded` records that everything it gave was overridden — the trailer
prints `· <rule> superseded`, `mt ls --json` carries `"superseded"`.

