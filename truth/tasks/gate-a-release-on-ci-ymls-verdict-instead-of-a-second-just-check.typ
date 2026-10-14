#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Gate a release on ci.yml's verdict instead of a second just check",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 4),
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
      sound-codebase: helps[a tag on a commit CI never passed cannot publish],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 14),
)

== Problem

1. Read `.github/workflows/release.yml`'s `check` job.
2. 🔴 It runs `just check` on the tagged commit, which `ci.yml` already passed: `just release` tags only once that run is green. So every release spends a second full run on a verdict GitHub already holds.


== TO-BE

- 🟢 A `ci-passed` job asks GitHub for a successful `ci.yml` run on the tagged commit, and the build waits on it.
- 🟢 A tag on a commit with no green `ci.yml` run fails there, naming the commit and how to re-run.


== Changes

- `.github/workflows/release.yml`: `ci-passed` in place of `check`.
- `crates/mindtape-cli/tests/it/release_workflow_e2e.rs`: the release notes tests, renamed from `release_notes_e2e.rs`, plus the gate's step against a stub `gh`.
- `crates/mindtape-cli/tests/it/main.rs` and `.mmz/config.yaml`: the new module name.


== Rollback

- Revert the commit.


== Out of scope

- `ci.yml`, which still runs `just check` on every push and PR.


== Dogfood

The 0.3.0 release passes through it.
