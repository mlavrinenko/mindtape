#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the shipped board from board.toml on the board pages",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: helps[a reader of `mt @name` sees why a project with no open task is missing from the board],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the pages show the file `mt` reads, so a changed default changes them],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "check-the-at-toml-keys-in-the-docs-against-its-schema.typ",
    )[both rewrite the board part of read-many-projects.typ, and the cli-notes entry links the key table that task adds]
    + depends-on(
      "split-the-cli-docs-drift-group-by-the-prose-each-source-feeds.typ",
    )[it drops board.toml from the sets-docs sources the split adds],
)

== Problem

+ Look for the board `mt` ships on `/cli/board/` and `/read-many-projects/`.
+ 🔴 Neither shows it. `crates/mindtape/src/at/board.toml` sorts by `ready:desc` then `open:desc` and hides `open == 0`, and no doc names either.
+ 🔴 The `board` entry in `www/utils/cli-notes.typ` restates the column keys in prose, and no gate ties it to `crates/mindtape/src/at/board.rs`.


== TO-BE

- 🟢 Both pages show `board.toml` as a block cut from the file, so its columns, `sort` and `hide` are the ones `mt` reads.
- 🟢 The `cli-notes.typ` entry says what a `filter` and a `top` column do and links the `at.toml` key table for the keys.
- 🟢 `sets-docs` drops `board.toml`, which the block shows.


== Source of truth

- `crates/mindtape/src/at/board.toml`: the board `mt` ships.


== Changes

- `www/content/read-many-projects.typ`: the red check first: render the block from `shipped-board.json`, which fails the build until the generator writes it.
- `www/generate-facts.sh`: write `shipped-board.json`, the file without its leading comment.
- `www/utils/cli-notes.typ`: the `board` entry renders the block and drops the key list.
- `outdatty.yaml`: drop `board.toml` from `sets-docs`.


== Out of scope

- The `mt board` table layout.
- A command that prints the merged board. The page shows the shipped one, and an `at.toml` override reads as written.


== Dogfood

This repo's docs build renders both pages, so a change to `board.toml` reaches them in the same commit.

