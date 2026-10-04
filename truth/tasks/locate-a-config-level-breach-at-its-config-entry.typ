#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Locate a config-level breach at its config entry",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a skipped rule points at the rule to fix, not at a random task],
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
  status: done(2026, 10, 4),
  links: related(
    "locate-a-refused-writes-breach-on-the-file-on-disk-through-one-task-call-span-service.typ",
  )[both change what a breach Location can name],
)

== Problem

+ Running \`mt check\` outside git on a project with a changed() rule
+ 🔴 #raw("crates/mindtape/src/config_rules_eval.rs"): A mandatory Location pins problems that are not about a task line onto an arbitrary or invented one.

Four tasks, run outside a git repository:
```
$ mt check
warning: tasks/a.typ:2: rules.budget: skipped: changed() cannot read git: not a git repository (or any of the parent directories): .git
ok: 4 task(s) valid
```
`validate_links.rs:205` invents `{task_id}.typ` for a cycle, and `check_templates.rs:24` uses `<built-in>/task.typ`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/violation.rs").

== TO-BE

- 🟢 A rule skipped for lack of git reports at its entry in `.mindtape/config.toml`, by line.
- 🟢 No breach names a path that does not exist.


== Changes

- `crates/mindtape/src/violation.rs`: let a location name a task field or a config entry.
- `crates/mindtape/src/config_rules_eval.rs`: report a skipped rule at its config line.
- `crates/mindtape/src/validate_links.rs`: drop the invented cycle path.
- `crates/mindtape-cli/src/check_templates.rs`: name a built-in template without a fake path.


== Rollback

- Revert the commit; config-level breaches point at a task again.


== Out of scope

- Breach line numbers inside task files, filed on their own.


== Dogfood

A rule this repo breaks in its config opens at that rule.
