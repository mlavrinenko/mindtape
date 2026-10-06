#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print the mt install note when mt import skips a record for the missing prelude",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a new user importing a backlog reads the fix in the error],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Fixing the first-run errors, whose prelude note reaches mt add only
+ 🔴 #raw("crates/mindtape-cli/src/import/write.rs"): \`mt import\` on a machine where \`mt install\` never ran skips every record with the Typst path and never names \`mt install\`

```
$ mt init --no-input && mt import issues.json   # mt install never ran
  Fix login: skipped
    [defaulted] status: no status supplied; defaulted to `proposed`
    - did not load: eval error: file not found (searched at <data>/typst/packages/local/mindtape/0.2.0/typst.toml)
Error: 1 record(s) skipped
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/import/write.rs").

== TO-BE

- 🟢 `mt import` and `mt recur` refusing a record whose task cannot find `@local/mindtape` print the `mt install` note once, as `mt add` and `mt ls` do.

== Changes

- `crates/mindtape-cli/src/import/write.rs`: print `install_hint::failure_prelude_hint` once after the skipped records.
- `crates/mindtape-cli/src/pre_write.rs`: the same once in `refuse_rejected`.

== Rollback

- Revert the two files.

== Out of scope

- `mt add`, which prints the note already.

== Dogfood

Not applicable: this repo's dev shell runs `mt install`, so its contributors never meet the missing prelude.
