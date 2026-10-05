#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a commit whose Refs task is not started and an integrate that leaves one wip",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "mt"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a commit cannot name a task whose flip it forgot],
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
  status: done(2026, 10, 5),
)

== Problem

+ Commit work with `Refs: some-task` while the task is still `proposed` in the staged tree.
+ 🔴 The commit lands. `flip-with-work` in `crates/mindtape/src/packs/mindtape-tasks.toml` fires only on a commit that moves a task into `wip` or `done`, so the missing flip is never seen.
+ Run `just integrate <branch>` for a branch whose last commit names a task that is still `wip` in its tip.
+ 🔴 It fast-forwards `main` with the task open, and the `done` flip is lost. Two agents this week committed work against a `proposed` task, or lost the `done` flip and rewrote commits.


== TO-BE

- 🟢 A `[[commits.rules]]` entry in `.mindtape/config.toml` refuses a commit naming a task that is neither started nor finished in the staged tree. A decision task closing in a task-only commit passes, since its staged status is `done`.
- 🟢 If the existing task-scope language cannot read that, it gains the predicate it needs, a field or state wrapper, and no Rust code names a status.
- 🟢 `.just/scripts/integrate.sh` refuses a branch for a task its commits name in `Refs:` that is still `wip` at the branch tip, and says which one.


== Changes

- `crates/mindtape/src/config_commits_tests.rs`: the red test first, a commit naming a `proposed` task refused and a `done` or `wip` one passing.
- `.mindtape/config.toml`: the `[[commits.rules]]` entry.
- `.just/scripts/integrate.sh`: before the fast-forward, list the `Refs:` stems of the branch's commits and refuse one still `wip` at the tip.
- `docs/src/contributing/commit-messages.typ`: say what the commit rule and the integrate check refuse.


== Alternatives

- A shell check in `.githooks/commit-msg`: a second parse of the trailer and the task file next to the commit-rule language.
- Ship the rule in the `mindtape-tasks` pack: the statuses that count as started are a project's vocabulary.
- Refuse only in `just integrate`: the fix is a rewrite of commits by then, which is the cost this removes.


== Out of scope

- Requiring `done` at integrate: a branch may stop at `wip` for a task that spans branches, and `just integrate` only refuses the ones it names as left open.


== Dogfood

This repo's commit hook and `just integrate` enforce both.
