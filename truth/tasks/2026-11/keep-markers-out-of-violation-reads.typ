#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Keep the markers pseudo-field out of violation reads",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("crates/mindtape/src/rules/eval.rs"): `rule_reads` inserts the reserved `markers` input into `Violation.reads`, whose contract says task fields.

`rule_reads` does `reads.insert(check.field.clone()?)`. `Violation.reads` documents the task fields, and `mt check --json` exposes it.

== TO-BE

- 🟢 `markers` is either absent from `Violation.reads` or documented as a reserved input alongside the task fields.
- 🟢 The add hint and the pre-write consumers keep their behavior, and the public JSON stays truthful.

```json
{"field": "rules.give-every-goal-verdict", "reads": ["markers"]}
```


== Changes

- `crates/mindtape/src/rules/eval.rs`: skip the reserved field or map it to a documented pseudo-name.
- `crates/mindtape/src/rules/typst_tests.rs`, `schema/output/check/1.schema.json` if the contract needs a note.

== Out of scope

- The goal check itself. This is the rule engine's read set.

== Dogfood

A consumer reading `reads` sees task fields, not a reserved input disguised as one.

== Goals

+ #Helps(adopter, to: keep-the-engine-agnostic)[the public output stays a task-field set].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

