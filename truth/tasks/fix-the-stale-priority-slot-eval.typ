#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fix the stale priority-slot eval",
  kind: "fix",
  status: proposed(2026, 9, 19),
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tests", "docs"),
  estimate: (files: 2),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[an eval asserting behavior the binary refuses grades a correct agent as failing],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the skill's test suite states what the binary does],
      docs-match-code: neutral,
    ),
  ),
)

== Problem

1. 🔴 `skills/mindtape/evals/evals.json` eval 4,
  `add-with-priority-slot-trap`, expects `--set priority=8` on the built-in
  fallback template to be "dropped silently — exit 0, success reported, no
  priority written". The binary now refuses it by name, so an agent that
  recovers the right way fails assertions written for the old behavior.

== Environment

`cd examples/minimal && MINDTAPE_DATA_HOME= ../../target/debug/mt add "Upgrade
the Postgres driver" --set priority=8 --print` → exit 1, `Error: unknown
--set/-f key(s): priority; known slots: title`, at `cfd1338`.

== TO-BE

- 🟢 The eval expects the loud refusal: a non-zero exit naming the unknown key
  and the known slots. It grades the recovery: drop the flag, file the task,
  get `priority: 8` onto it (a hand edit is the sanctioned path), run `mt check`.
- 🟢 The assertions test that recovery rather than the silent drop.

== Changes

- `skills/mindtape/evals/evals.json` — eval 4's `expected_output` and
  assertions, against the message in `crates/mindtape-cli/src/add_values.rs`.

== Out of scope

- The other evals and the eval runner.

== Dogfood

The eval suite is how the skill is graded; running it against the fixed eval
proves the expectations match the binary.
