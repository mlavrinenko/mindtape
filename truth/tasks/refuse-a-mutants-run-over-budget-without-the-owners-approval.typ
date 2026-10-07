#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a mutants run over budget without the owner's approval",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling",),
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
      sound-codebase: helps[a mutation run never eats hours of the machine by accident],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    7,
  )[both recipes count first and refuse over 100 without approval],
)

== Problem

+ An agent ran `just mutants-diff` on a branch with a wide diff.
+ 🔴 It started a sweep that would have run for hours, and only the owner killing it stopped it.

== TO-BE

- 🟢 `just mutants` and `just mutants-diff` count the mutants first and refuse a run over 100 unless `MINDTAPE_MUTANTS_APPROVED=1` is set.
- 🟢 The refusal says how many mutants, how long, and how to narrow the run.
- 🟢 CONTRIBUTING says only the owner sets the approval.

== Changes

- `.just/scripts/_mutants-env.sh`: `mutants_budget_check`.
- `.just/scripts/mutants.sh`, `.just/scripts/mutants-diff.sh`: call it before `cargo mutants`.
- `docs/src/contributing.typ`, `docs/src/contributing/mutation-testing.typ`: the rule.

== Rollback

- Revert the three scripts and the two doc sources.

== Out of scope

- The 13 s estimate per mutant. It is a measured average, not a gate.

== Dogfood

Every mutation run here goes through the check.
