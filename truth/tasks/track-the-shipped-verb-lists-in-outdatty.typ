#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the shipped verb lists in outdatty",
  kind: "docs",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (src-files: 2),
  x-goals: (
    product: verdicts(
      (:),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (
        docs-match-code: helps[a word edited in a shipped list flags the pages that quote it],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== AS-IS

+ Run `rg -l 'mindtape-dev-verbs' www skills` in the repo.
+ 🔴 `www/content/named-sets.typ` and `www/content/start-titles-with-a-verb.typ` quote the word lists in `crates/mindtape/src/dicts/*.txt` (the 500-verb count, `refactor`, `deprecate`, `rate-limit`, "Backport" missing from both).
+ 🔴 No group in `outdatty.yaml` lists the dicts as a source, so a word edited out of a list drifts the pages with no gate failing.


== TO-BE

- 🟢 An edit to `verbs.txt` or `dev-verbs.txt` makes `just outdatty check` fail until the pages that quote the lists are reviewed.


== Source of truth

- `crates/mindtape/src/dicts/verbs.txt` and `crates/mindtape/src/dicts/dev-verbs.txt`: the words and the 500 count.


== Changes

- `outdatty.yaml`: a `dict-words-docs` group with the two lists as its source and the pages that quote them as dependents.
- `outdatty.lock`: record the group.


== Out of scope

- `crates/mindtape/src/config/dicts.rs`: how dictionaries merge is not restated by these pages' word claims.


== Dogfood

Not applicable: the group guards this repo's own docs and ships nothing.

