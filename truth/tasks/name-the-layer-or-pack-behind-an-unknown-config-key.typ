#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the layer or pack behind an unknown config key",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a typo in a shared pack names the pack, so adopting one costs no hunt],
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

+ Widening packs to any config key, in `merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ`
+ 🔴 #raw("crates/mindtape/src/config/layers.rs"): a misspelled key in a pack or an ancestor `.mindtape/defaults.toml` fails the merged config without naming the file that holds it, so the reader looks in `.mindtape/config.toml`

```
$ cat ../.mindtape/defaults.toml
[projet]
x=1
$ mt check
error: can't parse config: unknown field `projet`, expected one of `project`, `tasks`, `ls`, `rm`, `retire`, `templates`, `add`, `go`, `typst`, `format`, `views`, `dicts`, `commits`, `on`, `actors`, `roles`, `extends`, `disable`
```

A pack (`extends = ["./team.toml"]` with `[projet]` in `team.toml`) prints the same line. Before packs took any key, the allow-list named the pack.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config/layers.rs").

== Environment

Any project, on `mt` at main: put `[projet]` in `../.mindtape/defaults.toml` or in a pack the project extends, then run `mt check`.


== TO-BE

- 🟢 An unknown or wrong-typed key fails naming the layer file or pack that set it, such as `unknown field 'projet' in pack /work/app/./team.toml`.
- 🟢 A key that only the merge makes wrong, with no single source to blame, keeps today's message.


== Changes

- `crates/mindtape/src/config/layers.rs`: when the typed deserialization of the merged document fails, retry each layer and pack table alone and name the first that fails on the same field.
- `crates/mindtape/src/config/layers_tests.rs`: the red tests first, one for a pack and one for an ancestor layer.


== Out of scope

- Line numbers for the key: `locate-a-pack-or-global-layer-rules-breach-at-its-own-config-entry.typ` covers locating rule entries.


== Dogfood

Not applicable: this repo's layers carry no typo. A contributor's typo in an official pack would name the pack.

