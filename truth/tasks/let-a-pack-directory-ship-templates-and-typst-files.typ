#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let a pack directory ship templates and Typst files",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("mt", "gating"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the templates and the `goals` check file come with the pack instead of a copy per project],
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

+ Share `.mindtape/templates/feat.typ` and the Typst file a `check = { typst = "truth/lib.typ", … }` rule reads between two projects.
+ 🔴 `extends` reads one TOML file, so both get copied, and the copies drift: across nine projects with one practice, `feat.typ` differs by 45 to 97 lines.


== TO-BE

```text
team/pack.toml           # extends = ["./team"]
team/templates/feat.typ  # mt add -t feat
team/lib.typ             # check = { typst = "lib.typ", function = "check-goals" }
```

- 🟢 An `extends` path naming a directory reads its `pack.toml`.
- 🟢 The pack's `templates/` joins the template dirs after the extending layer's own, so a project template with the same name wins, and `mt add --list` lists both.
- 🟢 A `typst =` path or a `[dicts]` `path` in the pack resolves from the pack directory. A check file is evaluated in a Typst world rooted at that directory, so it may sit outside the project, as a fetched pack will.


== Changes

- `crates/mindtape/src/config/packs_tests_body.rs`: the red tests first: a template, a check file and a dict file from a directory pack, and a project template overriding the pack's.
- `crates/mindtape/src/config/pack_resolve.rs`: read `<dir>/pack.toml` and record the pack directory.
- `crates/mindtape/src/config/packs.rs`: rebase the pack's relative paths onto its directory, and append its `templates/` to `templates.dirs`.
- `crates/mindtape/src/rules/typst.rs`: evaluate a pack's check file in a world rooted at the pack directory.
- `crates/mindtape-cli/src/cmd/add/values.rs`: read template dirs that may be absolute.
- `www/content/packs.typ`, `www/content/templates.typ`: the directory form.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- A `templates` key inside the pack TOML: one more path to keep in step with the files beside it.
- Copy the pack's templates into the project at `extends` time: a copy never receives the next fix.


== Out of scope

- Templates in a built-in pack: they would need writing to disk first, which the cache of fetched packs brings.
- A Typst file the task files import: a task imports it by a path inside the project.


== Dogfood

Not applicable yet: this repo's templates and `truth/lib.typ` belong to it alone. Fetched directory packs use this path.

