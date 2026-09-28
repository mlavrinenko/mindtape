#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "split or re-cap crates/mindtape-cli/src/flip.rs",
  kind: "refactor",
  priority: ice(
    impact: 10,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 5,
  estimate: (src-files: 4),
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
    28,
  )[moved the four pre-write guards into flip\_guards.rs; flip.rs 500->411, linecop clear],
)

== Problem

+ Run `linecop --format json`.
+ 🔴 #raw("crates/mindtape-cli/src/flip.rs") stands at 500 of its 500-line budget: 0 lines of headroom left, 100% of the cap spent.

Production code: linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose.

`just ask-refactoring` filed this, and the sweep re-reads the signal every run,
so the task is worth only as much as that measurement still is.

== TO-BE

- 🟢 `just check-file-size` passes and `linecop --baseline 90` no longer names
  this file.

== Changes

- Pick the remedy, then do it. Either carve the file along a seam that means
  something, or raise its cap in `.linecop.yaml` with an override that names a
  new sane number and says why. Both are correct answers. A split for its own
  sake is not: the cap is there to push back on scope growth, not on accuracy.
- If you split, the extracted module has to stand on its own and be named for
  what it does. An arbitrary bottom half moved into a `_more.rs` clears linecop
  and misses the point entirely.
- If you re-cap, write the reason next to the pattern in `.linecop.yaml`, in the
  style its `SKILL.md` / `AGENTS.md` / `CONTRIBUTING.md` entries already use. A
  written reason is the only thing that suppresses this finding on the next
  sweep; there is no ignore list anywhere else, deliberately.

== Out of scope

- Behaviour changes: a split moves code and changes nothing it does.

== Dogfood

Not applicable: a sweep finding changes no feature this repo could use.
