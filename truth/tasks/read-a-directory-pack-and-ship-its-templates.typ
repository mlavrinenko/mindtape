#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read a directory pack and ship its templates",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the templates come with the pack instead of a copy per project],
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
      "merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ",
    ),
)

== Problem

+ Share `.mindtape/templates/feat.typ` between two projects that adopt one practice.
+ 🔴 `extends` reads one TOML file, so the template gets copied, and the copies drift: across nine such projects `feat.typ` differs by 45 to 97 lines.


== TO-BE

```text
team/pack.toml           # extends = ["./team"]
team/templates/feat.typ  # mt add -t feat
```

- 🟢 An `extends` path naming a directory reads its `pack.toml`. `"./team"` beside a `team.toml` is refused as ambiguous, naming both. `"./team.toml"` still reads the file.
- 🟢 After every layer and pack is merged, each directory pack's `templates/`, when it exists, is appended to `templates.dirs` as an absolute path, so a project template of the same name wins and `mt add --list` lists both.
- 🟢 Of two directory packs, the later layer's comes first, then the one listed last, as a later rule replaces an earlier one. A pack two layers extend counts once.
- 🟢 The pack directory may sit outside the project, as a fetched pack will.


== Changes

- `crates/mindtape/src/config/packs_tests_dirs.rs` (new): the red tests first: the directory read, the missing `pack.toml`, the ambiguous pair, the appended and ordered template dirs. `crates/mindtape-cli/tests/it/add_e2e.rs` gets the `mt add` one: a pack outside the project, with a project template overriding it.
- `crates/mindtape/src/config/pack_registry.rs`: `PackOrigin::Dir`, holding the directory, and `PACK_FILE`.
- `crates/mindtape/src/config/pack_resolve.rs`: `resolve_dir_pack`, which a fetched pack reuses with its cache directory, and the ambiguous-pair refusal.
- `crates/mindtape/src/config/pack_dirs.rs` (new): append each directory pack's `templates/` to `templates.dirs`. `pack_fold.rs` lists its packs' directories, and `packs.rs` calls the append after the fold.
- `crates/mindtape/src/rules/sites.rs`, `crates/mindtape-cli/src/cmd/config.rs`: name a directory pack's rules by its `pack.toml` and `mt config show --origin` by its directory.
- `crates/mindtape-cli/src/cmd/add/values.rs` needs no change: `root.join(dir)` already takes an absolute dir.
- `www/content/packs.typ`, `www/content/templates.typ`, `www/generate-packs.sh`: the directory form, with two captures.
- `schema/config.schema.json`: regenerate, since the schema copies the `extends` doc.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A `templates` key inside the pack TOML: one more path to keep in step with the files beside it.
- Copy the pack's templates into the project at `extends` time: a copy never receives the next fix.


== Out of scope

- Check and dict files in a pack: `resolve-a-packs-check-and-dict-files-from-its-directory.typ`.
- Templates in a built-in pack: they would need writing to disk first, which the cache of fetched packs brings.


== Dogfood

Not applicable yet: this repo's templates belong to it alone. Fetched directory packs use this path.

