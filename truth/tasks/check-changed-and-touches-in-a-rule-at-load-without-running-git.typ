#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check changed() and touches() in a rule at load without running git",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
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
      sound-codebase: helps[a load-time check that reads git is a side effect of loading a config],
      docs-match-code: helps[/filters/ and /estimates/ can say a rule reading git is skipped outside a repository],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Running examples/estimates outside git.
+ 🔴 #raw("crates/mindtape/src/filter.rs"): \`expr\_problem\_in\_set\` dry-runs a rule with \`dry: false\`, so \`changed()\` and \`touches()\` read git at every config load.

+ Copy `examples/estimates` and `lib/` to a directory outside any git repository.
+ Run `mt check` in the copy.
+ 🔴 It prints `tasks.rules[3].assert: changed() cannot read git: not a git repository`, so the rule is never skipped.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/filter.rs").

== TO-BE

- 🟢 Loading a config checks the states and globs of `changed()` and the argument of `touches()` without running git, as `expr_problem` already does.
- 🟢 Outside a git repository, a rule reading `changed()` reports once as skipped, as `config_rules_eval.rs` intends, and `mt check` loads the config.


== Changes

- `crates/mindtape/src/filter.rs`: run `expr_problem_in_set` with `dry: true`, keeping the one-task set so `count()` still passes.
- `crates/mindtape/src/filter_tests.rs`: a rule with `changed()` loads outside a git repository.


== Rollback

Revert the commit; loading a config runs the rule's git reads again.

== Out of scope

- `changed()` and `touches()` in a filter typed on the command line, which should keep failing with git's reason.


== Dogfood

This repo's src-budget rule stops running `git status` on every config load.
