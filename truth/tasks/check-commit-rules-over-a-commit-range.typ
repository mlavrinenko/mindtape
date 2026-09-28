#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check commit rules over a commit range",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "gating"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a hook skipped with --no-verify still fails CI],
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
  status: wip(2026, 9, 28),
  links: depends-on("check-commit-messages-against-commit-rules.typ"),
)

== Problem

+ `git commit --no-verify -m 'task: close x'`.
+ 🔴 The commit lands: `mt check --commit-msg` runs only from a hook, and a hook is one flag away from skipped.


== TO-BE

- 🟢 `mt check --commits <range>` runs every `[[commits.rules]]` rule on each commit in the range, reading the message, the files and the task versions from the commit and its first parent.

```
$ mt check --commits origin/main..HEAD
a1b2c3d task: close x
  flip-with-work: x: start or close a task in a commit that carries its work
error: 1 commit(s) broke commit rules
```

- 🟢 A merge commit is checked against its first parent only.


== Changes

- `crates/mindtape-cli/tests/commit_range_e2e.rs` (red first): a range with one offending commit fails naming it.
- `crates/mindtape/src/commit_msg.rs`: build the commit record from a commit instead of the index.
- `crates/mindtape-cli/src/check.rs`: the `--commits` flag.
- `.github/workflows/ci.yml`: run it over the pushed range.


== Alternatives

- A server-side hook: needs a forge this repo does not run.


== Out of scope

- Rewriting offending history: the check reports, never repairs.


== Dogfood

CI runs it over every push to this repo.

