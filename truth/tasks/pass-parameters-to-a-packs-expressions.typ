#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pass parameters to a pack's expressions",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "gating", "decision"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project changes a pack's threshold in one `with` key and keeps the rule's later fixes],
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

+ Adopt a pack whose `judgment-stays-small` asserts `estimate.src-files <= 5`, in a project that wants 8.
+ 🔴 The only way is to copy the whole rule under its `name` and edit the number, so its `when` and `message` drift from the pack's: across nine projects the copies differ in a `kind != "milestone"` clause as well as the cap.
+ 🔴 A path pack declaring `[params]` is refused: `extends: pack '…/team.toml' may not declare: 'params'`.


== TO-BE

```toml
# the pack
[params]
max-files = 5

[[tasks.rules]]
name = "judgment-stays-small"
assert = 'estimate.src-files <= param("max-files")'

# the project
extends = [{ pack = "mindtape:budget", with = { max-files = 8 } }]
```

- 🟢 An `extends` entry is a string or a `{ pack, with }` table. `param("name")` reads the value for the pack that holds the expression, so two packs' names never collide, and a layer's own expressions have no `param()`.
- 🟢 `mt` resolves every `param()` at config load. It refuses an unknown `with` key with a suggestion, a value whose type differs from the default's, and one pack extended twice in a layer.
- 🟢 `mt config show --origin` prints each parameter's value and where it came from.


== Changes

- `truth/tasks/*.typ`: the leaf that builds it, linked with `parent` to `share-config-between-projects-through-packs.typ`. This decision changes no file itself.


== Alternatives

- `{{max-files}}` placeholders in the TOML: a second language beside the filter language, reaching keys that aren't expressions.
- One global `[params]` table in the project: two packs' `max-files` would collide.
- Copy the rule and override it by name, as today: the copies drift.


== Out of scope

- Parameters in a value that isn't an expression, such as a band's bounds: the project overrides that key by name.
- A set-valued parameter: a pack's `[dicts]` entry already lets a project replace the set by name.
- A parameter without a default: every pack stays usable with a bare `extends` line.


== Dogfood

This repo adopts `mindtape:budget` with its own caps instead of copying `judgment-stays-small` and `src-budget`.

