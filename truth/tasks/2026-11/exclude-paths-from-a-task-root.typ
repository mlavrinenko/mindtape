#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Exclude paths from a task root",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 3,
  ),
  difficulty: 5,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the task specifies the scope, the first-admitting-root rule and every call site.],
      enforced-practice: helps[`mt add` refuses a destination the root excludes, before it writes.],
      cheap-adoption: helps[a root of `.` narrows its own scope with one key, and a narrower root still loads.],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the config schema and the concepts page name the key and its glob base.],
    ),
    process: (
      sound-codebase: helps[one policy covers every layer, and config load rejects an invalid glob.],
      docs-match-code: helps[the concepts page and the config schema carry the rule.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
)

== AS-IS

+ Configure a project with `[[tasks.roots]] path = "."` and a `docs/` folder holding `.typ` examples with `task.with` calls.
+ Run `mt ls`.
+ 🔴 The examples load as tasks, because every `.typ` under the root is a source.
+ Read `[[tasks.roots]]` in `www/content/config.typ`.
+ 🔴 A root has `path`, `name` and `derive-tags`, and no way to narrow the paths it covers.

== TO-BE

- 🟢 A root takes `exclude`, a list of globs relative to the root. A path matching one is not a task source under that root.
- 🟢 Another root can still admit the path: an exclude narrows one root, it does not deny the project.
- 🟢 `mt add` refuses a destination the root excludes, and `mt check` and the commit states classify it the same way.
- 🟢 A root's derived tags and depth come from the first root that admits the file.

```toml
[[tasks.roots]]
path = "."
name = "home"
derive-tags = true
exclude = ["docs/**", "examples/**"]
```

== Changes

- `crates/mindtape/src/config/roots.rs`: `RootConfig.exclude`, `RootSpec::excludes` and `RootSpec::admits`, and a check that rejects an invalid glob.
- `crates/mindtape/src/task_source.rs`: the compiled excludes, `admits`, and `would_be_source`.
- `crates/mindtape/src/loader/mod.rs`: `derivation` reads the first root that admits the file.
- `crates/mindtape/src/loader/visible.rs`: `is_candidate` reads the root's scope, and `NotVisible::Excluded`.
- `crates/mindtape/src/config/roots_tests.rs`: config load rejects an invalid glob.
- `crates/mindtape/src/task_source_tests.rs`: an excluded path is not a source, and a second root still admits it.
- `crates/mindtape/src/loader/tests_visible.rs`: a root of `.` excludes `docs`, and a `docs/tasks` root loads it.
- `crates/mindtape/src/loader/derive_tests.rs`: the `RootConfig` literals carry the new field.
- `www/content/concepts.typ`: the exclude rule under "Which files count as tasks".
- `www/content/config.typ`: the `exclude` key on a root.
- `schema/config.schema.json`: the new key.
- `schema/output/config-show/1.schema.json`: the config-show schema carries the new key.
- `CHANGELOG.md`: the feature under `[Unreleased]`.

== Alternatives

- A project-wide `[tasks] exclude`: a deny no root can override, so a narrower root under an excluded folder could never load. It also makes the glob base project-relative, a second meaning beside `derive-tags`.
- Both a global and a per-root list: a replace loses the global excludes, an add prevents re-inclusion, and the merge has no obvious answer.
- A bare directory name that excludes its subtree: `globset` matches the path itself, so `docs` would not match `docs/foo.typ`. `docs/**` is the same rule `derive-tags` uses.

== Out of scope

- A project-wide exclude. A root's exclude composes with overlapping roots. A global deny would not.
- `mt fix --all`. It is project-wide and keeps its own scope.
- `mt explain`'s roots line. It does not show `derive-tags` either.

== Dogfood

Not applicable: this repo's only root is `truth/tasks`, which holds tasks and nothing else. The feature serves a project whose root is `.`.
