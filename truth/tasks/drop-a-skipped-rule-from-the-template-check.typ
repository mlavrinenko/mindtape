#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop a skipped rule from the template check",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
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
      sound-codebase: helps[a config-level breach is reported once, at the config entry],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Locating a pack rule's skipped warning at its own config entry.
+ 🔴 #raw("crates/mindtape/src/candidate.rs"): a rule skipped for lack of git is reported at its config entry and again for the built-in template

```
warning: pack.toml:1: rules.src-budget: skipped: changed() cannot read git: not a git repository
warning: built-in template task:1: rules.src-budget: skipped: changed() cannot read git: not a git repository
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/violation.rs").

== TO-BE

- 🟢 `mt check` in a project outside git prints one `skipped` warning per rule, at the rule's config entry.
- 🟢 The skipped report has its own `ViolationKind`, and `reads_set_breach` drops it from the template check.


== Changes

- `crates/mindtape-cli/tests/skipped_rule_once_e2e.rs`: the red test first, a pack `changed()` rule printing one `skipped` warning.
- `crates/mindtape/src/violation.rs`: a `SkippedRule` kind.
- `crates/mindtape/src/config_rules_eval.rs`: set it where the skip is reported.
- `crates/mindtape/src/candidate.rs`: drop it in `reads_set_breach`.


== Rollback

- Revert the commit.


== Out of scope

- Other breaches of the config, which no check repeats today.


== Dogfood

This repo's template check reports a skipped rule once.
