#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Read every task reference, link targets included, through one resolver",
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
      sound-codebase: helps[one resolver reads every task reference, so the readers cannot drift],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
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

- 🟢 `mt link we8 depends-on sde` links to the task whose hash is `sde` and writes its path, in an `--apply` map too.
- 🟢 A hash that several tasks share is refused, as it is for the task argument.
- 🟢 Every `<task>` and link target resolves through `mindtape::resolve_ref`, from the working directory or the linking file. `rm` and `link` keep no rule of their own.
- 🟢 A stem two folders share names the one beside the base, else is refused naming each path.
- 🟢 One `From<RefError> for Diag` words every refusal.


== Changes

- `crates/mindtape-cli/src/cmd/link/plan_tests.rs`, `crates/mindtape-cli/tests/it/stem_hash_e2e.rs`, `crates/mindtape-cli/src/util_tests.rs`: the red tests, a hash target, an ambiguous hash target, a stem two folders share.
- `crates/mindtape/src/model/reference.rs`: `resolve_ref` with its `Base` and `Via`, moved out of `handle.rs`, which keeps the hashes.
- `crates/mindtape-cli/src/util.rs`: `Refs`, which hashes once and reads an argument or a target.
- `crates/mindtape-cli/src/cmd/link/plan.rs`, `cmd/rm/select.rs`, `cmd/flip/mod.rs`: read through `Refs`.
- `crates/mindtape-cli/src/diag/from_lib.rs`: the one refusal per `RefError`.


== Rollback

- Revert the commit. Targets take stems and paths only, and `rm` checks stems on its own, as before.


== Out of scope

- `mt check`'s reading of a written edge, which takes a path only.
- `mt retire`'s link retargeting, which writes `Path::display`, so `\` separators on Windows.


== Dogfood

Agents in this repo link by the hashes `mt add` prints.
