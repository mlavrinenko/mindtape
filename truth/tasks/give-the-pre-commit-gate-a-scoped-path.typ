#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "give the pre commit gate a scoped path",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "gating"),
  estimate: (files: 4),
  status: proposed(2026, 8, 7),
)

== Summary

`.githooks/pre-commit` opens with `mmz --is-fresh --tag gate`, which asserts
every sub-gate of `just check` holds a fresh passing record. That is right for a
commit landing on the main checkout. It makes a delegated worktree commit
impossible: a fresh worktree has no records at all, so an agent must run the
full aggregate — a cold Rust workspace plus a Typst docs-site build and a
Pagefind index — before it may commit one line.

Observed 2026-08-07: three delegated agents stalled four times between them,
because that aggregate exceeds the harness's foreground window and gets
auto-backgrounded. All three had complete, correct, gate-clean work sitting
uncommitted. The only lever available was `git commit --no-verify`, which
switches off all three of the hook's assertions — the freshness check, the task
formatting check, and `mt check` — not just the expensive one. A gate whose
only escape is a sledgehammer trains people to reach for the sledgehammer.

== Scope

- Give the freshness assertion a scoped path. Two shapes are worth weighing;
  pick one and record why in the commit:
  - An explicit opt-out variable (e.g. `MINDTAPE_GATE_SCOPE=none`) that skips
    only the `mmz --is-fresh` line, leaving `typstyle --check truth/tasks` and
    `mt check` running. Cheap, obvious, and honest about being an override.
  - Scope the assertion to the sub-gates whose inputs the staged diff actually
    touches — a docs-only commit need not assert `just test`. Truer, but it
    needs a mapping from paths to mmz rules, and `.mmz/config.yaml` scopes are
    the natural place for it.
- Whichever wins, the other two assertions must keep running. The bug is that
  one expensive check is fused to two cheap ones.
- An override must be visible in the commit, not silent — the reader of
  `git log` should be able to tell a scoped commit from a fully gated one.
- Document it in `CONTRIBUTING.md` beside the existing quality-gate table, and
  say plainly when reaching for it is legitimate (a delegated worktree whose
  aggregate runs at integration) and when it is not (skipping a red gate).

== Notes

The fix is what makes delegated work committable here at all. Until it lands,
every agent-produced branch commit in this repo is a `--no-verify` commit.
