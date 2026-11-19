#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Stop warning about a task call in a template",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the task specifies the predicate, the two tests and the one behaviour that must not change.],
      enforced-practice: neutral,
      cheap-adoption: helps[a root of `.` stops printing a warning on every `mt ls`.],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the warning reports only a task saved where no root can load it.],
    ),
    process: (
      sound-codebase: helps[the loader stops reading a template it never loads.],
      docs-match-code: helps[the concepts page says a template earns no warning.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
)

== AS-IS

+ Configure a project with `[[tasks.roots]] path = "."` and the default template at `.mindtape/templates/task.typ`.
+ Run `mt ls`.
+ 🔴 It prints the warning on every run:

```
$ mt ls
warning: .mindtape/templates/task.typ:3: `task.with` call in a reserved path
hint: `.mindtape`, `.git` and `[templates] dirs` hold no task sources
```

+ Read `TaskSource::is_reserved` and `reserved_warning` in `crates/mindtape/src/task_source.rs`.
+ 🔴 A file under a configured template dir counts as reserved. A template is a `.typ` file whose whole job is a `task.with` call, and the warning fires on every template.

== Environment

- `mt` 0.6.0 built from `main` at the commit that filed this, on Linux.

== TO-BE

- 🟢 A template under a configured `[templates] dirs` entry earns no warning: it is what its directory is for, not a task saved in the wrong place.
- 🟢 A `.typ` file with a `task.with` call directly under `.mindtape` or `.git` still warns, because no root can load it.
- 🟢 The loader stops reading a template file it never loads.

```
$ mt ls
2026-11/add-an-on-rm-hook  Add an on.rm hook for removal actions
```

== Changes

- `crates/mindtape/src/task_source.rs`: `is_misfiled` is `is_reserved` without a template. `reserved_warning` reads it, and `descends_into` skips a template dir.
- `crates/mindtape/src/loader/visible.rs`: `is_candidate` admits a source or a misfiled task, never a template.
- `crates/mindtape/src/loader/tests.rs`: `a_template_is_not_a_task_and_earns_no_warning` covers both a `.mindtape` template and a configured one.
- `crates/mindtape/src/loader/tests_visible.rs`: the Git discovery test keeps the misfiled warning and drops the template one.
- `crates/mindtape/src/task_source_tests.rs`: a template is not misfiled, and a reserved component still is.
- `www/content/concepts.typ`: the template bullet says a template earns no warning.
- `CHANGELOG.md`: the fix under `[Unreleased]`.

== Out of scope

- The `exclude` key. It is a separate task, and this fix doesn't need a config key.
- `mt check`'s template pass. It already checks a template on its own.

== Dogfood

This repo's only root is `truth/tasks`, so it never sees the template. A project with a root of `.` stops printing the warning on every `mt ls`.
