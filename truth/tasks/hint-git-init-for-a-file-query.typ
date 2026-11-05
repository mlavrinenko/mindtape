#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hint git init for a file query",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "www", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[the first collector in a new project is one command away from working],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the page names the git requirement the error enforces],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("crates/mindtape/src/query/mod.rs"): A project outside git gets one `can't list files for a query: not a git repository` error per task, and the collector pages do not say a file query needs a working tree.

A temp project with an ast-grep collector but no `git init`:

```
$ mt check
error: tasks/a.typ:2: derived.callers: can't list files for a query: not a git repository (or any of the parent directories): .git
warning: built-in template task:3: derived.callers: can't list files for a query: not a git repository (or any of the parent directories): .git
```

`crates/mindtape/src/query/mod.rs` reports the git failure without a hint, and neither collector page says the query needs git.

== TO-BE

- 🟢 The error tells the reader to run `git init`.
- 🟢 Both collector pages say a file query reads a git working tree.
- 🟢 A red test pins the hint.

```
$ mt check
error: tasks/a.typ:2: derived.callers: can't list files for a query: not a git repository (or any of the parent directories): .git; run `git init` to make this project a working tree
```

== Changes

- `crates/mindtape/src/query/mod.rs`: append the `git init` hint when `git` reports `not a git repository`.
- `crates/mindtape/src/query/tests.rs`: a red test in a temp dir outside git.
- `www/content/config.typ` and `rules.typ`: say a file query reads a git working tree.

== Out of scope

- Making file queries work outside git.
- A different file walk.

== Dogfood

A new project that adds a collector before `git init` gets the fix in the error.
