#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check templates against an empty working tree",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 1),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a budget rule warns about real work, not about a template],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 29),
)

== Problem

+ Making `mt` serve its embedded prelude, which let `mt check` evaluate the built-in template in a test fixture
+ 🔴 #raw("crates/mindtape-cli/src/cmd/check/templates.rs"): `mt check` runs a git-state rule such as `changed(...)` against a template's sample task, so a working-tree budget fires on a template that is no one's work


In `estimate_rules_e2e`'s fixture, whose first status is `wip`, with five changed files against a five-file budget:
```
ok: 1 task
warning: built-in template task:3: rules.src-budget: split the remaining work into a new task
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/check/templates.rs").

== TO-BE

- 🟢 A template's sample task meets an empty change set, so a git-state rule never fires on it, and every other rule still does.


== Changes

- `crates/mindtape-cli/tests/it/estimate_rules_e2e.rs`: the red test first, no `src-budget` warning on the built-in template.
- `crates/mindtape-cli/src/cmd/check/templates.rs`: evaluate a template's rules against an empty change set.


== Rollback

- Revert the commit.


== Out of scope

- Rules that read other task fields, which a template's sample should keep meeting.


== Dogfood

This repo's `src-budget` rule stays quiet on its templates however large a working tree grows.

