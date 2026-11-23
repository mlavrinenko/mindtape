#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Rename the self-describing goal in the skill eval README",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (task: "remove-the-x-goals-exemption.typ", kind: "surfaced-by", note: none),
  ),
  tags: ("found", "docs", "stateless", "no-repro", "no-ui-ux"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Removing the old-name goal aliases from truth/goal-calls.typ.
+ 🔴 #raw("skills/mindtape/evals/README.md"): the eval README references the `self-describing` goal, which `truth/goal-calls.typ` does not bind. The live goal is `explain-itself`.

```
grep -n self-describing skills/mindtape/evals/README.md
140:These blind evals support the `self-describing` goal in `truth/goals.typ`.
```


== TO-BE

- 🟢 The eval README names `explain-itself`, the goal `truth/goals.typ` registers.


== Changes

- `skills/mindtape/evals/README.md`: replace `self-describing` with `explain-itself`.


== Out of scope

- The eval contents. Only the goal reference is stale.


== Dogfood

The skill's evals point at a goal the live registry has.


== Goals

+ #Helps(maintainer, to: match-docs-to-code)[because a reader looking up the evals finds the goal the registry registers].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).

