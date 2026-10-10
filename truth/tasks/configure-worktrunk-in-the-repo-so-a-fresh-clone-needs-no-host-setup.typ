#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Configure worktrunk in the repo so a fresh clone needs no host setup",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the gate and the wip check before a merge travel with the repo, not with one machine],
      cheap-adoption: helps[a fresh clone with nix and direnv forks and lands work the same way the maintainer does],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 11),
)

== Problem

+ On a machine without the maintainer's ~/.nix, clone mindtape and run `wt switch --create x`, commit, then `wt merge`.
+ 🔴 worktrunk's defaults squash the branch into one commit, losing each commit's `Refs:` footer, and commit a dirty tree. No gate runs, because the gate, the wip check and the gate-record carry are user hooks in ~/.nix (`apps/worktrunk.nix`, helpers `wt-tasks-done`, `wt-carry-gate`).
+ 🔴 `wt` isn't in this repo's dev shell either.


== TO-BE

- 🟢 `worktrunk` comes from `flake.nix`'s dev shell.
- 🟢 `.config/wt.toml` holds every hook: `pre-start` (direnv allow and load, `just build-mt`), `[[pre-merge]]` gate (`just check`) then the wip check, `pre-remove` gate-record and lcov carry, `post-merge` `just build-mt`.
- 🟢 Project config can't set merge defaults, so `just fork <slug>` runs `wt switch --create <slug> --yes` and `just land` runs `wt merge --no-squash --no-commit --yes`.
- 🟢 With nix and direnv alone, a clone delegates and lands like the maintainer.


== Changes

- `flake.nix`: `worktrunk` in the dev shell.
- `.config/wt.toml`: the hooks above. Today it holds only `just build-mt`.
- `.just/scripts/wt-tasks-done.sh`, `.just/scripts/wt-carry-gate.sh`: ported from the two `writeShellApplication`s in ~/.nix/apps/worktrunk.nix.
- `Justfile`: `fork` and `land`.
- AGENTS.md and the contributing docs: delegate with `just fork`, land with `just land`, regenerated through the docs recipes.


== Rollback

Revert the commit; the ~/.nix user hooks still cover the maintainer's machine until ~/.nix drops them.


== Out of scope

- Removing the ~/.nix user hooks: ~/.nix's own task, after this lands, so no hook runs twice.
- canibump and cratemplate, which follow this repo's result.
- Shipping the wip check as an `mt` subcommand and the carry in mmz, which would drop the per-repo scripts. Worth a follow-up once a second repo carries them.


== Dogfood

The next delegated task here forks with `just fork` and lands with `just land`; a scratch clone under a clean HOME proves the no-host-setup claim.

