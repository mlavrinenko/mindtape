#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Lint the GitHub workflows in just check",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("gating", "tooling"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a workflow error fails locally, before a push],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 14),
)

== Problem

1. Edit `.github/workflows/ci.yml` so a job `needs` a job that doesn't exist.
2. 🔴 Run `just check`. It passes: no gate reads the workflows, so the error shows only on GitHub, after a push.
3. 🔴 `tests/it/release_workflow_e2e.rs` runs a step of `release.yml`, but `just test` and `just cover` don't hash `.github/workflows/`, so a `release.yml` edit leaves both falsely fresh.


== TO-BE

- 🟢 `just check-workflows` runs actionlint, with shellcheck on each `run:` block, and `just check` runs it.
- 🟢 A workflow edit re-runs `just test` and `just cover`.

```yaml
scopes: {workflows: [".github/workflows/**"]}
rules: [{name: just check-workflows, tags: [gate], inputs: [workflows, gates-meta]}]
```


== Changes

- `Justfile`: a `check-workflows` gate in `_check-fast`.
- `.mmz/config.yaml`: a `workflows` scope, read by the new gate, `just test` and `just cover`.
- `flake.nix`: actionlint and shellcheck in the dev shell.
- `.github/workflows/release.yml`: the one shellcheck finding, `tr 'A-Z' 'a-z'`.


== Rollback

- Revert the commit.


== Out of scope

- Running workflows locally with `act`. It needs Docker and can't reproduce the runners' caches.


== Dogfood

Every `just check` here lints the workflows.
