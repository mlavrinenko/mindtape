#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Route the held-back allow hint through Diag::in_member",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
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
      sound-codebase: helps[one method names a set member in every hint],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 19),
)

== Problem

+ Naming the member project in a hint's `mt` command inside a set, which added `Diag::in_member`
+ 🔴 #raw("crates/mindtape-cli/src/util.rs"): `held_back_warning` builds a set member's `mt @<name> allow` by hand, so it skips `Diag::in_member` and prints no `note: project: <name>`

```
crates/mindtape-cli/src/util.rs:220:            Command::mt().arg(format!("@{name}")).word("allow"),
crates/mindtape-cli/src/project_set.rs:138:        .chain(held_back_warning(&held, Some(name)))
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/util.rs").

== TO-BE

- 🟢 A set member's held-back warning gets `hint: read them, then run mt @<name> allow` and `note: project: <name>` from `Diag::in_member`, and `held_back_warning` takes no project name.
- 🟢 Outside a set the hint still reads `read them, then run mt allow`.
- 🟢 `mt preset ls`, which reads no project, is built with `Command::mt_global()`.


== Changes

- `crates/mindtape-cli/src/diag/hint.rs`: a hint shape that renders a text and then a `Command` with no colon between them.
- `crates/mindtape-cli/src/util.rs`: build the allow hint with that shape and drop the `project` parameter.
- `crates/mindtape-cli/src/project_set.rs`: pass the member's held-back warning through `Diag::in_member`.
- `crates/mindtape-cli/src/cmd/init/preset.rs`: build `mt preset ls` with `Command::mt_global()`.


== Rollback

- Revert the four files.


== Out of scope

- The flip gate's `read the config's commands, then run mt allow` in `crates/mindtape-cli/src/cmd/flip/hooks.rs`, since a set refuses a flip.


== Dogfood

Not applicable, this repo is one project, not a set.
