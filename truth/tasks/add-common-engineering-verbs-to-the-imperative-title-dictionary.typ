#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add common engineering verbs to the imperative-title dictionary",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (files: 1),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: helps[a plain imperative title passes the rule instead of being bent around a word list],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: wip(2026, 9, 27),
)

== Problem

+ filing tasks with \`mt add\` in one session
+ 🔴 #raw("crates/mindtape/src/dicts/verbs.txt"): The imperative-title rule refused five ordinary imperative verbs in one session, forcing title rewrites.

```
error: roll-up-child-progress-over-parent-links: rules.imperative-title: start the title with an imperative verb
error: compute-child-progress-over-parent-links: rules.imperative-title: start the title with an imperative verb
error: page-an-ls-listing-with-skip-and-a-next-command-hint: rules.imperative-title: start the title with an imperative verb
error: filter-tasks-by-the-files-their-commits-touched: rules.imperative-title: start the title with an imperative verb
error: switch-this-repos-refs-trailers-to-stems: rules.imperative-title: start the title with an imperative verb
```
`switch`, `filter`, `compute`, `roll` and `page` are absent from `verbs.txt` and `dev-verbs.txt`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/dicts/verbs.txt").

== TO-BE

- 🟢 `switch`, `filter`, `compute` and `roll` pass the rule; `page` goes to `dev-verbs.txt` as a CLI sense.


== Changes

- `crates/mindtape/src/dicts/verbs.txt`, `dev-verbs.txt`: the missing verbs, sorted.


== Rollback

Revert the commit.


== Out of scope

- Replacing the dictionary with a part-of-speech tagger: the word list stays reviewable.


== Dogfood

Titles filed here stop bouncing off common verbs.

