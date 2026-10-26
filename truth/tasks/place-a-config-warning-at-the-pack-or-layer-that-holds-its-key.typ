#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Place a config warning at the pack or layer that holds its key",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a warning from a shared pack points into the pack],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ Fixing config errors to name their pack, in `place-a-config-error-only-at-a-source-that-declares-its-full-key.typ`
+ 🔴 #raw("crates/mindtape-cli/src/util.rs"): `config_warnings` places every load warning at the project config, whatever file holds the key

```
$ cat .mindtape/config.toml team.toml
extends = ["./team.toml"]
[tasks.fields.x-machine]
values = ["alpha"]
[[tasks.rules]]
name = "r"
assert = 'x-machnie == "alpha"'
severity = "warn"
$ mt ls
warning: .mindtape/config.toml: tasks.rules[0].assert: `x-machnie` is not declared in [tasks.fields]
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/util.rs").

== TO-BE

- 🟢 The warning above reads `team.toml:5: tasks.rules[0].assert: ...`, at the file and line of the rule that holds it.
- 🟢 The config load keeps the file texts it merged, and the warning pass places a key through them, as an error does.

== Changes

- `crates/mindtape-cli/src/util.rs`: place each warning through the sources the load kept, not `config_path(root)`.
- `crates/mindtape/src/config/locate.rs`: its `Sources` is the mechanism; `Config` has to carry it, as it carries `rule_sites`.

== Rollback

- Revert the commit.

== Out of scope

- Errors. `place-a-config-error-only-at-a-source-that-declares-its-full-key.typ` places those.

== Dogfood

Not applicable: this repo extends no pack with a warning today.
