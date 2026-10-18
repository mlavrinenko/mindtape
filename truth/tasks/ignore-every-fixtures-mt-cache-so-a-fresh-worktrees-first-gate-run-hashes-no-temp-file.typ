#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ignore every fixture's mt cache so a fresh worktree's first gate run hashes no temp file",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a fresh fork's first just check passes without a re-run],
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
  status: proposed(2026, 10, 18),
)

== Problem

+ Running the first `just check` in a fresh `just fork` worktree
+ 🔴 #raw(".gitignore"): only the root `/.mindtape/cache/` is ignored, so the temp file `ensure_ignored` writes before a fixture's cache `.gitignore` exists lands in the `examples/rules/**` mmz scope

```
==> just docs::check failed (log: target/check/docs__check.log)
mmz: input `examples/rules/.mindtape/cache/..gitignore.1d2a68f0-2a7c-4940-bd22-5b79d32cf81d.tmp` disappeared after it was resolved
check: 1 of 33 gates failed: just docs::check (2 not run)
```
A re-run passed.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".gitignore").

== TO-BE

- 🟢 The first `just check` in a fresh worktree never fails on a cache temp file, because `.gitignore` holds this line.

```
.mindtape/cache/
```


== Changes

- `.gitignore`: unanchor `/.mindtape/cache/` so it ignores every fixture's cache too.


== Rollback

- Restore the anchored line.


== Out of scope

- `crates/mindtape/src/loader/cache.rs`, whose self-ignoring cache serves user projects.


== Dogfood

Every `just fork` in this repo starts with a first gate run that can't trip on it.

