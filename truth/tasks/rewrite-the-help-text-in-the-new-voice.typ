#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the help text in the new voice",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 25),
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
  status: proposed(2026, 10, 7),
  links: depends-on("give-the-docs-a-conversational-voice.typ"),
)

== Problem

+ Run `mt ls --help`.
+ 🔴 `--skip` reads: "A truncated table's footer and `--json`'s `truncated.next_skip` already name the `--skip` that continues from this run, advanced past what it showed; pass that value rather than adding to this one by hand".
+ 🔴 `-w` prints `` `MindTape` `` with backticks, a leftover from a clippy lint.

== TO-BE

- 🟢 Every flag's help says what it does, from the user's side, in one or two sentences, per Voice in `docs/contributing/documentation.md`.

== Source of truth

- `docs/contributing/documentation.md`, Voice and Plain prose.

== Changes

- The clap doc comments under `crates/mindtape-cli/src/`, and any test or capture that matches their text.

== Out of scope

- Error messages: their own task.

== Dogfood

This repo's docs are the change.
