#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Judge the sweep's lcov freshness by content, not mtime",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
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
      sound-codebase: helps[the sweep runs after a checkout without a manual `just cover`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Running `just ask-refactoring` after reverting a probe edit with `git checkout`
+ 🔴 #raw("contrib/ask-refactoring/coverage.sh"): `ensure_lcov_fresh` refuses an lcov older than any `crates/**/*.rs` by mtime, but `just mmz cover` caches by content, so after a checkout or rebase the cache hits, the lcov is not rewritten, and the sweep refuses

```
ask-refactoring: just mmz cover (a cache miss reruns the suite)
ask-refactoring: target/coverage/lcov.info predates crates/mindtape/src/lib.rs — run `just cover` first
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring/coverage.sh").

== TO-BE

- 🟢 A tree whose sources match the content the lcov was measured on sweeps without a manual `just cover`.

== Changes

- `contrib/ask-refactoring/coverage.sh`: judge freshness by the content mmz keyed the lcov on, or touch the lcov on a cache hit.

== Rollback

- Revert the commit.

== Out of scope

- The mutants arm's mtime check, which has no cache to disagree with.

== Dogfood

This repo sweeps right after `just land` without a manual `just cover`.
