#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail mt import when it skips a record",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a script that imports can tell a partial import from a whole one],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Running the /import-export/ recipes in the docs generator, which must fail the build when a recipe fails.
+ 🔴 #raw("crates/mindtape-cli/src/import.rs"): `mt import` exits 0 when it skips records, even when it files none, so a script or CI step that imports cannot tell from the exit code that records were lost.

```
$ echo '[{"title":"Fix the login page","status":{"name":"todo","date":"2026-09-01","note":null}}]' | mt import
0 filed, 1 skipped
  Fix the login page: skipped
    - status: `todo` is not a configured status
$ echo $?
0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/import.rs").

== TO-BE

- 🟢 `mt import` exits 1 when it skips any record, after it writes the records that passed and prints the report.


== Changes

- `crates/mindtape-cli/src/import.rs`: return an error naming the skip count after the report.
- `crates/mindtape-cli/src/import_tests.rs`: assert the exit code for a batch with one bad record.


== Rollback

Revert the commit; a skip exits 0 again.

== Out of scope

- Refusing the whole batch on one bad record: the records that pass still land.


== Dogfood

The /import-export/ captures in `www/generate-import-export.sh` can drop their own skip check.
