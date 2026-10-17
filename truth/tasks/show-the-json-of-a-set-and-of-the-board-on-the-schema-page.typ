#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the JSON of a set and of the board on the schema page",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a script reading many projects parses keys the page shows from a real run],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the set and board shapes come from captured runs, so a changed key changes the page],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
  links: parent("keep-the-user-docs-generated-or-gated-not-restated.typ")
    + depends-on(
      "show-the-shipped-board-from-board-toml-on-the-board-pages.typ",
    )[both rewrite read-many-projects.typ, so they land in turn],
)

== Problem

+ Open `/schema/` to parse `mt @work ls --json` or `mt @work board --json` in a script.
+ 🔴 Its sections are Fields, A record, Priority and The listing, all for one project. A set's listing adds `project` and `error` (`crates/mindtape-cli/src/render/output.rs:211`), and `board --json` prints `project`, `path`, `hidden` and `cells` per row (`crates/mindtape-cli/src/cmd/board.rs:239`).
+ 🔴 One prose sentence on `/read-many-projects/` names the set keys, and no capture runs either command.


== TO-BE

- 🟢 `/schema/` has a section on many projects, with captured runs of `mt @work ls --json` and `mt @work board --json`, cut down with `jq`, over `examples/across-projects/`.
- 🟢 The section says what `cells` holds per column kind: a count, a task, `null` or an `error`.
- 🟢 `/read-many-projects/` links that section in place of its sentence on `--json` and csv.


== Source of truth

- `crates/mindtape-cli/src/render/output.rs`: the set listing's added keys.
- `crates/mindtape-cli/src/cmd/board.rs`: `print_json` and `cell_json`.


== Changes

- `www/generate-reference.sh`: the captures first, inside the `/read-many-projects/` block that sets the fixture up as `$HOME`, so the page shows only a shape that ran.
- `www/content/schema.typ`: the section, rendering the captures.
- `www/content/read-many-projects.typ`: the link.


== Out of scope

- `mt check --json`, whose shape no page shows for one project either. It gets a task of its own once a script needs it.


== Dogfood

This repo's docs build runs the captures, so a key renamed in `board.rs` changes `/schema/` in the same commit.

