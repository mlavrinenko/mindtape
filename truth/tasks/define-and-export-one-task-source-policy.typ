#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Define and export one task-source policy",
  kind: "fix",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a template is no longer listed as a task],
        implementable-tasks: helps[one exported policy names a task source],
      ),
      ..(
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the loader and later consumers share one definition instead of drifting],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 9),
)

== AS-IS

+ In a project whose config sets `[[tasks.roots]] path = "."`, run `mt ls`.
+ 🔴 The template is listed as a task, and `mt check` reports it:

```
$ mt ls
SLUG#  STATUS    TITLE  FILE
ss9              .mindtape/templates/task.typ
$ mt check
error: .mindtape/templates/task.typ:3: status: missing required status
```


== Environment

`mt 0.5.0` on Linux, in a project whose `.mindtape/config.toml` sets `[[tasks.roots]] path = "."`.


== TO-BE

- 🟢 One exported `TaskSource` policy classifies a `TaskPath` with project-relative/absolute forms and a regular/symlink/missing kind.
- 🟢 A source is a regular `.typ` under a root, outside `.mindtape`/`.git` components (case-insensitive) and outside every configured template dir.
- 🟢 Roots and template dirs keep `RootSpec::dir()`; the policy carries separate normalized forms, so commit and tag derivation do not change.
- 🟢 `collect_items`, `load_one` and the fallback walk call it; a reserved `.typ` with a `task.with` call emits one warning.
- 🟢 `mt ls` lists only real tasks, and `mt check` passes once the template's own slots are valid.

```
$ mt ls
SLUG#  STATUS    TITLE      FILE
5zv    proposed  Real task  dev/real-task.typ
$ mt check
ok: 30 tasks
```


== Changes

- `crates/mindtape/src/task_source.rs`: the policy, `TaskPath`, normalization and reserved-path warning.
- `crates/mindtape/src/lib.rs`: export the policy for the commit layer and the CLI.
- `crates/mindtape/src/config/roots.rs`: normalized root forms beside `RootSpec::dir()`.
- `crates/mindtape/src/config/templates.rs`: normalized template-dir forms.
- `crates/mindtape/src/loader/mod.rs`: use the policy in `collect_items`, `load_one` and the warning.
- `crates/mindtape/src/task_source_tests.rs`, `crates/mindtape/src/loader/tests.rs`, `crates/mindtape/src/loader/tests_one.rs`, `crates/mindtape-cli/tests/it/check_templates_e2e.rs`: the red tests first.


== Out of scope
- Candidate validation; the write guard owns the destination.

- Git-ignore visibility, commit classification, writer guards, config-root rejection and `mt fix`.
- Symlinked task files; this change deliberately stops treating them as sources.
- A textual prefilter for markerless `.typ`.


== Dogfood

This project's task root is `truth/tasks`, so its own listing is unchanged; the root-`.` repro project exercises the policy.

