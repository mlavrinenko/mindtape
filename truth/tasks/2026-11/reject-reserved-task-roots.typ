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
    product: verdicts(
      (
        self-describing: helps[the config error names the reserved path at load],
        enforced-practice: helps[a root that can never work never reaches the rest of the tool],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a contradictory config fails once, with the path named],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  links: depends-on("define-and-export-one-task-source-policy.typ"),
  status: done(2026, 11, 10),
)

== AS-IS

+ Set `[[tasks.roots]] path = ".mindtape/templates"` in a project and run `mt ls`.
+ 🔴 The root is accepted, and the template is loaded as a task:

```
$ mt ls --json | jq '.tasks | length'
1
```


== TO-BE

- 🟢 Working-tree `load_config` rejects a root inside `.mindtape`, `.git`, or a configured template dir, using the task-source policy's normalized forms.
- 🟢 A template dir equal to or an ancestor of the project root or a task root is rejected.
- 🟢 A root of `.` that contains template dirs stays valid.
- 🟢 The error names the root, the reserved path and the config layer that declared it.
- 🟢 Historical state configs skip this check; `mt config show` remains a raw merged view.

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

This project's roots are valid, so its config is unchanged; the rule protects a root-`.` adopter who points a root at a template dir.

