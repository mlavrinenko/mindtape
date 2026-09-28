#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Migrate this repo's Refs trailers to stems",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the Refs rule is checked on every commit here],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[CONTRIBUTING states the rule the hook enforces],
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 28),
  links: depends-on("check-commit-messages-against-commit-rules.typ"),
)

== Problem

+ Read `CONTRIBUTING.md`, Commit Messages.
+ 🔴 It asks for `Refs: truth/tasks/<stem>.typ`, a path that breaks when a task moves, and nothing checks it.


== TO-BE

- 🟢 `CONTRIBUTING.md` asks for `Refs: <stem>`; a commit touching only task files needs none.
- 🟢 `.githooks/commit-msg` runs `mt check --commit-msg "$1"` beside its `Gate-Scope` check.
- 🟢 The `mindtape-tasks` pack ships the `work-names-a-task` rule, so this repo and every project extending the pack get it.


== Changes

- `docs/src/contributing.typ`, regenerated `CONTRIBUTING.md`: the stem form.
- `.githooks/commit-msg`: the check.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: the `work-names-a-task` rule.
- `.github/workflows/ci.yml`: move the commit-rules step's `FLOOR` to this change's commit, so the path trailers before it stay exempt.


== Rollback

Revert the commit; the hook line and the pack rule go, and the path form returns to CONTRIBUTING.


== Out of scope

- Rewriting past trailers: history stays; the resolver reads a path's stem.


== Dogfood

Every commit here passes through it.

