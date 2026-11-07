#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record the no-documentation arm in the skill evals",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tests", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[the no-documentation arm proves a project starts without a skill],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the eval records which facts the CLI must carry itself],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: wip(2026, 11, 7),
  links: depends-on(
    "cut-the-skill-to-an-index-of-what-mt-cannot-say.typ",
    "name-the-next-command-in-every-mt-dead-end.typ",
  ),
)

== AS-IS

+ Read `skills/mindtape/evals/README.md` and `skills/mindtape/evals/evals.json`.
+ 🔴 The README says each eval runs "with and without the skill", but the JSON holds one expected output and one assertion set, so the without arm is a comparison nobody grades.

```
$ jq '.evals[0] | keys' skills/mindtape/evals/evals.json
[
  "assertions",
  "expected_output",
  "files",
  "fixture",
  "id",
  "name",
  "prompt"
]
```

+ 🔴 No recorded run says which evals pass with no skill, or which CLI fact each one needs.


== TO-BE

- 🟢 Each eval has a `without` block with its own assertions, or says why the arm is not meaningful.
- 🟢 `README.md` records one baseline run and names the CLI fact each failure needs.
- 🟢 A small check in `just check` validates the baseline shape for every eval.
- 🟢 A release runs both arms; a no-docs regression is visible before the goal is rated.

```
$ jq -r '.evals[] | [.id, (.without.assertions | length)] | @tsv' skills/mindtape/evals/evals.json
0	7
1	4
...
```


== Changes

- `skills/mindtape/evals/evals.json`: a `without` block per eval with the assertions that must pass with no skill or external docs.
- `skills/mindtape/evals/README.md`: how to run both arms, the baseline result, and which CLI fact each no-docs failure names.
- `.just/scripts/skill-evals-check.sh`: validate that every eval has the block and that the fixture paths exist.
- `Justfile`: add `skill-evals-check` to the check arm.


== Alternatives

- Reuse the same assertions for both arms: measures the skill twice and hides what the CLI lacks.
- Run the loop in CI: the executor and grader need a model, and the six-eval double run is too slow for every commit.


== Out of scope

- Changing the eval prompts or the fixture projects.
- Building an agent runner; the loop stays skill-creator's.


== Dogfood

This repo runs the baseline before rating `self-describing`, and the shape check runs in `just check`.

