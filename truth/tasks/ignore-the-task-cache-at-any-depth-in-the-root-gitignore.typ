#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ignore the task cache at any depth in the root gitignore",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate hashes the files it names, not the scratch another gate writes],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 19),
)

== Problem

+ Running `just check`, where mmz hashed a cache temp file that another gate renamed away
+ 🔴 #raw(".gitignore"): the line `.mindtape/cache/` is anchored to the repo root, so only the inner `.gitignore` that `mt` writes keeps an `examples/` cache out of mmz's inputs

```
.mmz/config.yaml:155:    - "examples/**"
.mmz/config.yaml:193:    - "examples/across-projects/**"
$ git check-ignore -v examples/minimal/.mindtape/cache/x
examples/minimal/.mindtape/cache/.gitignore:1:*
```

Without that inner file, `mmz --status=json` lists the cache in nine rules. With `**/.mindtape/cache/` in the root file it lists none.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".gitignore").

== TO-BE

- 🟢 No mmz input holds a file under any `.mindtape/cache/`, from the moment `mt` creates it:

```
# .gitignore
**/.mindtape/cache/
```

- 🟢 A test fails when the root `.gitignore` leaves an `examples/` cache unignored.


== Changes

- `.gitignore`: write the cache line as `**/.mindtape/cache/` and fix the comment above it.
- `crates/mindtape-cli/tests/it/gitignore_cache_e2e.rs`: ask `git check-ignore` about a cache path in a repo that holds only the real `.gitignore`.


== Rollback

- Revert the `.gitignore` line and delete the test file.


== Out of scope

- The inner `.gitignore` that `mt` writes, which still keeps a cache clean outside this repo.
- The order in which `mt` creates the key directory and that file.


== Dogfood

Every `just check` in a fresh worktree stops hashing the caches that parallel gates create.
