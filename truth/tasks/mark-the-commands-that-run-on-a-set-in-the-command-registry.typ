#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Mark the commands that run on a set in the command registry",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli", "docs"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent reads which commands take a set from one registry the binary enforces],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page, the skill and each command page render the set-capable commands from `cli.json`],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "show-the-json-of-a-set-and-of-the-board-on-the-schema-page.typ",
    )[both rewrite read-many-projects.typ, so they land in turn],
)

== Problem

+ Look up which commands take `mt @work` on `/read-many-projects/` and in `skills/mindtape/SKILL.md`.
+ 🔴 The page types "Only `ls`, `find`, `check` and `board` run on a set", and `docs/src/skill.typ` types the same four.
+ 🔴 No registry holds that list. It is whichever commands call `project_set::held_set()`: `cmd/ls/mod.rs:96`, `cmd/find.rs:40`, `cmd/check/mod.rs:58` and `cmd/board.rs:62`. A fifth one changes no doc and fails no gate.


== TO-BE

- 🟢 `CLASSIFICATION` in `crates/mindtape-cli/src/cli/command_class.rs` marks the set-capable read commands, and `main.rs` refuses a set for every other command from that mark, before it runs.
- 🟢 `mt _cli --json` carries the mark, each command page says whether the command runs on a set, and the page and the skill render the list from `cli.json`.


== Source of truth

- `crates/mindtape-cli/src/cli/command_class.rs`: the mark.
- `crates/mindtape-cli/src/project_set.rs`: the refusal `runs on one project`.


== Changes

- `crates/mindtape-cli/tests/it/at_e2e.rs`: the red test first: `mt @set <cmd>` for every unmarked read command prints the one-project refusal, and a marked one doesn't.
- `crates/mindtape-cli/src/cli/command_class.rs`: the mark and a lookup.
- `crates/mindtape-cli/src/main.rs`: refuse a held set from the mark.
- `crates/mindtape-cli/src/cli/tree_json.rs`: export the mark per command.
- `www/utils/cli-ref.typ`: say on a command page that it runs on a set.
- `www/content/read-many-projects.typ`, `docs/src/skill.typ`: render the list from `cli.json`.


== Out of scope

- Running more commands on a set. That's a feature question, and this task only names the ones that do.


== Dogfood

The owner runs `mt @home` over this repo and the others, and a command this repo adds to sets lists itself on the page through the mark.

