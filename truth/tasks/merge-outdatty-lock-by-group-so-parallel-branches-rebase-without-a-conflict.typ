#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge outdatty.lock by group so parallel branches rebase without a conflict",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.7,
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
      sound-codebase: helps[a rebase keeps both branches' recorded reviews instead of a hand-picked side that drops one],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 6),
)

== Problem

+ Fork two worktrees with `just fork`, and in each edit a source in a different outdatty group and run `just outdatty-update <group>`.
+ Integrate the first, then `git rebase main` in the second.
+ 🔴 `CONFLICT (content): Merge conflict in outdatty.lock`. The two branches changed different groups, but their hash lines sit close together in one YAML file. On 2026-10-06 this hit three of the five rebases.
+ 🔴 The quick fix, `git checkout --ours outdatty.lock`, drops the second branch's recorded reviews. Someone then has to rerun `outdatty update` for each group and claim reviews they did not redo.


== TO-BE

- 🟢 A rebase or merge that touches `outdatty.lock` merges it key by key (group, then `source` or `dependents`, then path): a key only one side changed takes that side.
- 🟢 A key both sides changed to different hashes takes the side whose hash matches the file in the merged tree. If neither matches, the merge stops with a conflict naming that group and path.
- 🟢 Every clone and every `just fork` worktree gets the driver with no manual step.


== Changes

- `.just/scripts/outdatty-lock-merge.sh`, new: the merge driver, given the base, ours and theirs files, writing the merged YAML (`yq` is in the dev shell). Red test first: a fixture of three lock versions that conflict as text but merge by key.
- `.gitattributes`: `outdatty.lock merge=outdatty-lock`.
- `flake.nix` shell hook, or `.just/scripts/fork.sh`: register `merge.outdatty-lock.driver` in the repo's git config.
- `docs/src/contributing.typ`: one sentence under Dependency Drift.


== Rollback

- Delete the `.gitattributes` line. Git falls back to its text merge.


== Out of scope

- Changing outdatty itself or its lock format. Its source lives outside this repo.
- Other generated files.


== Dogfood

The next two parallel branches in this repo rebase without touching `outdatty.lock` by hand.

