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
  status: done(2026, 10, 26),
  links: parent("share-config-between-projects-through-packs.typ"),
)

== Problem

+ 🔴 #raw("crates/mindtape/src/config/locate.rs"): `place_of` searches only layer files, and `key_line` falls back to the deepest ancestor key a file declares, so a bad value from a pack lands on an unrelated project line:

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

+ 🔴 A pack rule's broken expression lands on the project's rule of the same index, without naming the pack. With the pack's `bad` rule merged first and the project's own rule on line 5:

```
$ mt check
error: .mindtape/config.toml:5: tasks.rules[0].assert: invalid filter: 'nope(...)' can't be a comparison value; only 'now()' can
```


== Environment

Any project, on `mt` at main: extend a path pack that sets a bad `[views.board] filter`, with an unrelated `[views.mine]` in `.mindtape/config.toml`, then run `mt check`.


== TO-BE

- 🟢 The first error above reads `team.toml:2: views.board.filter: …`, at the pack line that sets the value.
- 🟢 The rule error reads `team.toml:<line of its [[tasks.rules]]>: tasks.rules[0].assert: …`, and a rule of the project's own is placed at its own `[[tasks.rules]]` table, whatever its index in the merged array.
- 🟢 A key no source declares in full gets no place, and the error leads with the key alone.


== Changes

One mechanism with `name-the-layer-or-pack-behind-an-unknown-config-key.typ`: `Sources` in `locate.rs` holds each layer's and pack's file name and text, in merge precedence.

- `crates/mindtape/src/config/locate.rs`: `Sources::place_of` searches every source, highest precedence first, and `key_line` returns a line only when the source declares the whole key. A rule-array key (`tasks.rules[2]`, `on.flip[0]`) counts the merged array, so `Sources` maps it through each entry's origin and `[[…]]` header line to its index among that file's own tables first. A rule written inline in its file, with no header, gets no place.
- `crates/mindtape/src/config/packs.rs`, `pack_fold.rs`: the merge keeps each resolved pack's source and fills `Sources` beside the layers.
- `crates/mindtape/src/config/layers.rs`: `build_config` places an error through those sources.
- `crates/mindtape/src/config/layers_tests_source.rs`, `locate_tests.rs`: the red tests first: a pack's bad value, an ancestor layer's, a key no source declares in full, and a pack rule beside the project's own.


== Out of scope

- An unknown key's source: `name-the-layer-or-pack-behind-an-unknown-config-key.typ`.
- A rule entry's place: `locate-a-pack-or-global-layer-rules-breach-at-its-own-config-entry.typ`.


== Dogfood

Once this repo's legend and priority move into packs, a bad value there points into the pack file.

