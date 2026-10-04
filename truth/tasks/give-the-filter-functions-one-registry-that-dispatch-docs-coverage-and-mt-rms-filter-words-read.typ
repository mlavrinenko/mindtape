#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give the filter functions one registry that dispatch, docs coverage and mt rm's filter words read",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
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
      sound-codebase: helps[a new filter function cannot be dispatched yet missing from the word list],
      docs-match-code: helps[a filter function with no doc entry fails a gate instead of waiting on an outdatty review],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing commit 1368e19c, which added the filter-docs outdatty group
+ 🔴 #raw("crates/mindtape/src/filter_eval.rs"): the function names live in a hand-kept WORDS array beside two match blocks that repeat four arms, and the docs track them only through an outdatty file list

```
$ grep -n 'pub(crate) const WORDS\|"ready" =>\|"exists" =>' crates/mindtape/src/filter_eval.rs
21:pub(crate) const WORDS: [&str; 17] = [
56:        "ready" => Ok(ctx.readiness.is_ready(ctx.task)),
59:        "exists" => crate::filter_predicates::exists(ctx),
83:        "ready" => Ok(ctx.readiness.is_ready(task)),
86:        "exists" => crate::filter_predicates::exists(ctx),
$ grep -n 'eval::WORDS' crates/mindtape/src/filter.rs
158:        eval::WORDS.contains(&word)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/filter_eval.rs").

== TO-BE

- 🟢 One table in `filter_eval.rs` names each predicate and function with its arity, and `nullary`, `function` and `Filter::names_term` read it, so `mt rm` never takes a new function's name for a stem hash.
- 🟢 `ready`, `blocked`, `terminal` and `exists` are dispatched in one place.
- 🟢 `mt _cli --json` carries the table, and check-doc-facts fails when `www/content/filters.typ` or `skills/mindtape/references/filters.md` misses a name.


== Changes

- `crates/mindtape/src/filter_eval.rs`: replace WORDS and the repeated arms with the table.
- `crates/mindtape/src/filter.rs`: read the table in `names_term`.
- `crates/mindtape-cli/src/cli_dump.rs`: add the table to the dump.
- `www/generate-facts.sh`: write `filter-functions.json` from the dump.
- `.just/scripts/check-doc-facts.sh`: add an arm over the two filter pages.


== Rollback

- Revert the commit; WORDS and the outdatty group carry the names again.


== Out of scope

- Retiring the `filter-docs` outdatty group: it still flags a changed meaning, which the table cannot.
- What each function means: the pages keep that prose.


== Dogfood

The next filter function added in this repo fails `just check` until both filter pages name it.

