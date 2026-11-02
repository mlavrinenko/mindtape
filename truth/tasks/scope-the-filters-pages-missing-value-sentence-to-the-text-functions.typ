#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Scope the filters page's missing-value sentence to the text functions",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 3),
  tags: ("found", "www"),
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
        docs-match-code: helps[the filters page states the missing-field rule the evaluator applies],
      ),
      "sound-codebase",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== AS-IS

+ Reviewing what a filter does with a field the task doesn't carry
+ 🔴 #raw("www/content/filters.typ"): the Text functions section says a missing field never matches, yet `!=` matches it, as the Operators section on the same page says

```
$ mt ls 'miss(due) && due != "x"'
your filter · 9 of 14 tasks
$ mt ls 'miss(due) && due == "x"'
your filter · 0 of 14 tasks
```
`compare.rs` makes `!=` hold on a missing field on purpose.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/content/filters.typ").

== TO-BE

- 🟢 The filters page states one missing-value rule: `!=` matches, `==`, the orderings and the text functions don't, and `!` flips each of them.
- 🟢 A captured run shows `!` letting a task with no value through.


== Changes

- `www/content/filters.typ`: scope the sentence to `matches()` and `in_dict()`, and add the `!` case to the Operators section with a capture.
- `www/generate-reference.sh`: capture the `>` and `!(<=)` pair over `estimate.src-files`.
- `crates/mindtape/src/filter/strings.rs`: say the text functions read no value as a miss, as `==` does, not as every comparison does.


== Rollback

- Revert the commits.


== Out of scope

- The evaluator. Its rule is consistent and pinned by `missing_field_is_unequal_to_every_literal`.
- Other docs pages, which already state the rule right.


== Dogfood

Not applicable. The repo's own rules already rely on the documented rule.

