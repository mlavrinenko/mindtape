#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Discover task files through Git",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[untracked ignored generated files stop appearing as tasks],
        cheap-adoption: helps[a root of `.` works without an ignore list of its own],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (sound-codebase: helps[ignored build trees are never compiled as tasks]),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  links: depends-on("define-and-export-one-task-source-policy.typ"),
  status: done(2026, 11, 10),
)

== AS-IS

+ In a repository whose config sets `[[tasks.roots]] path = "."` and whose `.gitignore` hides `target/`, put a task file at `target/gen.typ`.
+ 🔴 `mt ls` lists the generated file as a task:

```
$ mt ls
SLUG#  STATUS    TITLE           FILE
gen    proposed  Generated task  target/gen.typ
```


== TO-BE

- 🟢 Git lists each root: tracked regular files and untracked files it does not ignore.
- 🟢 A tracked non-symlink stays a task under an ignore pattern; an untracked ignored file does not.
- 🟢 A tracked-deleted file is not a working-tree task; a commit state still lists it.
- 🟢 An ignored root is not walked; one warning says only its tracked files are visible.
- 🟢 A non-Git project walks with the policy's skips and no directory symlinks.
- 🟢 Fallback only for a non-worktree root or missing `git`; other Git errors are load errors naming the root. Cache per load; export a destination probe, a per-root task-source listing and a project-wide visible listing.

```
$ mt ls
track  proposed  Tracked task    tasks/tracked.typ
new    proposed  Untracked task  tasks/new.typ
```


== Changes

- `crates/mindtape/src/loader/mod.rs`: Git listing, fallback, per-load cache, task 1's reserved-path warning, `would_be_visible`, `visible_task_sources` and `visible_typ_files`.
- `crates/mindtape/src/query/mod.rs`: share the NUL-path listing and Git-error classification.
- `crates/mindtape/src/lib.rs`: export all three helpers.
- `crates/mindtape/src/loader/tests.rs`, `crates/mindtape-cli/tests/it/ls_path_e2e.rs`: the red tests first, with Git config isolated.


== Alternatives

- The `ignore` crate: no subprocess, but it cannot tell a tracked file from an ignored one, so tracked tasks would differ from commit states.
- A hardcoded skip list: misses nested `.gitignore`, global excludes and new build dirs.
- `git ls-files` per root: chosen for exact Git semantics and reuse of the existing query pattern.


== Out of scope

- Walking an ignored root; it is warned about, not exempted.
- Nested repositories and submodules; Git does not descend, and the fallback keeps its current walk.
- A config knob for Git rules, `.hg`/`.svn`, a textual prefilter, and a runtime warning for ignored tasks.


== Dogfood

This project's task root is `truth/tasks`, so its own scan is narrow; the example projects and the root-`.` personal backlog exercise the Git listing.

