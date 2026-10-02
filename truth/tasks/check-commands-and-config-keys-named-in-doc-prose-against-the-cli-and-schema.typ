#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check commands and config keys named in doc prose against the CLI and schema",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "tooling"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a renamed flag or config key fails the gate wherever prose still names it],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    3,
  )[just check-doc-spans checks mt commands, flags and config keys in inline doc spans; fixed \[\[tasks.roots\]\] on config.typ and the mt export span],
  links: parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read `www/content/config.typ:120`.
+ 🔴 It names `[tasks.roots]`, which is TOML table syntax. `roots` is an array of tables, written `[[tasks.roots]]`, and line 79 of the same page says so.
+ Inline code in prose names subcommands, flags and config keys. `check-doc-facts` checks value-taking flags, and nothing checks the other names. A rename in `crates/mindtape-cli` or `crates/mindtape/src/config*.rs` leaves the prose wrong and the gate green.


== TO-BE

- 🟢 `just check-doc-spans` reads inline code spans in doc sources. A span starting `mt <word>` must name a real subcommand, and any `--flag` in it must exist on that subcommand (`mt _cli --json`). A span shaped like a config section or key path must exist in `schema/config.schema.json`, with `[[...]]` for arrays of tables.
- 🟢 A failure names file, line and the closest valid name.
- 🟢 Its header and gate note say which problem it stops.


== Changes

- `.just/scripts/check-doc-spans.sh`, new. Run it on today's tree first; it must name `config.typ:120`.
- `Justfile`: the recipe, wired into `check`.
- `.mmz/config.yaml`, `outdatty.yaml`: scope and `dev-docs` entry.
- `www/utils/gate-notes.typ`: one clause.
- Doc sources with hits: fix the names.


== Rollback

Revert the commit; drop the recipe from `check`.


== Out of scope

- Filter function names: `mt` has no machine-readable list of them yet; a follow-up adds one.
- Spans inside fenced blocks: the block gate covers those.


== Dogfood

`just check` runs it on every commit, so a CLI or config rename here fails until the docs follow.

