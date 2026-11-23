#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Guide a wrong-audience verdict without a mechanical fix",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(
    2026,
    11,
    23,
  )[forced past failing gate: "mmz --is-fresh --tag gate"],
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("lib/prelude.typ"): the wrong-audience message always suggests `#Helps`, which flips a `Hinders` or `Neutral` into a help.

`#Hinders(adopter, from: keep-changes-safe)[why]` reports `write #Helps(maintainer, to: keep-changes-safe)`. Following it changes both the effect and `from:` to `to:`.

== TO-BE

- 🟢 The message quotes the goal's audience and the verdict's audience, then asks the author to reconsider the goal, the audience or the task's framing.
- 🟢 It does not give a copy-paste replacement. A mechanical swap would hide a bad verdict.
- 🟢 The test pins a `Hinders` and a `Neutral` wrong-audience case and asserts the message does not contain `#Helps`.

```text
error: goal `keep-changes-safe` is maintainer work, rated adopter.
Reconsider the goal or the task: a `Hinders` for this goal is not a `Helps`,
and swapping the audience is not a fix.
```


== Changes

- `truth/goal-dsl.typ` after the core extraction, `lib/prelude.typ` today: rewrite the wrong-audience message.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`: pin the message for a `Hinders` and a `Neutral`.

== Out of scope

- Auto-fixing the audience. The author must decide whether the verdict or the goal is wrong.

== Dogfood

A confusing verdict gets a question, not a wrong edit that passes the gate.

== Goals

+ #Helps(adopter, to: guide-the-implementer)[the message guides a decision instead of a mechanical swap].
+ #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

