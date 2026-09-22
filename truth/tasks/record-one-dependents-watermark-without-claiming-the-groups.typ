#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record one dependent's watermark without claiming the group's",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 3,
  ),
  difficulty: 3,
  estimate: (files: 2),
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
      sound-codebase: helps[a recorded hash means a review of that file and nothing else],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 9, 22),
)

== Problem

+ Two delegated agents in this session each edited one dependent, re-recorded its group, and absorbed unrelated pending watermarks onto their commit.
+ 🔴 #raw("outdatty.yaml"): Re-recording a group to stamp one dependent also stamps every other pending dependent in it, which is the misattribution the manifest's own header warns against.

```
$ outdatty update --help
Options:
      --group <ID>
          Limit to the named group(s). Repeatable
```
No per-file flag exists, and `just outdatty-update GROUP` takes a group id
only (`Justfile:437`). The prio-facts branch absorbed pending hashes for
`cli-docs`, `prelude-docs` and `dev-docs`; the pty-isolate branch hit the
same on `docs/src/contributing.typ`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.yaml").

== TO-BE

- 🟢 Recording a watermark for one dependent leaves every other pending
  dependent in its group pending.
- 🟢 A commit's lock diff names only files its author actually reviewed.
- 🟢 Whatever the answer is, `outdatty.yaml`'s header says it, because that
  header is where the watermark rule is argued today.


== Changes

- `outdatty.yaml` — the header paragraph on what a recorded hash claims,
  corrected to say what the tool can actually scope, and the request it
  needs upstream.
- `Justfile` — `outdatty-update` gains an optional dependent path once
  `outdatty update` can take one; until then, say in the recipe comment that
  the group is the smallest unit and a re-record reviews all of it.


== Rollback

Revert both files. Nothing generated depends on either, and the lock is
unaffected.


== Out of scope

- Reconciling the groups that are currently pending. They are ok as of this
  commit; this is about the next time one is not.
- The `outdatty-update-all` blanket form, which already carries its own
  argument for staying non-default.


== Dogfood

This repo is outdatty's heaviest consumer, with ten groups and a gate that
reads them. The next dependent-only edit here is the test: its lock diff
should name one file.

