#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Align the ast-grep docs with the code",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.98,
    ease: 8,
  ),
  difficulty: 2,
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
        readable-tasks: helps[the page states the real granularity and the path rule],
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
        docs-match-code: helps[the prose and the skill name the behavior the code has],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
)

== AS-IS

+ Reviewing both waves of the ast-grep milestone after it landed
+ 🔴 #raw("www/content/estimates.typ"): The new pages claim ast-grep sees macro expansions and call-site granularity, say a collector names no path set, and the skill still imports 0.3.0.

- `www/content/estimates.typ` says text cannot see a call site a macro produces, implying ast-grep can; it cannot see expansions.
- `www/content/rules.typ` says a rule bounds a call-site count; `derived.callers` is one path per file.
- `www/content/config.typ` says the collector names no path set, while the same paragraph allows a `@name` glob.
- `skills/mindtape/references/hand-editing.md` imports `@local/mindtape:0.3.0`, and `lib/typst.toml` is 0.4.0.
- Neither collector page says globs use repository-root paths, so a nested project's `src/**` matches nothing.

== TO-BE

- 🟢 The macro sentence says what ast-grep actually does: it matches source shape, not comments, strings or definitions.
- 🟢 The collector text says file count, not call-site count.
- 🟢 `glob` is described as an optional narrowing set, not a missing path set.
- 🟢 Both collector pages say paths are repository-root relative, as `changed()` reads them.
- 🟢 The skill imports 0.4.0.

```
A glob matches repository-root paths. In a nested project, prefix the
project directory, as in `sub/src/**`.
```

== Changes

- `www/content/estimates.typ`: replace the macro claim and add the `.rs` extension rule.
- `www/content/rules.typ`: say file count, describe `glob`, add the repo-root path rule.
- `www/content/config.typ`: same path-set and repo-root wording.
- `skills/mindtape/references/hand-editing.md`: bump the import to 0.4.0.
- `CHANGELOG.md`: only if a wording change needs a note.

== Out of scope

- Changing the query behavior to project-relative paths.
- New pages.
- The checked-example finding, which has its own task.

== Dogfood

A reader sizes a rename from a count that matches what the engine actually sees.
