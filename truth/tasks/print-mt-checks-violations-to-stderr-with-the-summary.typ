#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print mt check's violations to stderr with the summary",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent that reads only stderr sees which rule failed, not just how many],
      enforced-practice: neutral,
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
  status: cancelled(
    2026,
    10,
    8,
  )[Duplicate of report-check-problems-on-stderr-in-the-error-format, which moves every mt check line to stderr and adds the count.],
)

== Problem

+ Capturing every error \`mt\` prints for the error inventory
+ 🔴 #raw("crates/mindtape-cli/src/cmd/check/mod.rs"): \`mt check\` prints each violation to stdout and only the count to stderr, so stderr alone says a check failed without saying why

```
$ mt check 2>/dev/null
error: tasks/add-redis-client.typ:2: rules.imperative-title: start the title with an imperative verb
$ mt check >/dev/null
Error: 1 violation(s), 0 load error(s)
$ mt check --commit-msg /tmp/msg 2>/dev/null
error: /tmp/msg: commits.rules.work-names-a-task: name the task this commit works on: `Refs: <stem>`
$ mt check --commit-msg /tmp/msg >/dev/null
Error: 1 commit message problem(s)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/check/mod.rs").

== TO-BE

- 🟢 Every `error:` and `warning:` line `mt check` prints goes to stderr, next to its summary.
- 🟢 stdout carries only the `ok:` line, so `mt check >/dev/null` still shows what failed.


== Changes

- `crates/mindtape-cli/src/cmd/check/mod.rs`: print breach lines with `eprintln!`.
- `crates/mindtape-cli/src/cmd/check/commit.rs`: the same for commit-message problems.
- `crates/mindtape-cli/src/cmd/check/range.rs`: the same for `--commits`.


== Rollback

- Revert the three files; violations go back to stdout.


== Out of scope

- The wording and prefix of the lines, which the error-format task decides.


== Dogfood

The pre-commit hook's `mt check` failures show their reasons even where the hook drops stdout.
