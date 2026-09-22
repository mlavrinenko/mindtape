#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite skill eval 4 to grade mt set instead of a hand edit",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (files: 1),
  tags: ("found", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the graded recovery goes through the validator instead of around it],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the eval stops asserting a fact that stopped being true],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 22),
)

== Problem

+ Adding \`mt set\` per the task that put this eval's regrade out of scope.
+ 🔴 #raw("skills/mindtape/evals/evals.json"): Eval 4's expected output states that priority has no CLI writer, and its rubric grades a hand edit as the correct recovery; \`mt set \<task> priority=8\` is now that writer.

`skills/mindtape/evals/evals.json:108` says "hand-edit the new file to add `priority: 8` (the sanctioned path — priority has no CLI writer and no slot here)". `skills/mindtape/evals/evals.json:121` grades "added by hand edit after filing". `mt set --help` now lists `priority` among the writable fields.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("skills/mindtape/evals/evals.json").

== TO-BE

- 🟢 Eval 4 still exercises the refusal path: `--set priority=8` against a one-slot template is still refused loudly.
- 🟢 It then grades `mt set <stem> priority=8` as the recovery, not a hand edit.
- 🟢 No expected output in the suite claims a field has no CLI writer when it has one.


== Changes

- `skills/mindtape/evals/evals.json` — eval 4's `expected_output`, and the rubric criterion naming the hand edit.


== Rollback

Revert the file. The eval is data, and nothing imports it.


== Out of scope

- The refusal half of the eval, which stays as it is. An unknown `--set` key on a one-slot template is still the trap being tested.
- Every other eval. None of them grades a field write.


== Dogfood

The suite is the fixture. Re-running eval 4 grades a transcript that names `mt set`.

