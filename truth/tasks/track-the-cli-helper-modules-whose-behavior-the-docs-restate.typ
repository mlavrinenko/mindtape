#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the CLI helper modules whose behavior the docs restate",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
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
      docs-match-code: helps[a change to a guard, a hook runner or the ls rule cascade asks for the prose that restates it],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
)

== Problem

+ Splitting the cli-docs drift group into one group per command source
+ 🔴 #raw("outdatty.yaml"): Each CLI group lists only a command's `*Args` module, so prose on /statuses/, /hooks/, the `mt ls`, `mt allow` and `mt rules` pages, /templates/ and /commit-rules/ restates modules no group lists.


```
$ grep -c "crates/mindtape-cli/src/cmd/flip/guards.rs" outdatty.yaml
0
```
The same count is 0 for `flip/hooks.rs`, `ls/rules.rs`, `allow.rs`, `rules.rs`, `check/templates.rs`, `check/commit.rs` and `check/range.rs`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.yaml").

== TO-BE

- 🟢 Editing `cmd/flip/guards.rs` fires `cli-flip-docs`, and editing `cmd/ls/rules.rs` fires `cli-ls-docs`.
- 🟢 `mt allow` and `mt rules` each have a group pairing the module with its command page and the prose that restates it.


== Changes

- `outdatty.yaml`: add each helper module whose behavior a page restates to its command's group, and add `cli-allow-docs` and `cli-rules-docs`.
- `outdatty.lock`: re-record each changed group after reviewing its dependents.


== Rollback

- Revert `outdatty.yaml` and `outdatty.lock`.


== Out of scope

- Helpers no prose restates, such as `cmd/link/plan.rs`. Listing them would fire a review on internal refactors.


== Dogfood

The next commit in this repo that changes a flip guard fires `cli-flip-docs`, and its author reviews /statuses/.

