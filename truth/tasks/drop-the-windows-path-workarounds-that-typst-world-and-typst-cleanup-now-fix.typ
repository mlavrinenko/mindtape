#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the Windows path workarounds that typst-world and typst-cleanup now fix",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 10),
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
      sound-codebase: helps[Mindtape stops carrying code that compensates for its own dependencies],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ typst-world 0.5.6 builds a project file id with `/` on every platform, and adds `project_file_id`. typst-cleanup 0.2.1 mints a tombstone's `path=` with `/`.
+ 🔴 Mindtape still carries the workarounds for both: `loader::disk_world`, the `slash_path` id in `validate/candidate.rs`, and the `\` rewrite in `TaskLinks::tombstone`.


== TO-BE

- 🟢 Mindtape builds its worlds with `HarvestWorld::with_root`, `HarvestWorld::new` and `project_file_id`, and renders a tombstone as minted.


== Changes

- `crates/mindtape/Cargo.toml`, `crates/mindtape-cli/Cargo.toml`, `Cargo.lock`: typst-world 0.5.6 and typst-cleanup 0.2.1.
- `crates/mindtape/src/loader/world.rs`, `crates/mindtape/src/loader/world_tests.rs`: delete.
- `crates/mindtape/src/loader/mod.rs`, `crates/mindtape/src/lib.rs`, `crates/mindtape/src/loader/tests_prelude.rs`, `crates/mindtape-cli/src/cmd/eval.rs`: build with the upstream constructors.
- `crates/mindtape/src/validate/candidate.rs`: `project_file_id`. A candidate outside the root now says so, not "bad virtual path".
- `crates/mindtape-cli/src/cmd/rm/mod.rs`, `crates/mindtape-cli/src/cmd/rm/tests.rs`: drop the tombstone rewrite and its test.


== Rollback

Revert the commit. Both crates still build against the old code.


== Out of scope

- `mindtape::slash_path`, which `Task::file_key`, link targets and the `mt rm` git probe use for their own `/` paths.
- The Windows path separator policy and the preset gates, which wait on the owner.


== Dogfood

Not applicable. This repo is developed on Linux.

