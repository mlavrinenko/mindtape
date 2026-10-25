#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Load one task file without loading the project",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "speconaut"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a tool reading one task per git hook call pays for one file, not the backlog],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 25),
)

== Problem

+ In speconaut, run any commit inside a session; its pre-commit and commit-msg hooks look up the bound task through `find_task` in `src/extraction/task_bridge.rs`.
+ 🔴 `find_task` calls `mindtape::load_project` and filters the result for one path, so every hook call walks every task root and content-hashes every task file, however large the backlog.
`crates/mindtape` exports `load_project` and `load_many`, and nothing that loads a single file under a project's config.


== TO-BE

```rust
pub fn load_one(root: &Path, config: &Config, path: &Path) -> Result<Option<Task>, LoadError>
```

- 🟢 A valid task file gives `Ok(Some(task))`, read through the same content-hash cache `load_project` uses, with its waits annotated as `load_project` does. Its core fields equal the same file's task out of `load_project`.
- 🟢 A file with no `task.with` gives `Ok(None)`, since `load_project` skips it silently.
- 🟢 A load failure gives `Err(LoadError)`, the same error `load_project` lists for that file.
- 🟢 A path outside every configured root gives `Err(LoadError)` naming the path, because such a file would get no folder tags and so differ from its `load_project` task.
- 🟢 Fields that need the whole set (`topo`, `blocks`, `children`, `progress`) stay empty on the task, and the docs on `load_one` say so.


== Changes

- `crates/mindtape/src/loader/tests.rs`: the red test first, one case each for a valid task, a file with no `task.with`, a load failure and a path outside the roots, plus core-field equality with `load_project` for the same file.
- `crates/mindtape/src/loader/mod.rs`: `load_one`, sharing the overrides, the cache lookup and the harvest of a miss with `load_project`.
- `crates/mindtape/src/loader/many.rs`: rename its private `load_one(root)` to `load_root`.
- `crates/mindtape/src/lib.rs`: re-export `load_one`.
- `CHANGELOG.md`: an entry under `[Unreleased]`.


== Alternatives

- Keep the full load and lean on the cache: the walk and hash still grow with the backlog, on every hook call.
- A `--file` filter on the loader options: one more knob on the batch path for what is a different question.


== Out of scope

- Graph-derived fields on a single file, which need every task.
- Switching speconaut's `find_task` over, which is speconaut's change to make.


== Dogfood

Not applicable here: `mt` commands read the whole set. speconaut's git hooks are the user.

