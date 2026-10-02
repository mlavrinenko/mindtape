#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept delete, group and hyphenated verbs in the imperative-title rule",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the title rule refuses only titles that are not imperative],
      cheap-adoption: helps[a team adopting mindtape-tasks does not hit false refusals on common verbs],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    2,
  )[delete, group, update, rate-limit and about 220 more verbs added to dev-verbs.txt; verbs.txt is frequency-derived and stays untouched],
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ In a project that extends `mindtape-tasks`, run `mt add "Delete the old fixture"`, `mt add "Group the index by section"` and `mt add "Rate-limit the login endpoint"`.
+ 🔴 Each prints `rules.imperative-title: start the title with an imperative verb` and writes nothing.
+ `crates/mindtape/src/dicts/verbs.txt` and `dev-verbs.txt` lack `delete` and `group`. `word(title, 1)` returns `Rate-limit` whole, and no dict lists hyphenated verbs.


== Environment

- `mt 0.3.0` at commit `00dd4fee`, Linux.


== TO-BE

- 🟢 The three titles above pass.
- 🟢 A hyphenated first word passes when the dict lists it whole, and the dicts list the common ones.


== Changes

- `crates/mindtape/tests/imperative_title.rs`, new: the three titles, red first.
- `crates/mindtape/src/dicts/verbs.txt`, `dev-verbs.txt`: add `delete`, `group` and the common hyphenated verbs (`rate-limit`, `re-run`, `hot-fix`).
- `crates/mindtape/src/filter*`: only if the chosen rule needs a change to `word()` or `in_dict()`.


== Out of scope

- Grammar checks beyond the first word.


== Dogfood

This repo's own task titles go through the rule on every `mt add`.

