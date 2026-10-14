#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Run CI's coverage stage beside the rest",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (src-files: 5),
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
      sound-codebase: helps[a push gets its verdict in about 5.5 minutes, not 9],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 14),
)

== Problem

1. Time `just check`'s stages cold in a fresh clone on 4 cores, with no kache and no mmz cache.
2. 🔴 fast takes 116 s, build 215 s and cover 212 s, one after another in CI's one job, about 9 minutes. cover shares no build with the other two, yet waits for them.


== TO-BE

- 🟢 `just check fast build` runs only those stages, in order; a name outside the three fails at dispatch.
- 🟢 CI runs `fast build` and `cover` plus the flake build on two runners at once.

```yaml
matrix:
  include: [{part: "lint, test", stages: fast build}, {part: "coverage, nix build", stages: cover}]
```


== Changes

- `Justfile`: `check *STAGES` with a pattern.
- `.just/scripts/check.sh`: run the named stages, each by its literal name, and refuse a `_check-stages` list it doesn't run.
- `.just/scripts/arg-pattern-check.sh`: pin the pattern.
- `.github/workflows/ci.yml`: the `check` job as a two-part matrix.
- `docs/src/contributing/gates.typ`: say so.


== Rollback

- Revert the commit.


== Out of scope

- Splitting the fast or build stage further: their gates share one build, so more jobs means more cold compiles.


== Dogfood

Every push here after 0.3.0 runs the split.
