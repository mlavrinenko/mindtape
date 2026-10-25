#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the tweaks and commits packs",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a commit names a started task and a product change carries a changelog entry wherever the packs are adopted],
      cheap-adoption: helps[the tweak and commit rules adopt in one line each instead of a copy per project],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on("ship-the-wip-caps-and-budget-packs.typ"),
)

== Problem

+ Read the `[[tasks.rules]]` and `[[commits.rules]]` of this repo's `.mindtape/config.toml`.
+ 🔴 `tweak-lands-done`, `tweak-stays-one-file`, `refs-task-is-started` and `changelog-with-product-change` sit there by hand. Eight projects copy them, and `changelog-with-product-change` exists in four versions.


== TO-BE

```toml
extends = ["mindtape:tasks", "mindtape:tweaks", "mindtape:commits"]

[dicts.product]
values = ["crates/**", "lib/**", "!**/*_tests.rs"]
```

- 🟢 `mindtape:tweaks` holds `tweak-lands-done` and `tweak-stays-one-file`, and the `[dicts.source]` of `**` that `tweak-stays-one-file` reads, equal to `mindtape:budget`'s, so the two packs agree.
- 🟢 `mindtape:commits` holds `refs-task-is-started` and this repo's `changelog-with-product-change`, which skips `fixup!` subjects and a `Changelog: none` trailer, with a `[dicts.product]` of `**` that a project replaces.
- 🟢 Each pack loads alone over the floor's legend: `extends = ["mindtape:tweaks"]` passes `mt check`.


== Changes

- `crates/mindtape/src/config/packs_tests.rs`: the red tests first: each pack loads alone and beside `mindtape:budget`, and its rules run.
- `crates/mindtape/src/packs/tweaks.toml`, `crates/mindtape/src/packs/commits.toml` (new): the packs.
- `crates/mindtape/src/config/packs.rs`: register both.
- `.mindtape/config.toml`: extend both packs and drop the four hand-written rules.
- `www/content/packs.typ`, `www/content/commit-rules.typ`: the two packs.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Fold both into `mindtape:tasks`: a project that files no tweaks or keeps no changelog would `disable` four rules.
- A `CHANGELOG.md` path parameter: no adopting project keeps its changelog elsewhere, and one that does overrides the rule by name.


== Out of scope

- `work-names-a-task`, `flip-with-work` and `named-tasks-finish-at-the-tip`: they already ship in `mindtape:tasks`.
- The `tweak` tag in a project's `[tasks.tags]`: the tag list is the project's.


== Dogfood

This repo's tweak and commit rules come from the two packs, and its `[dicts.product]` stays local.

