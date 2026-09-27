#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a transition into done while blocked",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli"),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[finishing or starting blocked work needs an explicit --force],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
)

== Problem

+ `mt link b depends-on a` with `a` still proposed.
+ 🔴 `mt flip wip b` and then `mt done b` both succeed: the legend's `to` edges check the status pair, never the graph.


== TO-BE

- 🟢 A legend status can declare `unblocked = true`; entering it while the task is `blocked` is refused like an illegal transition, naming the unfinished blockers, `--force` to override:

```
$ mt done publish-0-4-0
error: publish-0-4-0: `done` requires an unblocked task; waits on: write-atomic (proposed), link-cycle (wip) (use --force to override)
```

- 🟢 `mt init`'s legend and this repo's set it on `wip` and `done`.
- 🟢 `mt check` reports a task already in such a status while blocked.


== Changes

- `crates/mindtape/src/config_status.rs` (tests first): the `unblocked` key and its JSON Schema.
- `crates/mindtape-cli/src/flip.rs`: refuse, sharing the transition refusal's `--force` path.
- `crates/mindtape/src/validate_links.rs`: the `mt check` report.
- `.mindtape/config.toml`, the `mt init` template: set it on `wip` and `done`.
- `www/content/config.typ`: document the key.


== Alternatives

- A pack gate `when = 'blocked && into("done")'`: hardcodes a legend value, and a gate is a hook, not a transition rule.
- Refuse on every terminal status: `cancelled` and `failed` must stay reachable from blocked work.


== Out of scope

- The parent/children rule: `truth/tasks/derive-child-progress-over-parent-links.typ` reuses this key.


== Dogfood

This repo's legend sets it, so closing a task with an open dependency is refused here first.

