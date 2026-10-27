#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Decide whether jscpd's clone classes replace the repeats arm with a gate",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.6,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "repeats"),
  estimate: (src-files: 3),
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
      sound-codebase: helps[short repeats stop growing between sweeps instead of waiting for one],
      docs-match-code: neutral,
    ),
  ),
  assignee: "max",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read the `jscpd-clone-classes` blocker that `.config/canibump.toml` carried before the commit that dropped it.
+ 🔴 Its plan was to drop the repeats arm of `just ask-refactoring` and gate short repeats in `just lint dry` once jscpd shipped clone classes. The fork qahq pins ships them now, and nothing tracks the plan.
+ Run the repeats arm on its own: `jscpd --no-tips -f rust -k 15 -l 1 -r json -o target/rep crates`, then count `classes` with 300 tokens of gain.
+ 🔴 It finds 26 snippets, such as a create-parent-then-write pair pasted 92 times, so a gate at that bar fails today's tree.


== TO-BE

- 🟢 The repeats arm either stays as the advisory sweep it is, with the reason written in `contrib/ask-refactoring/repeats.sh`, or gives way to a gate on classes by copies and size with a baseline of its own.


== Changes

- `contrib/ask-refactoring/repeats.sh`: the reason it stays, or its removal.
- `.just/lint.just`: a short-repeats gate, if the decision is to gate.
- `.mmz/config.yaml`: the gate's scope, if one is added.


== Rollback

- Revert the commit; the sweep arm keeps reporting.


== Out of scope

- The dry gate at 50 tokens, which already gates new clones.


== Dogfood

This repo's own short repeats are what the decision is about.
