#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the self-describing goal and rate the backlog",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[every task is judged against the no-documentation bar],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the goal names the CLI as the source a newcomer reads],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: wip(2026, 11, 7),
  links: depends-on(
    "cut-the-skill-to-an-index-of-what-mt-cannot-say.typ",
    "name-the-next-command-in-every-mt-dead-end.typ",
    "record-the-no-documentation-arm-in-the-skill-evals.typ",
  ),
)

== AS-IS

+ Open `truth/goals.typ` and run `mt explain rules`.
+ 🔴 Six product goals say what Mindtape is for, and none says a newcomer should need no external documentation. The acid test of `implementable-tasks` names "An agent", though the property holds for any implementer.

```
$ rg -n 'acid-test' truth/goals.typ
9:  acid-test: [Anyone opening an open task file can say within 30 seconds what
14:  acid-test: [An agent in a fresh session, given only the task file, lands the
19:  acid-test: [`mt add` refuses a task that breaks the project's rules, and
```


== TO-BE

- 🟢 `self-describing` is a product goal. Its acid test reads "A newcomer in a fresh session, given a project and `mt` but no external documentation, can list, file and flip a task, and every refusal names the command that fixes it."
- 🟢 `implementable-tasks` says "An implementer", not "An agent".
- 🟢 Every open task and every template rates the new goal, and `mt check` is green.
- 🟢 A task filed later sees the goal in `mt explain` and in its `x-goals` template.

```
$ mt explain
...
goals: readable-tasks, implementable-tasks, enforced-practice, cheap-adoption,
  typst-over-markdown, plain-files, self-describing
```


== Changes

- `truth/goals.typ`: add `self-describing` and reword the `implementable-tasks` acid test for any implementer.
- `truth/tasks/*.typ`: add the neutral verdict to every open task.
- `.mindtape/templates/*.typ`: add the goal to each `product-neutral` default.
- `skills/mindtape/evals/README.md`: name the baseline the goal depends on.


== Out of scope

- The skill cut and the CLI hints, which the first two tasks own.
- The eval blocks, which `record-the-no-documentation-arm-in-the-skill-evals` owns.


== Dogfood

This repo's five open tasks and fifteen templates rate the new goal, and `mt check` proves it.

