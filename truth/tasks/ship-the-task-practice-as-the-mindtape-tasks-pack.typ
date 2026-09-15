#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Ship the task practice as the mindtape-tasks pack",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (files: 4),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 15),
  links: depends-on("let-a-config-extend-rule-packs.typ")
    + depends-on("move-goal-verdicts-into-x-goals.typ"),
  x-goals: (
    product: (
      readable-tasks: helps[every adopting repo's tasks share one skimmable shape],
      implementable-tasks: helps[every adopting repo's tasks carry reproduction steps and named files],
      enforced-practice: neutral,
      cheap-adoption: helps[a repo gets the flagship task practice from one line, and later fixes with the binary],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== Problem

+ Run `mt init` in a new repo, then `mt add` a task with an empty body.
+ 🔴 Nothing refuses it. The six-section rules, the word caps and the imperative-title rule live only in this repo's `.mindtape/config.toml`, a block another repo can only copy and then watch drift.


== TO-BE

- 🟢 One line gives a repo this repo's task practice:

```toml
extends = ["mindtape-tasks"]
```

- 🟢 This repo's `.mindtape/config.toml` extends the pack instead of carrying those rules, and `mt check` still passes on the backlog.


== Changes

- `crates/mindtape/src/packs/mindtape-tasks.toml` (new): the section, content and title rules moved out of `.mindtape/config.toml`, each with a `name`, plus the dictionary the title rule reads.
- `crates/mindtape/src/config_packs.rs`: embed and register the pack.
- `crates/mindtape/src/config_packs_tests.rs`: one task that passes and one that fails under the pack alone.
- `.mindtape/config.toml`: `extends = ["mindtape-tasks"]` in place of the moved rules.

== Alternatives

- Document the rules for other repos to copy: that is the drift the Problem shows.
- Put them in the embedded floor for every project: it forces this practice on repos that never asked for it.


== Out of scope

- A goals rule: goals and the function that checks them belong to each repo, as `move-goal-verdicts-into-x-goals.typ` sets up here.
- This repo's own policy, such as the `agent` and `max` assignees and the close gate.


== Dogfood

Built in: this repo becomes the pack's first consumer.

