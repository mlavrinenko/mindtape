#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Document the task-source contract",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a reader finds one statement of what a task file is],
        self-describing: helps[the docs explain why the template is not a task],
        implementable-tasks: helps[the contract points at the code that owns each rule],
      ),
      ..(
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (docs-match-code: helps[every claim links to the source that holds it]),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  links: depends-on(
    "define-and-export-one-task-source-policy.typ",
    "discover-task-files-through-git.typ",
    "expose-each-commit-states-config-and-tasks.typ",
    "make-commit-classification-use-the-states-config.typ",
    "refuse-reserved-or-ignored-task-destinations.typ",
    "reject-reserved-task-roots.typ",
    "align-mt-fix-with-task-discovery.typ",
  ),
  status: proposed(2026, 11, 9),
)

== AS-IS

+ Open the "A task is a file" section of `www/content/concepts.typ` and the `tasks_only` definition in `www/content/commit-rules.typ`.
+ 🔴 They say `mt` reads every `.typ` under the roots and counts any such path as a task, contradicting the template pass, Git-visible discovery and the task-source policy.


== TO-BE

- 🟢 One canonical section defines a task source: extension, root, reserved names, template dirs, Git visibility, symlink/case policy, outside-project roots and the non-Git fallback.
- 🟢 It states that ignored paths are not scanned, existing ignored tasks become invisible but are not deleted, and global or info excludes are machine-local.
- 🟢 Concepts, templates, config, task-files, commit-rules, trace-commits and the add/import/recur/ls/check/fix pages link to it.
- 🟢 The root-`.` example explains why `.mindtape/templates/task.typ` is not a task; a second example shows a tracked-ignored task and a symlinked task file that stops being a source.
- 🟢 The changelog has a migration section, and a docs check asserts every listed page links to the canonical anchor.


== Source of truth

- `crates/mindtape/src/task_source.rs`: the policy, `TaskPath`, normalization and reserved-path warning.
- `crates/mindtape/src/config/roots.rs`, `config/templates.rs`, `config/layers.rs`: normalized forms and root rejection.
- `crates/mindtape/src/commit/tree.rs`, `commit/mod.rs`: commit classification.
- `crates/mindtape-cli/src/cmd/write_guard.rs`, `cmd/add/target.rs`: destination refusals.
- `crates/mindtape-cli/src/cmd/fix/scope.rs`, `cmd/check/templates.rs`: fixer and template pass.


== Changes

- `www/content/concepts.typ`: the canonical task-source section.
- `www/content/templates.typ`, `config.typ`, `task-files.typ`: link and narrow claims.
- `www/content/commit-rules.typ`, `trace-commits-to-tasks.typ`: the new `tasks_only` definition, including markerless `.typ`.
- `www/content/cli/add.typ`, `cli/import.typ`, `cli/recur.typ`, `cli/ls.typ`, `cli/check.typ`, `cli/fix.typ`: link, refusal and scope text.
- `CHANGELOG.md`: the migration and behavior changes.


== Out of scope

- Rewriting every mention; link to the canonical section.
- The comparison pages and marketing copy.
- Any claim of a `mt fix <file>` form, which does not exist.


== Dogfood

This project's own docs link to the section and `just docs-check` holds the claims to the code.

