#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check commit trailers against live tasks",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "cli"),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a commit names a live task by stem, checked by a hook instead of review],
      cheap-adoption: helps[one hook line opts a repo in; nothing is installed for it],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
)

== Problem

+ `git log -200 --format='%(trailers:key=Refs,valueonly)'` in this repo.
+ 🔴 83 trailers are repo-root paths and one is a bare stem: nothing checks them, a path breaks when a task moves folders, and a mistyped trailer silently names nothing.


== TO-BE

- 🟢 `mt check --commit-msg <file>` is a read command a `commit-msg` hook calls; a repo opts in with one hook line, nothing installs it.

```toml
[commits]
trailer = "Refs"
require = true
exempt  = '^task:'      # regex over the subject
filter  = '!terminal'   # which tasks a commit may name
```

```
$ git commit -m 'fix: typo'
error: commit message has no `Refs:` trailer; name the task by stem
$ git commit -m 'fix: x' -m 'Refs: truth/tasks/foo.typ'
error: Refs: `truth/tasks/foo.typ` is a path; write the stem `foo`
```

- 🟢 Reads only the message file and the tree, so it works without git.


== Changes

- `crates/mindtape-cli/tests/` (red first): missing, path-shaped, unknown and filtered-out trailers refused; an exempt subject passes.
- `crates/mindtape/src/commit_msg.rs` (new): trailer parsing and the checks.
- `crates/mindtape/src/config.rs` and its JSON Schema: `[commits]`.
- `crates/mindtape-cli/src/` check command: the `--commit-msg` flag.
- `www/content/config.typ`: document `[commits]` and the flag.


== Alternatives

- Repo-root or task-root path: breaks when a task moves folders; a task-root path is ambiguous with several roots.
- A stable `id:`: survives `mt mv` but is unreadable; stems are already unique project-wide (`mt check` rejects a duplicate).
- An installer for the hook: users wire hooks with their own tool.


== Out of scope

- This repo opting in: `truth/tasks/switch-this-repos-refs-trailers-to-stems.typ`.
- Resolving renamed or deleted tasks: `truth/tasks/add-a-touches-filter-over-the-files-a-tasks-commits-changed.typ`.


== Dogfood

This repo's `commit-msg` hook runs it once the opt-in task lands.

