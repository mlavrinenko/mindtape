#import "@local/mindtape:0.4.0": *
#import "/truth/lib.typ": *

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
    product: (
      readable-tasks: helps[a bare index of names tells an agent what the project checks before it opens a rule],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the pack docs and the index use the same names],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 8),
  links: depends-on("fold-the-project-listings-into-mt-explain.typ"),
)

== AS-IS

+ Run `mt explain rules` in this repo.
+ 🔴 Names such as `goals`, `src-budget` and `section-allowlist` need their message or the pack docs to say what they check.
+ Open the bare `mt explain` index.
+ 🔴 Some names are terse enough that even the group heading does not explain the rule. The reader opens the rule or its message.


== TO-BE

- 🟢 Every rule name in a pack and in the project says what it checks without its message.
- 🟢 The pack docs, `mt explain rules` and the tests that quote a rule use the same name.
- 🟢 A rule keeps its behavior, and only its name, its docs and the tests that assert it change.

```text
tasks.rules.keep-one-task-in-progress
tasks.body.headers.rules.allow-only-declared-sections
commits.rules.name-task-in-refs-trailer
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

- Changing a rule's condition, assertion, message or severity stays out. Only its name moves.
- Renaming collectors, statuses, link kinds or templates stays out. Their names are not rule names.


== Dogfood

The bare `mt explain` index reads as a list of what each rule checks, without opening the rule or its message.

