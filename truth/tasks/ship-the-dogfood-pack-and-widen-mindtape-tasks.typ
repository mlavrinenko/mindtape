#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the dogfood pack and widen mindtape:tasks",
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
      enforced-practice: neutral,
      cheap-adoption: helps[`mindtape:tasks` brings the `kind` field its own rules read, and the Dogfood section adopts in one line],
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
    + depends-on("ship-the-tweaks-and-commits-packs.typ"),
)

== Problem

+ Extend `mindtape:tasks` in a project with no `[tasks.fields.kind]`.
+ 🔴 The pack's `environment-for-fix` and its siblings read `kind`, which the pack doesn't declare, so each project copies `[tasks.fields.kind]`, and fifteen copy `[tasks.comments]` as well.
+ 🔴 `one-dogfood-section`, `dogfood-after-out-of-scope`, `dogfood-non-empty`, `dogfood-word-cap` and the `section-allowlist` override that admits Dogfood sit by hand in this repo, and six projects copy them.


== TO-BE

```toml
extends = ["mindtape:tasks", "mindtape:dogfood"]
```

- 🟢 `mindtape:tasks` declares `[tasks.fields.kind]` with `feat`, `fix`, `refactor`, `docs`, `test`, `dev` and `milestone`, `required = "!terminal"` and `example = "feat"`, and `[tasks.comments]` with `policy = "forbid"`.
- 🟢 `mindtape:dogfood` holds the four Dogfood rules and a `section-allowlist` that admits Dogfood. Listed after `mindtape:tasks`, it replaces that pack's allowlist by name.
- 🟢 A test fails when the dogfood allowlist differs from the tasks pack's by anything but `Dogfood`, so the two can't drift.


== Changes

- `crates/mindtape/src/config/packs_tests.rs`: the red tests first: `kind` resolves from the pack alone, the Dogfood allowlist replaces the tasks pack's, and the drift check.
- `crates/mindtape/src/packs/tasks.toml`: the two keys.
- `crates/mindtape/src/packs/dogfood.toml` (new): the pack.
- `crates/mindtape/src/config/packs.rs`: register it.
- `.mindtape/config.toml`: extend `mindtape:dogfood` and drop the Dogfood rules, the allowlist override, `[tasks.fields.kind]` and `[tasks.comments]`.
- `www/content/packs.typ`: both packs.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- The Dogfood rules inside `mindtape:tasks`: Dogfood is this practice's own check on its tool, which a project using `mt` alone has no use for.
- Let one allowlist rule extend another: the allowlist's `allowed` is an array, which replaces whole, and a second mechanism for one pair of packs costs more than a test.


== Out of scope

- The `goals` rule: its check file and goal lists belong to each project.


== Dogfood

This repo's Dogfood rules come from `mindtape:dogfood`, and every open task still passes `mt check`.

