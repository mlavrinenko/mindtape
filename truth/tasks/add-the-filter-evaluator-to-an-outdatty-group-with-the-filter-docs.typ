#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the filter evaluator to an outdatty group with the filter docs",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a changed filter function flags the two pages that describe it],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Moving the skill's filter list into skills/mindtape/references/filters.md.
+ 🔴 #raw("outdatty.yaml"): No outdatty group lists the filter evaluator as a source, so a new or renamed filter function leaves the filter docs unflagged.

`rg -n 'filter_eval|filter_predicates|filter_strings|filter_compare|src/filter\.rs' outdatty.yaml` prints nothing. The function names (`has`, `contains`, `touches`, `stale` and the rest) are matched in `crates/mindtape/src/filter_eval.rs:72-119`. `www/content/filters.typ` and `skills/mindtape/references/filters.md` list them by hand.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.yaml").

== TO-BE

- 🟢 A change to a filter function in `crates/mindtape/src/filter*.rs` makes `outdatty check` name `www/content/filters.typ` and `skills/mindtape/references/filters.md` as pending.


== Changes

- `outdatty.yaml`: a group with `filter.rs`, `filter_eval.rs`, `filter_predicates.rs`, `filter_strings.rs` and `filter_compare.rs` under `crates/mindtape/src/` as sources, and the two filter docs as dependents.
- `outdatty.lock`: the recorded group.


== Rollback

Delete the group from `outdatty.yaml` and record the lock again.

== Out of scope

- Deriving the function list from the code: the docs say what each function means, and no fact file carries that.


== Dogfood

The group gates this repo's filter page and the skill's filter reference.
