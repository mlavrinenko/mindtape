#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Validate every task against config declared rules",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("mt",),
  estimate: (files: 12),
  assignee: "agent",
  status: proposed(2026, 9, 11),
  links: depends-on(
    "look-a-value-up-in-a-dictionary-from-a-filter-expression.typ",
  ),
)

== Summary

Every constraint MindTape enforces today is one the tool knows by name — a
status legend, a numeric scale, a tag allow-list, a heading path. A project
that wants to hold any other convention has nowhere to put it.

`[[tasks.rules]]` is the general form: an assertion written in the filter
language that must hold for every task the rule selects. It constrains no
field in particular, because the expression already names the field it is
about.


== Scope

+ Add `[[tasks.rules]]` with `name`, `assert` (a filter expression that must
  evaluate true), `when` (an optional filter gating which tasks the rule
  applies to), `message` and `severity` (`warn`, the default, or `error`).
+ Evaluate rules inside the validation pass, not in `mt check` alone. Every
  write command validates before it writes, so `mt add`, `flip`, `link`,
  `import`, `recur` and `review` all enforce a rule for free — a convention
  that only fails hours later at check time is a convention nobody keeps.
+ Report a breach as `<file>:<line>: rules.<name>: <message>`, prefixed
  `error:` or `warning:` by severity. A warning prints and passes; an error
  fails the command. A rule with no `message` falls back to naming its own
  `assert` expression.
+ Dry-run each `assert` and `when` once at config load against a probe task,
  the way `required` expressions already are, so a malformed rule is one
  config error rather than one violation per artifact.
+ Document the section on the config page, and cover: an assert that holds, one
  that fails, `when` excluding the offending task, both severities against the
  exit status, a rule reaching a dict, and a malformed expression refused at
  load.

