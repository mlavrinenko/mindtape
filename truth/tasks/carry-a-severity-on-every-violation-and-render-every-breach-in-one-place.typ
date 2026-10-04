#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry a severity on every Violation and render every breach in one place",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 5),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a breach's label always matches whether it fails the command],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[five call sites stop choosing the severity label on their own],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 4),
)

== Problem

+ Reviewing commit fb6e260d with \`\[tasks.comments\] policy = "warn"\` in a scratch project
+ 🔴 #raw("crates/mindtape-cli/src/check.rs"): A comment under the warn policy prints as error while mt check exits 0, because the caller decides blocking and an unset severity renders as error.

```
$ mt check; echo "exit=$?"
error: tasks/a.typ:5: comments: line comment: "please reword"
ok: 1 task(s) valid
exit=0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/violation.rs").

== TO-BE

- 🟢 A warn-policy comment prints `warning:` and a forbid-policy comment `error:`.
- 🟢 Every `Violation` carries its severity, and one renderer prints `severity: file:line: field: message` for `mt check`, refused writes and flips.
- 🟢 The `Violation::severity` doc matches what prints.


== Changes

- `crates/mindtape/src/violation.rs`: make `severity` non-optional; fix the doc that says MindTape's own checks print bare.
- `crates/mindtape/src/config_comments.rs`: set the severity from the policy.
- `crates/mindtape-cli/src/check.rs`: move `breach_at` into one renderer and treat comments like every other breach.
- `crates/mindtape-cli/src/pre_write.rs`: keep `Violation`s in `Verdict` and drop `qualify`.
- `crates/mindtape-cli/src/task_edit.rs`: print through the renderer.


== Rollback

- Revert the commit; each command labels breaches on its own again.


== Out of scope

- Breach line numbers, filed on their own.
- Commit-rule breach text, which has no task line.


== Dogfood

Every `mt check` and refused write here prints one label per outcome.
