#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Finish the docs and comments the fixes left behind",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.98,
    ease: 8,
  ),
  difficulty: 1,
  links: (
    (
      task: "add-ast-grep-queries-as-a-shared-query-engine.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "www", "docs", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[the page states the detail and the file set the command shows],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the prose and the internal comments name the current behavior],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 6),
)

== AS-IS

+ Verifying the fixes for the ast-grep review
+ 🔴 #raw("www/content/config.typ"): The collector pages still say only a Typst collector names a detail and that a glob-less query reads every git file, and the distinct note is undocumented.

- `www/content/config.typ` and `www/content/rules.typ` say a Typst collector also names its module and function, while `mt rules` now names an ast-grep query too.
- `www/content/cli/rules.typ` repeats the Typst-only claim.
- The collector pages say "without one it reads every file git lists", now false because only the language's files are read.
- `www/content/estimates.typ` does not mention the distinct-union note.
- `crates/mindtape/src/config/collect.rs` says the collector names no path set, and `crates/mindtape/src/config/active_rules.rs` says the kind is `regex` or `typst`.

== TO-BE

- 🟢 Every page says a Typst collector names its module and function and an ast-grep collector names its language, pattern and glob.
- 🟢 A glob-less ast-grep query reads every file of its language that git lists.
- 🟢 The estimates page documents the distinct-union note.
- 🟢 The internal comments match the shape.

```
A Typst collector names its module and function; an ast-grep collector
names its language, pattern and optional glob.
```

== Changes

- `www/content/config.typ`, `rules.typ`, `cli/rules.typ`: fix the detail claim.
- `www/content/config.typ`, `rules.typ`: say "every file of that language git lists".
- `www/content/estimates.typ`: document the distinct-union note.
- `crates/mindtape/src/config/collect.rs`, `active_rules.rs`: update the stale doc comments.
- `just docs gen` before the docs gates.

== Out of scope

- New pages.
- Changing the CLI output.

== Dogfood

A reader of the collector pages learns what `mt rules` actually prints and what a glob-less query reads.
