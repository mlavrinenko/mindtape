#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace just fork and just integrate with worktrunk",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the gate and wip check run on every merge from one shared hook, not a script only this repo carries],
      cheap-adoption: helps[a contributor needs only the wt they already use],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 11),
)

== Problem

+ Delegate work: `just fork <slug>`, commit, then `just integrate <branch>`.
+ 🔴 The worktree lifecycle is 393 lines of bash (`.just/scripts/fork.sh`, `.just/scripts/integrate.sh`) plus an e2e test, which the user's worktrunk (`wt`) now covers: sibling worktree, `direnv allow`, the `just check` gate, the no-wip-task check, carrying `.mmz/cache` and the lcov into the primary checkout.
+ Only the dev `mt` build is mindtape-specific. `wt-tasks-done` already runs `just build-mt` before it judges the wip check, but nothing builds it on worktree creation or after a merge lands.

== TO-BE

- 🟢 `wt switch --create <slug>` stands a worktree up and leaves its dev `mt` built; `wt merge` gates, fast-forwards, removes the worktree and rebuilds the target's dev `mt`.
- 🟢 `.config/wt.toml` holds only those two mindtape-only steps.
- 🟢 `fork.sh`, `integrate.sh`, their recipes, scopes, outdatty sources and e2e test are gone; every doc that named them names `wt` instead.

== Changes

- `.config/wt.toml`: a `pre-start` hook running `just build-mt` and a `post-merge` hook doing the same in the target worktree.
- `.just/scripts/fork.sh`, `.just/scripts/integrate.sh`, `crates/mindtape-cli/tests/integrate_refs_e2e.rs`: delete.
- `Justfile`: drop the `fork` and `integrate` recipes; reword the `build-mt` comment.
- `.mmz/config.yaml`, `outdatty.yaml`, `outdatty.lock`: drop the two script scopes and the sources that named them.
- `CONTRIBUTING.md`, `docs/src/contributing.typ`, `docs/src/contributing/commit-messages.typ`, `flake.nix`, `.mindtape/config.toml`: say `wt switch --create` / `wt merge`; regenerate the Markdown docs through their recipe.

== Rollback

Revert the commits; `just fork` and `just integrate` return with their scripts.

== Out of scope

- Done tasks that name `just fork` or `just integrate`: history.
- The user-level hooks (gate, wip check, gate-record carry): they live in the user's config, not this repo.

== Dogfood

This repo delegates through `wt` from now on; the smoke test of this task is the first use.
