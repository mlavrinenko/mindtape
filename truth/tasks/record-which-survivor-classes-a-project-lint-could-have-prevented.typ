#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record which survivor classes a project lint could have prevented",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
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
      sound-codebase: neutral,
      docs-match-code: helps[the triage taxonomy lives in the doc and in exclude_re],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 1),
)

== Problem

+ Read the `exclude_re` list in `.cargo/mutants.toml` (7 entries today).
+ 🔴 Each entry's reason says why the mutant cannot be killed, but nothing names its class, so the next sweep re-triages the same class by hand and no one can count which classes recur.
+ The 2026-09-30 triage split survivors into test gaps, equivalents and lint-preventable shapes; only the first two left a record.

== TO-BE

- 🟢 Every `exclude_re` entry leads with its class (type-tautology, pure-memoisation, platform-specific, no-public-constructor, same-call, discarded-effect, aliased-input) before its reason; the two `exclude_globs` entries are classed too (entry-point, cfg-unreachable).
- 🟢 `docs/src/contributing/mutation-testing.typ`'s Triage section lists the classes and what each one suggests: a test, an exclusion, a source-form rewrite, or a lint.
- 🟢 The type-tautology class names the source forms that avoid it in review (`x != 0` rather than `x > 0` on an unsigned count, so the comparison mutant stays killable) and says plainly that no source lint can catch it: the equivalence lies between mutant and source.

== Changes

- `.cargo/mutants.toml`: lead each `exclude_re` reason with its class name, and class the `exclude_globs` entries.
- `docs/src/contributing/mutation-testing.typ` (with regenerated `.md`): the class list in Triage, with the type-tautology review rule folded in from the cancelled `catch-type-tautological-guards-before-they-reach-the-sweep`.
- Dropped: teaching `contrib/ask-refactoring/mutants.sh` to derive a class from the mutation operator, so `just ask-refactoring` output does not yet link a survivor to its class.

== Rollback

Revert the comments and the doc subsection; the exclusions still work, only the taxonomy goes.

== Out of scope

- Automating the classification: the record is the goal, not a classifier.
- Re-judging any individual exclusion: each keeps its current verdict.
- Carrying the class into `just ask-refactoring`'s filed tasks.

== Dogfood

The next sweep's triage starts from a class list instead of free prose, and a recurring class is visible by grepping `exclude_re`.
