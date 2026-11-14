#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Reject reserved task roots",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a root that can never work never reaches the rest of the tool.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the config error states the reserved path at load.],
    ),
    process: (
      sound-codebase: helps[a contradictory config fails once, with the path named.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  links: depends-on("define-and-export-one-task-source-policy.typ"),
  status: done(2026, 11, 10),
)

== AS-IS

+ Set `[[tasks.roots]] path = ".mindtape/templates"` in a project and run `mt ls`.
+ 🔴 The loader accepts the root and loads the template as a task:

```
$ mt ls --json | jq '.tasks | length'
1
```


== TO-BE

- 🟢 Working-tree `load_config` rejects a root inside `.mindtape`, `.git`, or a configured template dir, using the task-source policy's normalized forms.
- 🟢 The loader rejects a template dir equal to or an ancestor of the project root or a task root.
- 🟢 A root of `.` that contains template dirs stays valid.
- 🟢 The error states the root, the reserved path and the config layer that declared it.
- 🟢 Historical state configs skip this check. `mt config show` remains a raw merged view.

```
$ mt ls
error: tasks.roots: `.mindtape/templates` is reserved for project metadata and templates
```


== Changes

- `crates/mindtape/src/config/layers.rs`: run the check from the working-tree `load_config`, not `build_config`.
- `crates/mindtape/src/config/roots.rs`: validate roots and template dirs with the task-source policy.
- `crates/mindtape/src/config/layers_tests.rs`, `crates/mindtape/src/config/roots_tests.rs`: the red tests first, including provenance and the historical-state bypass.


== Alternatives

- Warn only: a root that can never yield tasks survives silently.
- Silently ignore the root: the loaded template is the bug.
- Refuse at working-tree config load: chosen, the earliest point with the config in hand.


== Out of scope

- Nonexistent, file and unreadable roots.
- Validating historical state configs, and changing root precedence.


== Dogfood

This project's roots are valid, so its config does not change. The rule protects a root-`.` adopter who points a root at a template dir.

