#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the comments of 12 files in check-doc-prose.pending, batch 2 of 4",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 3,
  ),
  difficulty: 1,
  estimate: (src-files: 12),
  tags: ("found", "tooling"),
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
      docs-match-code: helps[the comments that explain each gate are held to the same prose rules as the docs],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Landing the check-doc-prose scan of every shell script, YAML file and Justfile comment
+ 🔴 #raw(".just/scripts/check-doc-prose.pending"): 12 files, lines 13 to 24 of the pending list, still tell history or shout in all-caps words

```
$ sed -n '13,24p' .just/scripts/check-doc-prose.pending
.just/scripts/check-secrets.sh
.just/scripts/cover.sh
.just/scripts/crap-baseline-check.sh
...
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/check-doc-prose.pending").

== TO-BE

- 🟢 Each file's comments state the rule and its reason in present tense with no all-caps emphasis, and the file leaves the pending list in its own commit.
- 🟢 The pending list is empty and the gate has no PENDING arm to carry.

== Changes

- `.just/scripts/check-doc-prose.pending`: drop each file once its comments pass.
- Each listed file: rewrite its full-line comments plainly, per docs/contributing/documentation.md. One commit per file.

== Rollback

- Revert the rewrite commit and restore the file's line in the pending list.

== Out of scope

- Rust doc comments and clap help text.
- Code lines: only full-line comments change.

== Dogfood

Every gate script in this repo explains itself in present tense, and `just check` keeps it so.
