#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve unchanged task files against a past tree's package pin",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.85,
    ease: 3,
  ),
  difficulty: 4,
  estimate: (src-files: 2),
  tags: ("found", "mt", "tests", "stateless"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[a range resolves every task from the tree it checks],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 3),
)

== AS-IS

+ Verifying the 0.4.0 `mt check --commits` package-version fix
+ 🔴 #raw("crates/mindtape/src/commit/tasks.rs"): A commit state re-evaluates only the task files the commit changed, so an unchanged file still resolves `@local` through the working tree's pin.

`overlay` starts from the working tree's task list and rebuilds only the paths in `StateSource::differs`, filled from the range's changed files:

```
let tasks = self.tasks.iter().filter_map(|task| { ... }).collect();
```

A range across a pin move therefore uses the commit's `lib/` only for task files that commit touched. A pin path or name also comes from the working tree's config, not the past tree's.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/commit/tasks.rs").

== TO-BE

- 🟢 Every task file in a commit state resolves against that state's own package files, changed or not.
- 🟢 A range across a pin move reads the past tree's pin path and version.

```
$ mt check --commits v0.4.0..HEAD
ok: 2 commits
```


== Changes

- `crates/mindtape/src/commit/tasks.rs`: overlay every task file from the state tree, and read pins from that tree's config.
- `crates/mindtape-cli/tests/it/commit_range_e2e.rs`: cover a pin move that leaves a task file unchanged.


== Out of scope

- A pin whose manifest is missing still serves every version; that fallback is deliberate and separate.


== Dogfood

This repo's own ranges across a prelude bump resolve every task from its commit tree.

