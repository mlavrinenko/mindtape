#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Reject dot segments in a static add-hook dir",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  links: (
    (
      task: "let-rules-and-templates-choose-where-new-tasks-land.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a bad destination is a config error, not a late load failure],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (sound-codebase: helps[static and derived dirs share one path rule]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 11),
)

== AS-IS

+ Reviewing the static dir validation against the derived dir validation in the add-hook commit.
+ 🔴 #raw("crates/mindtape/src/config/add_hooks_dir.rs"): `validate_dir` accepts `.` and `..` components while `validate_derived_dir` refuses them.

```
[[on.add]]
name = "escape"
actions = [{ dir = "../outside" }]
```

The config loads. The add then refuses in `resolve_dir` with `the task directory is outside the configured roots`, so the failure is late and repeats the path rule. `tasks//bug` and `tasks/./bug` also load and write, so the practical problem is inconsistent validation, not a late load error.


== TO-BE

- 🟢 A static `dir` rejects empty, `.` and `..` components at config load.
- 🟢 A derived `dir` keeps its add-time check, because its value is a Typst return.
- 🟢 The message names the hook, the action and the component.

```
error: on.add[0].actions[0].dir: a dir must not hold `.` or `..`
```


== Changes

- `crates/mindtape/src/config/add_hooks_dir.rs`: factor one component check and call it from both validators, with the static side running at load.
- `crates/mindtape/src/config/add_hooks_tests.rs`: refuse `../up`, `tasks/./bug` and `tasks//bug` at load.
- `crates/mindtape-cli/tests/it/add_hooks_e2e.rs`: pin the static config-load refusal.


== Out of scope

- The absolute-path rule both validators already share, and the derived dir's existing add-time check.


== Dogfood

A project hook cannot defer a bad destination to a half-built task.

