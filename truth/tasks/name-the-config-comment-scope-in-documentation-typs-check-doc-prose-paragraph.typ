#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the config comment and schema description scopes in documentation.typ",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "docs"),
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
      docs-match-code: helps[the paragraph states what the gate scans],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Extending check-doc-prose.sh to the Justfile and YAML config comments and to the config schema descriptions, with docs/src/contributing/ out of reach in that worktree.
+ 🔴 #raw("docs/src/contributing/documentation.typ"): the paragraph on `just check-doc-prose` does not say the gate also scans the full-line comments of the Justfile, `.just/*.just` and the three YAML configs, and every `description` in `schema/config.schema.json`.

The paragraph lists the history words, the acronym list and the skipped spans, and names no config file.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("docs/src/contributing/documentation.typ").

== TO-BE

- 🟢 The check-doc-prose paragraph names the Justfile, `.just/*.just`, `.mmz/config.yaml`, `outdatty.yaml` and `.linecop.yaml` as scanned for their `#` comment lines, and `schema/config.schema.json` as scanned for its descriptions.

== Changes

- `docs/src/contributing/documentation.typ`: add one sentence naming the config comment and schema description scopes, then `just docs md` to render docs/contributing/documentation.md.

== Rollback

Revert the commit.

== Out of scope

- The script itself: `.just/scripts/check-doc-prose.sh` already scans those files.

== Dogfood

The paragraph is what a contributor reads to learn what the gate scans.
