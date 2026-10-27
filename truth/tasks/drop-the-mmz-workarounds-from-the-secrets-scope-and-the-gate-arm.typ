#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the mmz workarounds from the secrets scope and the gate arm",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 6),
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
      sound-codebase: helps[the secrets gate hashes every tracked file outside `truth/tasks/` by construction, so no checker has to guard a hand-kept list, and each `just check` arm hashes its inputs once],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read the `secrets-tree` scope in `.mmz/config.yaml`.
+ 🔴 It lists one glob per top-level directory, because mmz had no negative glob, and `.just/scripts/check-secrets-manifest.sh` runs as a third arm of `just lint::secrets` to catch a directory nobody listed. mmz 0.13.0 ships `exclude:`, so both guard a gap that is gone.
+ Read `.just/scripts/mmz-arm.sh`.
+ 🔴 It runs `mmz --is-fresh -- just <gate>` before each arm only to tell a hit from a run, since `on_hit: ""` prints nothing. That hashes every arm's inputs twice. mmz 0.15.0 writes the answer to the file `MMZ_REPORT` names.


== TO-BE

- 🟢 `secrets-tree` is the whole tree minus the task files, and the manifest arm is gone:

  ```yaml
  secrets-tree:
    globs: ["**"]
    exclude: ["truth/tasks"]
  ```
- 🟢 `mmz-arm.sh` runs mmz once per arm and reads `skipped` or `ran` from `MMZ_REPORT`.


== Changes

- `flake.lock`: bump the `mmz` input to 0.15.0.
- `.mmz/config.yaml`: rewrite `secrets-tree` as above, drop `script-check-secrets-manifest` and its comments, and reword the `on_hit` note.
- `.just/scripts/check-secrets-manifest.sh`: delete.
- `.just/scripts/check-secrets.sh`: drop the call to arm three.
- `.just/scripts/mmz-arm.sh`: read the decision from `MMZ_REPORT` instead of `--is-fresh`.
- `.config/outdatty.yaml`: drop the deleted script from its group.
- `docs/src/contributing/gates.typ`, `docs/contributing/gates.md`: say how the scope excludes task files and how the arm counts hits.


== Rollback

Revert the commit. The old scope list, the manifest arm and the `--is-fresh` pre-check all work on mmz 0.15.0.


== Out of scope

- Other scopes that enumerate directories, such as `examples`. Each lists what its gate checks, not the tree minus a hole.


== Dogfood

Every `just check` in this repo runs both changes: the secrets arm hashes the new scope, and every arm counts its hit through `MMZ_REPORT`.

