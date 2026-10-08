#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replay Typst warnings for files served from the content-hash cache",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 1,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a Typst warning shows on every run, not only the first],
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
  status: proposed(2026, 10, 8),
)

== Problem

+ Closing report-typst-errors-at-their-line-with-typsts-hints
+ 🔴 #raw("crates/mindtape/src/loader/cache.rs"): a Typst warning prints on the run that evaluates a file and never again, because a cache hit skips evaluation and the cache stores no warnings

```
$ mt ls   # first run
warning: tasks/a.typ:1:1: label `<orphan>` is not attached to anything
$ mt ls   # second run, bytes unchanged
(no warning)
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/loader/cache.rs").

== TO-BE

- 🟢 The cache stores a file's warnings beside its marker and replays them into `LoadOutcome::warnings` on a hit.

== Changes

- `crates/mindtape/src/loader/cache.rs`: store the warnings with the entry. `crates/mindtape/src/loader/mod.rs`: replay them on a hit.

== Rollback

- Restore the two functions.

== Out of scope

- Errors, which are never cached.

== Dogfood

A second `mt ls` over a file with an orphan label prints the same warning.
