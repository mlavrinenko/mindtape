#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Import each record into the folder its file names",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project with subfolders exports and imports back whole],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the round-trip page's claim holds for nested projects],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 5),
)

== Problem

+ Reviewing the import commits 851468df to 8b99e87b
+ 🔴 #raw("crates/mindtape-cli/src/import.rs"): \`mt import\` writes every record into one folder and ignores its \`file\`, so a cross-folder link breaks; the stem check sees that folder only; the report hides a \`-2\` rename.

```
3 filed, 1 skipped
  ...
  hb3: skipped
    - tasks/hb3.typ:6: depends-on: link target `tasks/a/dup.typ` not found
Error: 1 record(s) skipped
```
An id taken in another folder is skipped as `duplicate id`; one taken in the destination lands as `alpha-2.typ`, reported `alpha: filed`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/import.rs").

== TO-BE

- 🟢 With neither `--into` nor `--to`, a record lands at its `file` when that lies under a configured root.
- 🟢 A stem taken anywhere in the project gets the `-2` suffix, not a duplicate-id skip.
- 🟢 The report names the stem written: `alpha: filed as alpha-2`.


== Changes

- `crates/mindtape-cli/tests/import_e2e.rs`: red test first: a two-folder export imports back whole.
- `crates/mindtape-cli/src/import.rs`: place by `file`, seed taken stems from every loaded task, report the written stem.
- `crates/mindtape-cli/src/import/write.rs`: take a destination per record.
- `www/content/import-export.typ`: say where records land.


== Rollback

- Revert the commit; every record lands in one destination again.


== Out of scope

- Rewriting links to follow a `-2` rename, which the round-trip page documents.


== Dogfood

Not applicable; this repo files tasks with `mt add`, not import.
