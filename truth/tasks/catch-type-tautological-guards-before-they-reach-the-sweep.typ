#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Catch type-tautological guards before they reach the sweep",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling", "tests"),
  estimate: (src-files: 2),
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
      sound-codebase: helps[unkillable source forms are named instead of re-triaged],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: cancelled(
    2026,
    10,
    1,
  )[No source lint can see a mutant-vs-source equivalence, as the task itself says; the source-form review rule folds into record-which-survivor-classes-a-project-lint-could-have-prevented.],
)

== Problem

+ Read `.cargo/mutants.toml`'s `exclude_re` entry for `replace > with >= in resolve_and_select`.
+ 🔴 The sweep reached a `usize` compared `>= 0` and hand-excluded it as provably equivalent: no test can distinguish it, so the mutant is a doomed triage cycle. The same shape produced the `parse_batch` `<`→`<=` exclusion.
+ Clippy's correctness lints do not see a mutated comparison, and no source lint can: the equivalence is between the mutant and the source, not inside the source.

== TO-BE

- 🟢 The source forms whose mutants are unkillable on unsigned or bounded types are known, written down, and preferred against in review (e.g. `x != 0` over `x > 0` where `x` is a count, so the mutant flips killably).
- 🟢 Whichever check can be automated (a lint for literal tautologies in source, an ast-grep pattern run over `crates/**/*.rs`) runs where cheap; the rest is recorded as a review rule beside the mutation-testing deep dive.

== Changes

- Start from the four hand-excluded equivalents in `.cargo/mutants.toml`; classify each as type-tautology, pure memoisation, platform-specific or no-public-constructor.
- `docs/src/contributing/mutation-testing.typ` (with regenerated `.md`): a short subsection naming the type-tautology class and the source forms that avoid it.
- If a syntactic rule is feasible, add it as `sgconfig.yml` + a rule file and a `just` recipe; if not, record why the class stays manual.

== Rollback

Drop the doc subsection and any ast-grep config; the exclusions stay as they are today.

== Out of scope

- Forbidding `match`, `&&` or `||` to shrink the mutant count: metric-gaming, explicitly rejected on 2026-09-30.
- A general static-analysis pipeline: this is only the equivalence residue clippy cannot see.

== Dogfood

A deliberate `usize >= 0` in a scratch branch is caught by the new check, or the review rule names it before the next sweep does.
