#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name packs as source:pack, starting with mindtape:tasks",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "gating"),
  estimate: (src-files: 16),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[one `<source>:<pack>` form names a built-in pack and, later, a team's],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ")
    + depends-on(
      "ship-the-shared-practice-as-official-mindtape-packs.typ",
      "rename-the-rule-packs-page-to-packs.typ",
    ),
)

== Problem

+ Run `mt config show --origin` in this repo.
+ 🔴 The built-in pack reads `pack mindtape-tasks`: a built-in is told apart only by a reserved `mindtape-` prefix, a form with no room for a pack from another source.


== TO-BE

```toml
extends = ["mindtape:tasks", "./team.toml"]
```

- 🟢 An entry with a `/` is a path. Any other entry must be `<source>:<pack>`, and `mindtape` is the one source known without a declaration.
- 🟢 `mindtape-tasks` is refused as unknown, with `mindtape:tasks` as the suggestion. An unknown `<pack>` lists what the source holds. A source other than `mindtape` is refused as an unknown pack source, listing the known sources.
- 🟢 The pack file moves to `crates/mindtape/src/packs/tasks.toml`. The mmz scopes and outdatty sources glob `crates/mindtape/src/packs/*.toml`, and the docs generator copies each pack to `www/generated/packs/<pack>.toml`, so a new pack needs no edit outside its own file, the registry and `TOML_FMT_PATHS`.


== Changes

- `crates/mindtape/src/config/packs_tests_extends.rs`: the red tests first: `mindtape:tasks` resolves, `mindtape-tasks` and `mindtape:nope` fail with their suggestions.
- `crates/mindtape/src/config/pack_resolve.rs`: split an entry into source and pack.
- `crates/mindtape/src/config/pack_registry.rs` (new): move `EMBEDDED_PACKS` and the origin types out of `packs.rs`, keyed by pack name under the `mindtape` source, so `packs.rs` keeps room under its 500-line cap for the leaves after this one.
- `crates/mindtape/src/config/packs.rs`: read the registry from there. `crates/mindtape/src/lib.rs`: re-export the origin types from their new file.
- `crates/mindtape/src/config/mod.rs`: the `extends` field doc, which the schema copies, and the header entry for `pack_registry.rs`.
- `schema/config.schema.json`: regenerate.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: move to `tasks.toml`.
- `.mindtape/config.toml`, `examples/api-service/.mindtape/config.toml`: `extends = ["mindtape:tasks"]`.
- `.mmz/config.yaml`, `.config/outdatty.yaml`: glob the pack files. `Justfile` (`TOML_FMT_PATHS`): name `tasks.toml`, since `meta::gate-drift` compares that list with the TOML files git knows and refuses a glob.
- `www/generate-packs.sh`, `www/utils/recipes.typ`: copy and read every pack from `www/generated/packs/`.
- `.config/outdatty.yaml`, `.githooks/commit-msg`, `.just/scripts/check-doc-prose.sh`, `www/utils/demo.typ`, `crates/mindtape-cli/src/cmd/config.rs`: the new name and path.
- `www/content/*.typ`, `docs/src/**/*.typ`, `skills/mindtape/references/filters.md`: the new name.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Keep `mindtape-tasks` as an alias: the project ships no compatibility before its first release.
- `@mindtape/tasks`: the `/` already marks a path.


== Out of scope

- Declared sources such as `acme:`: `read-pack-sources-and-packs-lock-into-the-config.typ`.
- Built-in dictionary names such as `mindtape-verbs`.


== Dogfood

This repo's `.mindtape/config.toml` reads `extends = ["mindtape:tasks"]`.

