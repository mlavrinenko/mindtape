#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pick one format for error messages and inventory them",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer understands what `mt` does and how to use it on the first read],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 7),
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== Problem

+ Run `mt wip nonexistent-xyz`.
+ 🔴 It prints `error: nonexistent-xyz: no task with id "nonexistent-xyz"` and then `Error: 1 task(s) had problems`.
+ Run `mt ls 'foo('`.
+ 🔴 It prints `Error: failed to parse filter: Found an unmatched opening parenthesis '('.`

== TO-BE

- 🟢 Each error is one line that starts with `error:`, quotes the value that failed, and names the next step when there is one.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- An inventory of every message `mt` prints, grouped by where it is built, and one format for all of them, shown on three real messages.
- A follow-up task for the mechanical rewrite of the rest.

Proposed format, from the inventory in `target/error-inventory.md`:

```
error: <what failed>: '<value>'[: <why>]
hint: <next step>
note: <context, no action>
warning: <same shape; nothing was refused>
```

- `crates/mindtape-cli/src/main.rs`: prints errors itself, so no `Error:` or `Caused by:`; a cause chain joins into one line. Exit 0 ok, 1 refused or failed, 2 bad command line.
- Quoting: user values in single quotes, as clap does; field and rule names bare; at most two hints.
- Breaches keep `error: tasks/x.typ:2: rules.one-wip: <message>`. On a write, `error:` means nothing was written.
- Batches: no summary for one target, else `error: flipped 1 of 3 tasks; 2 failed`.
- A config `message` is the imperative next step; a gate's becomes the hint.
- Typst: `error: file:line:col: <Typst's words>` plus Typst's hints. Only the prefix is coloured.

Rewrite split, at most 5 files each, the first one first: renderer and `main`; refs and suggestions; batches; single writes and check; filter parse; filter functions; config load; validators A; validators B; framework and body; Typst diagnostics; `mt add`; mv, rm, recur; other commands; rule messages.

== Out of scope

- `--help` text: its own task.

== Dogfood

This repo's docs are the change.
