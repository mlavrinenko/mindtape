#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the slug# column in the --hyperlinks help and offer it only where it applies",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 7),
  tags: ("found", "cli"),
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
      docs-match-code: helps[the help and the schema name the column the table prints, on the commands that print it],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
)

== Problem

+ Auditing the user docs for restated facts after the `mt @name` milestone
+ 🔴 #raw("crates/mindtape-cli/src/cli/mod.rs"): `--hyperlinks` says it links the `#` and `file` columns, but the column is `slug#`, and the flag is global, so it shows on `mt at` and `mt board`, whose output has neither

`mt board --help` (and `mt at --help`, `mt add --help`) prints:
```text
      --hyperlinks <WHEN>
          When to make the `#` and `file` columns clickable links.
```
while `mt ls` heads its first column `SLUG#`. `schema/config.schema.json:1072`, `:1385` and `:1393` say "the `#` and `file` cells", from the doc comments at `crates/mindtape/src/config/ls.rs:78`, `:82` and `:122`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cli/mod.rs").

== TO-BE

- 🟢 The help and the three schema descriptions name the `slug#` and `file` columns.
- 🟢 `--hyperlinks` is declared on `ls`, `find` and `flip`, whose output prints a task table, and `mt at --help` and `mt board --help` don't list it.


== Changes

- `crates/mindtape-cli/tests/it/hyperlinks_e2e.rs`: the red tests first: `mt ls --help` names `slug#`, and `mt board --hyperlinks always` is a usage error.
- `crates/mindtape-cli/src/cli/mod.rs`: drop the global flag, and declare it once as a flattened argument.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`, `crates/mindtape-cli/src/cmd/find.rs`, `crates/mindtape-cli/src/cmd/flip/mod.rs`: flatten it in.
- `crates/mindtape-cli/src/main.rs`: hand the command's flag to `util::init_hyperlinks_flag`.
- `crates/mindtape/src/config/ls.rs`: name `slug#` in the three doc comments.
- `schema/config.schema.json`: regenerate.


== Rollback

- Revert the commit. The config key and its values stay as they are.


== Out of scope

- Stating the default, `never`, once. The help, the schema and `www/content/cli/ls.typ` each say it, because each reader needs it where they read.


== Dogfood

This repo names tasks by the `SLUG#` hash `mt ls` prints, so the help names the column its owner reads all day.

