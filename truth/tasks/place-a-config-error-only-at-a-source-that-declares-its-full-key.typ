#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Place a config error only at a source that declares its full key",
  kind: "fix",
  priority: ice(
    impact: 5,
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
      cheap-adoption: helps[an error in a shared pack points into the pack, not the project],
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

+ Reviewing `merge-every-config-key-from-a-pack-below-the-layer-that-extends-it.typ`, which lets a pack set `[views]`
+ 🔴 #raw("crates/mindtape/src/config/locate.rs"): a bad value a pack or an ancestor `defaults.toml` set is placed at a line of the project config that doesn't hold it: `place_of` searches only layer files, and `key_line` falls back to the deepest ancestor key the file declares

```
$ head .mindtape/config.toml team.toml
==> .mindtape/config.toml <==
extends = ["./team.toml"]

[views.mine]
filter = "has(title)"

==> team.toml <==
[views.board]
filter = "(("
$ mt check
error: .mindtape/config.toml:3: views.board.filter: failed to parse filter: a '(' is never closed
```

Line 3 is `[views.mine]`, matched through the ancestor key `views`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/config/locate.rs").

== Environment

Any project, on `mt` at main: extend a path pack that sets a bad `[views.board] filter`, with an unrelated `[views.mine]` in `.mindtape/config.toml`, then run `mt check`.


== TO-BE

- 🟢 The error above reads `team.toml:2: views.board.filter: …`, at the pack line that sets the value.
- 🟢 A key no source declares in full gets no place, and the error leads with the key alone.


== Changes

- `crates/mindtape/src/config/locate.rs`: place a problem only where a source declares its full key, and search pack texts (`ResolvedPack.text`) beside the layer files, in merge precedence.
- `crates/mindtape/src/config/packs.rs`: keep each resolved pack's path and text for `place_of`.
- `crates/mindtape/src/config/locate_tests.rs`: the red tests first: a pack's bad value, an ancestor layer's, and a key no source declares in full.


== Out of scope

- An unknown key's source: `name-the-layer-or-pack-behind-an-unknown-config-key.typ`.
- A rule entry's place: `locate-a-pack-or-global-layer-rules-breach-at-its-own-config-entry.typ`.


== Dogfood

Once this repo's legend and priority move into packs, a bad value there points into the pack file.

