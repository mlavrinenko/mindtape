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
  status: proposed(2026, 10, 26),
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

- 🟢 An `extends` path naming a directory reads its `pack.toml`. A `team.toml` beside a `team/` directory is refused as ambiguous, naming both.
- 🟢 After every layer and pack is merged, each directory pack's `templates/` is appended to the template dirs, so a project template with the same name wins and `mt add --list` lists both.
- 🟢 The pack directory may sit outside the project, as a fetched pack will.


== Changes

- `crates/mindtape/src/config/packs_tests_body.rs`: the red tests first: a template from a directory pack, a project template overriding it, the ambiguous pair.
- `crates/mindtape/src/config/pack_resolve.rs`: read `<dir>/pack.toml`, refuse the ambiguous pair, and record the pack directory.
- `crates/mindtape/src/config/packs.rs`: append each pack's `templates/` to `templates.dirs` after the fold.
- `crates/mindtape-cli/src/cmd/add/values.rs`: read template dirs that may be absolute.
- `www/content/packs.typ`, `www/content/templates.typ`: the directory form.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A `templates` key inside the pack TOML: one more path to keep in step with the files beside it.
- Copy the pack's templates into the project at `extends` time: a copy never receives the next fix.


== Out of scope

- Check and dict files in a pack: `resolve-a-packs-check-and-dict-files-from-its-directory.typ`.
- Templates in a built-in pack: they would need writing to disk first, which the cache of fetched packs brings.


== Dogfood

Not applicable yet: this repo's templates belong to it alone. Fetched directory packs use this path.

