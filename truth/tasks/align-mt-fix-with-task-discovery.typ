#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Align mt fix with task discovery",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "tooling", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the fixer and the loader agree on which files are tasks],
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
      (sound-codebase: helps[one scope rule replaces the ad-hoc skip list]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  links: depends-on(
    "define-and-export-one-task-source-policy.typ",
    "discover-task-files-through-git.typ",
  ),
  status: proposed(2026, 11, 9),
)

== AS-IS

+ In a root-`.` project with an ignored `target/generated.typ` that imports an unsupported prelude, run `mt fix --all --dry-run`.
+ 🔴 It enters `target/` and proposes an edit to generated output:

```
$ mt fix --all --dry-run
rewrite target/generated.typ  0.1.0 -> 0.5.0
would rewrite 1 import
```


== TO-BE

- 🟢 Default scope is `visible_task_sources` plus every in-project template dir, including pack template dirs.
- 🟢 `--all` is `visible_typ_files` (every Git-visible `.typ`) in the project plus template dirs, skipping reserved metadata; it includes non-task `.typ` such as `lib/` and docs.
- 🟢 Untracked ignored non-template files are in neither scope; tracked-ignored task sources are in both.
- 🟢 In a non-Git project, `--all` keeps the `.git`/`target`/`.mindtape/cache` skip list, adds reserved dirs and re-includes template dirs.
- 🟢 Tracked-deleted files are filtered before the fixer reads them; symlinked directories are not followed.

```
$ mt fix --all --dry-run
rewrite truth/tasks/real-task.typ  0.1.0 -> 0.5.0
rewrite .mindtape/templates/task.typ  0.1.0 -> 0.5.0
would rewrite 2 imports
```


== Changes

- `crates/mindtape-cli/src/cmd/fix/scope.rs`: build both scopes from `visible_task_sources` and `visible_typ_files` plus template dirs.
- `crates/mindtape-cli/src/cmd/fix/scope_tests.rs`, `crates/mindtape-cli/tests/it/fix_e2e.rs`: the red tests first.


== Out of scope

- Explicit file arguments; `mt fix` takes a fixer name, not a path.
- Prelude import matching and symlink policy beyond directory links.


== Dogfood

This project's default `mt fix` scope stays `truth/tasks` plus `.mindtape/templates`; `--all` stops entering its gitignored build trees.

