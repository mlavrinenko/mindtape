#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry a rebased branch's lcov to the primary checkout on land",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[main stays green after a land, with no second instrumented build],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Running `just check` on main after four parallel branches landed, each rebased onto main first
+ 🔴 #raw(".just/scripts/wt-carry-lcov.sh"): after a branch is rebased and landed, main's `just crap` fails on a stale lcov, though `just cover` read fresh in the branch's records

```
just crap: target/coverage/lcov.info predates crates/mindtape/src/filter/eval.rs — run `just cover` first
  (a source the lcov never saw scores as 0% covered, not as unknown)
check: 1 of 29 gates failed: just crap
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/wt-carry-lcov.sh").

== TO-BE

- 🟢 After `just land` of a rebased branch, `just check` on main passes with no `just cover` rebuild.
- 🟢 When the carry skips, it says why on stderr.


== Changes

- `.just/scripts/wt-carry-lcov.sh` and `.just/scripts/_lcov-stale-reason.sh`: judge staleness by content, as mmz does, not by mtime (a rebase gives touched files new mtimes); or report the skip.


== Rollback

- Revert the commit. Main pays a `just cover` after a rebased land, as now.


== Out of scope

- mmz itself.


== Dogfood

The next rebased `just land` leaves main's `just check` green.
