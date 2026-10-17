#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Stop calling examples/minimal the floor config in the evals README",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "docs"),
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
      sound-codebase: neutral,
      docs-match-code: helps[the eval notes describe the fixture the evals run on],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
)

== Problem

+ Reviewing the skill's evals while splitting cli-docs
+ 🔴 #raw("skills/mindtape/evals/README.md"): The README calls examples/minimal the floor config, but minimal declares its own todo, wip, done and cancelled legend, while the floor's has proposed, accepted and failed too.


```
$ rg -n "floor config" skills/mindtape/evals/README.md
13:| `examples/minimal` | 4-5 | The floor config. Its two absences are the test: no `blocked` in the legend, and no template — so `--set priority=8` is refused by name |
$ rg -n '^value' examples/minimal/.mindtape/config.toml
39:value = "todo"
44:value = "wip"
49:value = "done"
55:value = "cancelled"
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("skills/mindtape/evals/README.md").

== TO-BE

- 🟢 The README's row for examples/minimal says it's the smallest config, with its own four-status legend.


== Changes

- `skills/mindtape/evals/README.md`: rewrite the examples/minimal row.


== Rollback

- Revert the README line.


== Out of scope

- The evals themselves, whose prompts already name the right legend.


== Dogfood

Not applicable: the README only guides whoever runs the evals.

