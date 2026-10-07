#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry the worktree's coverage into main on integrate",
  kind: "dev",
  priority: ice(
    impact: 5,
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
      sound-codebase: helps[`just check` on main passes right after an integrate, without a 7-minute coverage rebuild],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Integrating the area-move branches, then re-running the gate on main
+ 🔴 #raw(".just/scripts/integrate.sh"): Integrate copies the worktree's \`.mmz/cache\` gate records but not \`target/coverage/lcov.info\`, so \`just crap\` on main refuses the stale lcov after every integrate that touched source.

After `just integrate filter` (gate fresh, fast-forwarded): `just check` → `check: 1 of 28 gates failed: just crap`; `just crap` → `target/coverage/lcov.info predates crates/mindtape/src/commit_task_expr.rs — run just cover first`. The same after `just integrate lib-areas`.

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/integrate.sh").

== TO-BE

- 🟢 `just integrate <branch>` copies the worktree's `target/coverage/lcov.info` beside the gate records, and `just check` on main passes with no `just cover`.

== Changes

- `crates/mindtape-cli/tests/integrate_e2e.rs` (or the integrate test that exists): red first, an integrate leaving `just crap` fresh.
- `.just/scripts/integrate.sh`: copy the lcov after the fast-forward, only when the worktree's is newer.

== Rollback

- Revert the commit; main falls back to `just cover` after an integrate.

== Out of scope

- Any other `target/` artifact: only crap reads one across checkouts.

== Dogfood

The next CLI area-move integrate is checked on main without a rebuild.
