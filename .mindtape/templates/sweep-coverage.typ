#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: slot("title", prompt: "cover <path>"),
  kind: "test",
  priority: ice(
    impact: slot(
      "impact",
      kind: "int",
      prompt: "1-10, rises as the covered fraction falls",
    ),
    confidence: slot(
      "confidence",
      kind: "float",
      prompt: "0-1, high: llvm-cov counted the lines, it did not guess",
    ),
    ease: slot(
      "ease",
      kind: "int",
      prompt: "1-10, falls with how many lines are unreached",
    ),
  ),
  difficulty: slot(
    "difficulty",
    kind: "int",
    prompt: "0-9 on the judgment scale",
  ),
  estimate: slot("estimate", kind: "text", prompt: "(src-files: N)"),
  assignee: "agent",
  tags: slot("tags", kind: "list", prompt: "swept, coverage"),
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
      sound-codebase: helps[unreached lines get a test that fails when they
        break],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Run `just cover`.
+ 🔴 #raw(slot("path", prompt: "repo-relative path of the undercovered file")) is #slot("percent", kind: "text", prompt: "covered lines as a percentage")% covered: #slot("uncovered", kind: "text", prompt: "count of unreached lines") of #slot("lines", kind: "text", prompt: "count of instrumented lines") instrumented lines are never reached by the suite, against a floor of #slot("floor", kind: "text", prompt: "the coverage floor this file is measured against")%.

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just cover` reports this file at or above the floor, or the `just cover`
  recipe excludes it with a reason.

== Changes

- Pick the remedy, then do it: test the unreached paths, or exclude the file
  from scoring with a reason next to its entry in the `just cover` recipe, in
  the style its `main.rs` and `recur_rrule.rs` exclusions already use.
- If you test, start with the branches that decide something; `just mutants -f`
  on the same file says whether the new tests actually pin behaviour.
- A written exclusion reason is the only thing that suppresses this finding on
  the next sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Tests that walk the lines without asserting on them: they move the number and
  buy nothing, which is the failure mode a coverage target invites.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
