#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check commit messages against commit rules",
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

- 🟢 `mt check --commit-msg <file>` evaluates `[[commits.rules]]` (`name`/`when`/`assert`/`message`/`severity`, the `[[tasks.rules]]` shape) with `scope = "commit"`; a repo opts in with one `commit-msg` hook line.
- 🟢 Commit fields: `subject`, `refs` (trailer values as stems), `files` (staged paths), `tasks_only`.
- 🟢 A trailer value that is a path, or names no task, is always refused, naming the stem to write.
- 🟢 Rules merge by `name` across rule packs, so a project overrides or disables a shipped one.

```toml
[commits]
trailer = "Refs"

[[commits.rules]]
name    = "work-names-a-task"
scope   = "commit"
when    = '!tasks_only'
assert  = 'has(refs)'
message = "name the task this commit works on: `Refs: <stem>`"
```

- 🟢 Outside a git repository `--commit-msg` fails naming the reason.


== Changes

- `crates/mindtape-cli/tests/commit_msg_e2e.rs` (red first): a missing, path-shaped and unknown trailer each refused; a `when` that does not match passes.
- `crates/mindtape/src/commit_msg.rs` (new): trailer parsing, the staged file list, the commit record the filter reads.
- `crates/mindtape/src/config_commits.rs` (new) and the JSON Schema: `[commits]` and `[[commits.rules]]`, merged by `name` like `[[tasks.rules]]`.
- `crates/mindtape-cli/src/check.rs`: the `--commit-msg` flag.
- `www/content/config.typ`: document both.


== Alternatives

- Fixed knobs (`require`, `exempt`, `filter`): cannot tell a trunk team's start commits from a worktree team's, a `docs/` typo from a `services/` change, or a bot commit from a human one.
- `run = "script"` rules: a user's own git hook already does that.
- Repo-root or task-root path in the trailer: breaks when a task moves folders; stems are unique project-wide (`mt check` rejects a duplicate).
- A stable `id:`: survives `mt mv` but is unreadable.
- An installer for the hook: users wire hooks with their own tool.


== Out of scope

- `scope = "task"` rules (`into()`, `named`, `before()`): `truth/tasks/evaluate-task-scope-commit-rules.typ`.
- Checking a commit range in CI: `truth/tasks/check-commit-rules-over-a-commit-range.typ`.
- This repo opting in: `truth/tasks/migrate-this-repos-refs-trailers-to-stems.typ`.
- Resolving renamed or deleted tasks: `truth/tasks/add-a-touches-filter-over-the-files-a-tasks-commits-changed.typ`.


== Dogfood

This repo's `commit-msg` hook runs it once the opt-in task lands.
