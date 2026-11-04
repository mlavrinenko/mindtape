#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the ast-grep docs and generated output in step",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "docs", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a reader learns the shape query from the same page as grep and glob],
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
        docs-match-code: helps[the page and the changelog name the same supported set and problem behavior as the engine],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 5),
  links: parent("add-ast-grep-queries-as-a-shared-query-engine.typ")
    + depends-on("expose-ast-grep-through-the-collector-consumer.typ"),
)

== AS-IS

+ Read `www/content/estimates.typ`.
+ 🔴 It documents `grep` and `glob` only, so `ast_grep` has no user page.
+ `CHANGELOG.md` has no entry for the shared engine or the shape query.


== TO-BE

- 🟢 `www/content/estimates.typ` documents `ast_grep(language, pattern, glob?)`, the Rust-only set and the unresolved-problem behavior.
- 🟢 The page shows a count over real call sites and the `mt eval` breakdown.
- 🟢 `CHANGELOG.md` names the shared engine and the shape query under `[Unreleased]`.
- 🟢 Generated output is regenerated, never hand-edited.


== Source of truth

- `crates/mindtape/src/query/mod.rs`: the supported languages and the problem text.
- `lib/prelude.typ`: the helper signature.
- `www/generated/estimates.json`: the checked slices the page renders.


== Changes

- `www/content/estimates.typ`: the ast-grep subsection and example.
- `www/generated/estimates.json`: regenerate if the example project moves.
- `CHANGELOG.md`: the two feature entries.


== Out of scope

- A new docs page; the estimate page already owns file-count queries.
- The collector docs, which ride with the collector wave.


== Dogfood

The page is the reference for the shape query this backlog starts using.

