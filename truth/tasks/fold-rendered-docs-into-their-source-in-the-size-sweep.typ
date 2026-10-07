#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fold rendered docs into their source in the size sweep",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("size", "tooling"),
  estimate: (src-files: 4),
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
      sound-codebase: helps[one rule says how a near-cap prose file is fixed, and the sweep files one task per artifact],
      docs-match-code: helps[the sweep's remedy and `.linecop.yaml` link to the rule instead of contradicting it],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    7,
  )[one finding per doc; the prose remedy links to the rewrite rule],
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 It would file `size-AGENTS-md` and `size-docs-src-agents-typ`, and `size-skills-mindtape-SKILL-md` and `size-docs-src-skill-typ`: two tasks for each rendered doc, one for the render and one for its source.
+ 🔴 Its Markdown remedy says compressing prose in place is wrong and cites "the 300-line overrides", which `.linecop.yaml` no longer has. `.linecop.yaml` says an always-loaded file is rewritten, never re-capped, and "Rewrite the whole file" in `docs/contributing/documentation.md` says the same. Three places state the rule, and one contradicts the other two.


== TO-BE

- 🟢 The size arm files one finding per source. A rendered file, which names its source in its `Generated from` header, joins that source's finding, and the Problem lists each measured file.
- 🟢 "Rewrite the whole file" in `docs/contributing/documentation.md` is the one statement of how a near-cap prose file is fixed. The sweep's prose remedy and `.linecop.yaml` link to it.


== Changes

- `.just/scripts/size-fixture-check.sh`: red first, a rendered file and its source scored as one finding.
- `contrib/ask-refactoring/size.sh`: map a rendered file to its source and group by source; the prose remedy links to the rule.
- `.mindtape/templates/sweep-size.typ`: the measurement bullet becomes a slot that holds one line per measured file.
- `.linecop.yaml`: the always-loaded overrides link to the rule instead of restating it.


== Rollback

- Revert the commit.


== Out of scope

- The Rust and shell size findings. Each waits until an edit hits its cap.
- The mutants arm.


== Dogfood

This repo's next sweep files one task per doc and points it at the rule.
