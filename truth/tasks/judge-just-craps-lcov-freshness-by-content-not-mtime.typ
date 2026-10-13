#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Judge just crap's lcov freshness by content, not mtime",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
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
      sound-codebase: helps[just check passes after a rebase without a manual just cover],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ fixing the same mtime rule in the sweep (j5g)
+ 🔴 #raw(".just/scripts/_lcov-stale-reason.sh"): just crap refuses after a checkout because the cover gate hits by content and leaves the lcov older than touched sources

just crap: target/coverage/lcov.info predates crates/mindtape/src/lib.rs — run `just cover` first

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/_lcov-stale-reason.sh").

== TO-BE

- 🟢 just crap passes on a tree whose sources match the content mmz keyed the lcov on.

== Changes

- `.just/scripts/_lcov-stale-reason.sh`: ask `mmz --is-fresh -- just cover`, as contrib/ask-refactoring/undertested.sh does.

== Rollback

- Revert the commit.

== Out of scope

- The mutants arm.

== Dogfood

This repo runs just check right after a rebase.
