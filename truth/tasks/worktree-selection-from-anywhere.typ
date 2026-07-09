#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Worktree selection from anywhere",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 7,
  ),
  tags: ("cli",),
  status: done(2026, 7, 9),
)

== Summary

Let `mt` run from anywhere on the system and operate on a chosen worktree, rather
than only the project discovered by walking up from the current directory. Today
`Project::open` (`crates/mindtape-cli/src/util.rs`) resolves the root purely from
`current_dir` via `find_root`; there is no way to point `mt` at a different
worktree without `cd`-ing into it.

Orthogonal to named views and to view auto-activation — this decides which
project `mt` reads, not which query or default view applies within it. Split out
from the `named-views-and-contexts` task so the "context" concept stays about
views, not about project selection.

== Scope

- A worktree override that short-circuits the cwd walk-up in `Project::open`:
  - Global flag `-w, --worktree <path>` on `mt`.
  - Env var `MINDTAPE_WORKTREE=<path>` as the fallback (flag wins).
  - When set, resolve the root from the given path (still requiring a
    `.mindtape/` there) instead of `find_root(cwd)`.
- Decide relative-path semantics: resolve `-w` against cwd, absolute as-is;
  `MINDTAPE_WORKTREE` likewise.
- Keep every command routed through the single discovery entry point so `ls`,
  `add`, `check`, `flip`, `go`, `rm` all honour the override uniformly.
- Tests: flag over env precedence; override targets the named worktree; a path
  with no `.mindtape/` errors clearly; relative-path resolution.
