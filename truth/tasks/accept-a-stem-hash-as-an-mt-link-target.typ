#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept a stem hash as an mt link target",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent links tasks by the handles mt ls prints],
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
  status: proposed(2026, 10, 12),
)

== Problem

+ Linking two tasks I had just filed, by the hashes `mt add` printed.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/link/plan.rs"): `mt link` takes a stem hash for its task but refuses one as a target, though `mt add` and `mt ls` print the hash as the task's handle.

```
$ mt link we8 depends-on sde
error: truth/tasks/sweep-the-jscpd-baseline-into-one-task-per-clone.typ: edge target names no task: 'sde' of kind 'depends-on'
hint: run mt ls to see your tasks
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/link/plan.rs").

== TO-BE

- 🟢 `mt link we8 depends-on sde` links to the task whose hash is `sde`, and writes its path, as a stem target does.
- 🟢 A hash that several tasks share is refused, as it is for the task argument.


== Changes

- `crates/mindtape-cli/src/cmd/link/plan_tests.rs`: the red test, a hash target.
- `crates/mindtape-cli/src/cmd/link/plan.rs`: resolve a target through the same resolver as the task argument.


== Rollback

- Revert the commit. Targets take stems and paths only, as before.


== Out of scope

- `mt link --apply`, whose file maps stems. A hash there is a separate question.


== Dogfood

Agents in this repo link by the hashes `mt add` prints.
