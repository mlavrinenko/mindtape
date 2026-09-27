#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read estimate units and kebab-case names in filters",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a bound or budget on any unit is one rule expression, not a Rust checker],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[filters name fields the way the task files spell them],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 27),
)

== Problem

+ `mt ls 'estimate.files > 3'`.
+ 🔴 It is refused: `estimate` supports only `has(estimate)`/`miss(estimate)` (`crates/mindtape/src/filter_predicates.rs`), so no rule can compare an estimate, and bounds on it needed the bespoke `[[tasks.estimate.max]]`/`min` checker.
+ `mt ls 'has(x-goals)'`.
+ 🔴 `Error: invalid filter: expected a field, got Sub`: the tokenizer reads `-` as minus, so no filter can name a kebab-case key, although Typst, and every task file, spells names that way (`x-goals`, `readable-tasks`, a unit such as `src-files`).


== TO-BE

- 🟢 Names follow Typst's identifier rule: a `-` between name characters is part of the name. `has(x-goals)` and `estimate.src-files` parse; a leading `-` is still a negative number, and string literals are untouched.
- 🟢 `estimate.<unit>` reads a declared unit as a number, for any unit name; mindtape knows none of them. An absent unit behaves like any absent numeric field, and `has(estimate.src-files)` tests presence.
- 🟢 A unit outside `[tasks.estimate] units` in a saved expression is a config error at load, like an unknown field.


== Changes

- `crates/mindtape/src/filter_predicates_tests.rs` (red first): `has(x-goals)`, `estimate.src-files > 3`, `has(estimate.src-files)`, `priority > -1`, a `-` inside a string literal.
- `crates/mindtape/src/filter.rs`: a pre-lex pass that keeps a kebab-case name one token before `evalexpr` sees it.
- `crates/mindtape/src/filter_predicates.rs`: the dotted `estimate.<unit>` lookup.
- `www/content/filters.typ`: document both.
- `CHANGELOG.md`: the additions.


== Alternatives

- snake_case units (`src_files`): parses today, but splits the vocabulary between task files and filters, and still leaves `x-goals` unreachable.
- A quoted accessor (`estimate["src-files"]`): works without a lexer change, at the cost of a second spelling for the same name.
- Arithmetic (`+ - * /`): dropped. The only rules that needed it were a per-file line budget and summed budgets, and neither catches a real problem here.


== Out of scope

- Aggregates: `truth/tasks/add-a-count-aggregate-to-filters.typ`.
- Retiring `[[tasks.estimate.max]]`/`min`: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

This repo's estimate bounds become `assert = 'estimate.src-files <= 5'` rules in the retirement task.
