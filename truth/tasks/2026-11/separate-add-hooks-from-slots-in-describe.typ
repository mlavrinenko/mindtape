#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Separate add hooks from slots in mt add --describe",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 2,
  links: (
    (
      task: "merge-and-explain-on-add-hooks-across-layers-and-packs.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "cli", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the CLI explains what it will run without making the reader guess],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the text output matches the JSON shape it documents],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Reviewing mt add --describe after the add hooks joined the output.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/values.rs"): Hook rows are appended to slot rows with no heading, so a hook named like a slot reads as a slot.

```
$ mt add --describe
title  string  required
ready  ready  set { assignee = "ready-hook" }
```

The second row is a hook, not a slot named `ready`. The JSON output already separates `slots` and `hooks`.


== TO-BE

- 🟢 The text output separates the two blocks the JSON names.
- 🟢 A reader can tell a slot name from a hook name without reading the action text.

```
slots
title  string  required

hooks
ready  ready  set { assignee = "ready-hook" }
```


== Changes

- `crates/mindtape-cli/src/cmd/add/values.rs`: print a `slots` and a `hooks` heading, with a blank line between the blocks.
- `crates/mindtape-cli/src/cmd/add/values_tests.rs`: pin the two-block text output; the e2e describe test checks only JSON.


== Out of scope

- The JSON shape, which already has separate `slots` and `hooks` arrays.


== Dogfood

`mt add --describe` in this repo becomes readable as two lists.

