#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Adopt mindtape:ice in the ice-strict init answer",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (src-files: 2),
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
      sound-codebase: helps[the strict ICE answer reads the one copy in `mindtape:ice`, not a second one in `axes.rs`],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
  links: related(
    "share-config-between-projects-through-packs.typ",
  )[its Out of scope names this follow-up],
)

== Problem

+ Run `mt init --no-input --priority ice-strict` in an empty directory.
+ 🔴 The written `.mindtape/config.toml` sets `framework`, `allow-manual` and `required` under `[tasks.priority]` by hand, copied into `crates/mindtape-cli/src/cmd/init/wizard/axes.rs`, with no `extends` line, though `mindtape:ice` ships the same practice.


== TO-BE

- 🟢 The `ice-strict` answer writes `extends = ["mindtape:ice"]` instead of the copied keys, so the project takes the pack's `required = "!terminal"` and `example = 3` and gets its fixes on upgrade.
- 🟢 The comment above that answer names `mindtape:ice` with a link to `/packs/`.
- 🟢 An axis fragment may carry a top-level `extends`, and `mt init` folds every fragment's entries into one array at the head of the config, each entry once.


== Changes

- `crates/mindtape-cli/src/cmd/init/compose.rs`: lift each fragment's `extends` out, fold the entries in fragment order without duplicates, write the one array before the first table, and leave `extends` out of the array-conflict check. Red tests in `compose_tests.rs` first.
- `crates/mindtape-cli/src/cmd/init/wizard/axes.rs`: the `ice-strict` fragment becomes the `extends` line and its comment.
- `www/content/config.typ`: the presets section says an answer can adopt a pack.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Move the `gated` and `flow` presets onto `mindtape:tasks` and `mindtape:flow`: their legends and spec sections differ from the packs by design, with the reasons in their comments, so adopting the packs would change what `mt init` gives their users.
- Leave the strict answer as a copy: it already differs from the pack on `required`.


== Out of scope

- The presets: none matches a shipped pack today. One adopts a pack through the same fold once a pack holds its practice.
- New packs.


== Dogfood

Not applicable: this repo's config already extends `mindtape:ice` by hand, and only a new project runs `mt init`.
