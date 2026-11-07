#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make every rule name say what it checks",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("mt", "tooling", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        readable-tasks: helps[a bare index of names tells an agent what the project checks before it opens a rule],
      ),
      ..(
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (docs-match-code: helps[the pack docs and the index use the same names]),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 6),
  links: depends-on("fold-the-project-listings-into-mt-explain.typ"),
)

== AS-IS

+ Run `mt explain rules` in this repo.
+ 🔴 Names such as `goals`, `src-budget` and `section-allowlist` need their message or the pack docs to say what they check.
+ Open the bare `mt explain` index.
+ 🔴 Some names are terse enough that even the group heading does not explain the rule; the reader opens the rule or its message.


== TO-BE

- 🟢 Every shipped and project rule name says what it checks without its message.
- 🟢 The pack docs, `mt explain rules` and the tests that quote a rule use the same name.
- 🟢 No rule changes behavior; only its name, its docs and the tests that assert it.

```text
tasks.rules.keep-one-task-in-progress
tasks.body.headers.rules.allow-only-the-declared-sections
commits.rules.name-the-task-in-a-reflog-footer
```


== Changes

- `crates/mindtape/src/packs/tasks.toml`: rename the task and body rules, and the comments that quote them.
- `crates/mindtape/src/packs/budget.toml`: rename the judgment and source-budget rules.
- `crates/mindtape/src/packs/wip-caps.toml`: rename the two work-in-progress caps.
- `crates/mindtape/src/packs/tweaks.toml`: rename the two tweak rules.
- `crates/mindtape/src/packs/dogfood.toml`: rename the Dogfood rules.
- `crates/mindtape/src/packs/commits.toml`: rename the commit rules.
- `crates/mindtape/src/packs/flow.toml`: rename the flip hook labels.
- `.mindtape/config.toml`: rename the `goals` rule, the `gates-fresh` hook and every `disable` entry.
- `crates/mindtape/src/config/packs_tests_budget.rs`, `crates/mindtape-cli/tests/it/rules_e2e.rs` and the other tests that assert a name: update them.
- `www/content/rules.typ`, `www/content/packs.typ`, `www/content/hooks.typ` and the other docs that quote a name: update them.
- `CHANGELOG.md`: an `[Unreleased] Changed` entry naming the renames.


== Out of scope

- Changing a rule's condition, assertion, message or severity; only its name moves.
- Renaming collectors, statuses, link kinds or templates; their names are not rule names.


== Dogfood

The bare `mt explain` index reads as a list of what each rule checks, without opening the rule or its message.

