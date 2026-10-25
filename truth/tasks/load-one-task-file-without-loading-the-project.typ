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

- 🟢 `mindtape::load_one(root, config, path)` returns that one task, or the warning that kept it from loading, through the same content-hash cache `load_project` uses.
- 🟢 Fields that need the whole set (`topo`, `blocks`, `children`, `progress`) are absent on a task `load_one` returns, and its docs say so.


== Changes

- `crates/mindtape/src/loader/tests.rs`: the red test first, `load_one` returning the same core fields `load_project` gives that file, and a warning for a file that is not a task.
- `crates/mindtape/src/loader/mod.rs`: `load_one`, sharing the overrides, the cache lookup and the harvest of a miss with `load_project`.
- `crates/mindtape/src/lib.rs`: re-export `load_one`.


== Alternatives

- Keep the full load and lean on the cache: the walk and hash still grow with the backlog, on every hook call.
- A `--file` filter on the loader options: one more knob on the batch path for what is a different question.


== Out of scope

- Graph-derived fields on a single file, which need every task.
- Switching speconaut's `find_task` over, which is speconaut's change to make.


== Dogfood

Not applicable here: `mt` commands read the whole set. speconaut's git hooks are the user.

