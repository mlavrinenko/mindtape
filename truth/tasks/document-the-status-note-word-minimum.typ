#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document the status note word minimum",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 4),
  kind: "docs",
  assignee: "agent",
  status: cancelled(
    2026,
    9,
    15,
  )[Backlog review 2026-09-15: no close note ever failed a reader, and an agent pads ten words for free.],
  links: depends-on("require-a-minimum-word-count-on-a-status-note.typ"),
)

== Summary

`require-a-minimum-word-count-on-a-status-note.typ` adds `note-min-words` to the status legend with unit tests only. Nothing drives it through the real binary, the config page does not mention it, and this repo's own closing notes are still unpoliced.

== Scope

Add an end-to-end case beside the flip tests in `crates/mindtape-cli/tests/`: under `note-min-words = 10` on `done`, `mt flip done -m done` is refused and writes nothing, and a ten-word note lands. Document the key beside `note` in `www/content/config.typ`, including that `mt check` never applies it, and add a CHANGELOG.md entry under Unreleased.

Then set it on this repo's closing statuses in `.mindtape/config.toml`, the same key on `done`, `failed` and `cancelled`:

```toml
[[tasks.status]]
value = "done"
note-min-words = 10
```

== Dogfood

Built in: from this task on, every close in this repo carries a note of at least ten words.
