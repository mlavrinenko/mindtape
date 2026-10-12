#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the jscpd gate's upstream workaround with canibump",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a workaround that waits on upstream says when it can go, instead of outliving its reason],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
  links: depends-on(
    "gate-on-new-jscpd-clones-renamed-and-near-miss-copies-included.typ",
  )
    + depends-on("track-the-sweeps-repeats-arm-with-canibump.typ"),
)

== Problem

+ Read `check-dry` in the `Justfile`.
+ 🔴 It filters jscpd's JSON report down to the new clones with `jq`, because under `--baseline` every reporter prints all clones, old ones included. Nothing says when upstream fixes that, so the filter stays after its reason is gone.


== TO-BE

- 🟢 `canibump.toml` holds a blocker on the upstream jscpd issue for new-only reports, and `just canibump` says when the filter can go.


== Changes

- `canibump.toml`: the blocker, beside the one the repeats arm waits on.
- `Justfile`: a link to the issue in `check-dry`'s comment, which canibump's scan then finds listed.


== Rollback

- Revert the commit. The filter stays, untracked.


== Out of scope

- Posting the upstream issue, which the maintainer does from the draft in `var/`. File this task's blocker once that issue exists.


== Dogfood

This repo's `just canibump` reports the jscpd blocker.
