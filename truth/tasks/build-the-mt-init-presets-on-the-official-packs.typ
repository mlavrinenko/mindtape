#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Build the mt init presets on the official packs",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (src-files: 8),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new project starts on the shared packs and gets their fixes on upgrade, not a frozen copy],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one copy of each practice, in the packs, not a second one in the presets],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: related(
    "share-config-between-projects-through-packs.typ",
  )[its Out of scope names this follow-up],
)

== Problem

+ Run `mt init --preset gated --no-input` in an empty directory.
+ 🔴 The written `.mindtape/config.toml` restates a status legend, the body header rules and two `[[on.flip]]` gates by hand, about 245 lines copied from `crates/mindtape-cli/src/cmd/init/presets/gated/config.toml`, with no `extends` line.
+ 🔴 `axes/workflow_flow.toml` declares five statuses while `mindtape:flow` declares six, so the two copies of one legend already drift.


== TO-BE

- 🟢 Each preset and wizard axis that matches an official pack writes an `extends` entry for it, such as `extends = ["mindtape:tasks", "mindtape:flow"]`, plus only the keys the pack leaves to the project.
- 🟢 `--annotate` keeps its comments for the keys the config still writes, and names each adopted pack with a link to `/packs/`.
- 🟢 A preset with nothing in common with any pack, such as `todo-done`, stays as it is.


== Changes

- `crates/mindtape-cli/src/cmd/init/preset_tests.rs`, `compose_tests.rs`: the red tests first: each preset's config loads to the same `mt config show --json` as today, minus the differences this task decides.
- `crates/mindtape-cli/src/cmd/init/presets/*/config.toml`: replace the copied keys with `extends`.
- `crates/mindtape-cli/src/cmd/init/axes/*.toml`, `compose.rs`: let an axis contribute an `extends` entry and fold the entries into one array.
- `www/content/config.typ`: the presets section shows the `extends` lines.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Ship the presets as packs themselves: a preset is a starting file the user edits, and a pack is shared policy. Two names for one config would blur both.
- Leave the presets as copies: they drift from the packs, as the five-versus-six legend shows.


== Out of scope

- Moving other projects' configs onto the packs: each project files its own task.
- New packs: this task adopts the eight that ship.


== Dogfood

This repo already extends the official packs, so `mt init --preset gated` in a scratch directory should print a config close to this repo's `.mindtape/config.toml`.

