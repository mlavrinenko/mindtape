#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Exempt a milestone's close from the flip-with-work rule",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "found"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a milestone closes through its leaves under the shipped pack, as a decision does],
      cheap-adoption: helps[an adopter with milestones drops the local override of flip-with-work],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 23),
)

== Problem

+ Under `extends = ["mindtape-tasks"]`, with a task of `kind: "milestone"` in `wip` and all its leaves done, stage `mt done <milestone>` alone and commit with a `Refs: <milestone>` footer.
+ 🔴 The `commit-msg` hook prints `commits.rules.flip-with-work: <milestone>: start or close a task in a commit that carries its work`. The rule exempts a `decision` and a close that retires other tasks, but not a milestone, which ends through its leaves and carries no work of its own.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.
- Hit in corpuspeak closing `ship-the-mvp`, which now overrides the rule in its own `.mindtape/config.toml`.


== TO-BE

- 🟢 A milestone closes in a task-only commit under the shipped pack, as a decision does.
- 🟢 A leaf or a task of no `kind` is still refused for a bare flip.


== Changes

- `crates/mindtape-cli/tests/it/commit_milestone_e2e.rs`: the red test first, a milestone's close passing and a leaf's bare close refused.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: `flip-with-work` adds `|| staged(kind == "milestone")`, with its comment saying why. The pack already reads `kind` in its section rules.
- `www/content/commit-rules.typ` and `docs/src/contributing/commit-messages.typ`: the rule's description names the milestone exemption.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Out of scope

- The WIP-cap rules, which stay this repo's policy and not the pack's.
- corpuspeak's local override, which its owner deletes after the release.


== Dogfood

This repo has no open milestone to close, so the e2e test closes one in a scratch repo under the shipped pack.
