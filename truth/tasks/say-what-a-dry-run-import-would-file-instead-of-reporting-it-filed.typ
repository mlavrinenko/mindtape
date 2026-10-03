#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say what a dry-run import would file instead of reporting it filed",
  kind: "dev",
  priority: ice(
    impact: 4,
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
      cheap-adoption: helps[a reader previewing an import can tell the preview from the run],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Capturing the CLI reference's `mt import` examples for the docs.
+ 🔴 #raw("crates/mindtape-cli/src/import.rs"): `mt import --dry-run` prints the same report as a real import, `3 filed`, while it writes nothing; `mt mv`, `mt rm` and `mt link` say `would` and end with `dry run: nothing written`.

```
$ mt import export.json --dry-run
3 filed, 0 skipped
  Fix the login timeout bug: filed
$ ls tasks | wc -l
5
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/import.rs").

== TO-BE

- 🟢 `mt import --dry-run` reports `would file` per record and ends with `dry run: nothing written`, as `mt mv --dry-run` does.


== Changes

- `crates/mindtape-cli/src/import.rs`: the report wording under `--dry-run`, and the closing line.
- `crates/mindtape-cli/src/import_tests.rs`: assert the dry-run wording.


== Rollback

Revert the commit; the dry run reports `filed` again.

== Out of scope

- The JSON report shape: it carries no wording.


== Dogfood

Not applicable; this repo imports nothing.
