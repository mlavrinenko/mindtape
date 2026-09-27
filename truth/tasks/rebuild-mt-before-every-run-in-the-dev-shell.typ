#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rebuild mt before every run in the dev shell",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 1),
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
      sound-codebase: helps[mt never answers from a binary older than the tree beside it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 27),
)

== Problem

+ `just fork x`, then `mt flip wip <task>` in the new worktree.
+ 🔴 It fails loading `.mindtape/config.toml`: `target/debug` is empty, so the stale `~/.cargo/bin/mt` answers.
+ `just integrate <branch>` onto `main`, then `mt ls`.
+ 🔴 `unknown field unblocked`: the fast-forward rebuilt nothing, so `target/debug/mt` predates the config it reads.


== Environment

- NixOS, the flake dev shell via direnv; `~/.cargo/bin/mt` installed from an older release; commit `fa314b1`.


== TO-BE

- 🟢 The dev shell's `mt` runs `cargo build -q` and then the fresh binary; a no-op build costs ~0.2 s.
- 🟢 It builds into `target/mt-dev`, off the lock `just check` holds on `target/` and off the rrule arm's feature build of `target/debug/mt`.


== Changes

- `flake.nix`: an `mt-dev` wrapper in place of `target/debug` on `PATH`.
- `docs/src/contributing.typ`: one line on it.


== Out of scope

- Copying a built binary into a fork: cargo cannot trust a copy, and the source checkout's binary is just as stale.


== Dogfood

Every delegated agent's first `mt` call in a fresh worktree.

