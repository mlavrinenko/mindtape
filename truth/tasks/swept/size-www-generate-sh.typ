#import "@local/mindtape:0.2.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "split or re-cap www/generate.sh",
  kind: "refactor",
  priority: ice(
    impact: 10,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 4,
  estimate: (files: 3),
  assignee: "agent",
  tags: ("swept", "size"),
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
      sound-codebase: helps[a file under its cap changes without reading its
        neighbours],
      docs-match-code: neutral,
    ),
  ),
  status: done(
    2026,
    9,
    23,
  )[split the release-workflow derivation into www/generate-release-targets.sh; generate.sh 300->245; baseline 90 clear],
)

== Problem

+ Run `linecop --format json`.
+ 🔴 #raw("www/generate.sh") stands at 300 of its 300-line budget: 0 lines of headroom left, 100% of the cap spent.

linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose.

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just check-file-size` passes and `linecop --baseline 90` no longer names
  this file.

== Changes

- Pick the remedy, then do it. Either move a self-contained part of this file
  somewhere it stands on its own, or raise its cap in `.linecop.yaml` with an
  override that names a new sane number and says why. Both are correct answers.
  A split for its own sake is not: the cap is there to push back on scope
  growth, not on accuracy.
- If you split, what comes out has to be nameable for what it does. A cut made
  where the line count happens to land clears linecop and misses the point.
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Behaviour changes: a split moves code and changes nothing it does.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
