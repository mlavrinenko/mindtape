#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Close the section allowlist to the four spec sections",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "gating"),
  estimate: (files: 1),
  kind: "dev",
  assignee: "agent",
  status: done(
    2026,
    9,
    15,
  )[The allowlist closed straight onto the six sections: every open task moved in the same change, so no union step was needed.],
  links: depends-on(
    "move-the-sweep-templates-to-the-four-section-spec.typ",
    "rewrite-open-task-bodies-into-the-four-section-spec.typ",
  ),
)

== Summary

The four-section spec landed beside Summary and Scope so the backlog could migrate without a red `mt check`. Once no open task and no template carries the old pair, keeping it in the allowlist only lets the next task grow it back.


== Scope

Precondition: this listing prints nothing.

```sh
rg -l '^== (Summary|Scope)$' $(mt ls '!terminal' --format path) .mindtape/templates
```

Then, in `.mindtape/config.toml`, drop Summary and Scope from the top-level allowlist, and delete the Summary `unique` rule, the Summary-before-Scope `order` rule, and the Scope `min-words` and gating fenced-block content rules the TO-BE rules replaced.

== Dogfood

Built in: the closed set lands in this repo's own `.mindtape/config.toml`.
