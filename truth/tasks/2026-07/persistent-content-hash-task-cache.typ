#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "persistent content-hash task cache",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  tags: ("mt", "cli"),
  status: done(2026, 7, 17),
)

== Summary

Cache each file's harvested Task (canonical JSON) under a git-ignored .mindtape/cache/. The key is the file's content hash plus a build key (config hash + lib/ hash + mt version). Warm mt ls / mt check then stat the roots and read the cache instead of re-evaluating unchanged task files through Typst. Only changed files re-eval. This is the warm-path win: the common case runs repeated commands over a static backlog.

== Scope

- New git-ignored `.mindtape/cache/` artifact (one entry per task, keyed by content hash).
- Cache key = task file content hash + build key (serialized config hash, `lib/`/prelude hash, `mt` version). Any build-key change busts the whole cache.
- Loader: stat each root file, hash it, hit → deserialize cached Task. Miss → harvest + write entry. Body still read from disk (cheap), never cached.
- Correctness first: a stale or partial entry must never surface a wrong Task. On any key/format mismatch, fall back to eval.
- Stacks with batch harvest (cold path): this attacks repeated invocations, while batch attacks a single cold run.
- Tests: unchanged file skips eval. Edited file re-evals. A config, lib or version bump busts the cache. A corrupt entry falls back.
