#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print the new task's slug hash when mt add writes it",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: helps[the short handle to type next is on screen the moment the task exists],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
)

== Problem

+ Filing tasks from the pre-release docs review, then referring to one as ksr
+ 🔴 #raw("crates/mindtape-cli/src/cmd/add/"): mt add reports only the absolute path it wrote, so the slug hash that every other command accepts is unknown until a separate mt ls

```
$ mt add -t finding -f - < 9.yaml
wrote <repo>/truth/tasks/stop-the-crap-ratchet-flagging-untouched-same-named-functions.typ
$ mt eval ksr
ksr    proposed  low       Stop the CRAP ratchet flagging untouched same-named functions
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/add/").

== TO-BE

- 🟢 `mt add` reports the slug hash with the path it wrote, in the form the project's other commands use for a task reference, and the path is relative to the working directory like `mt ls`'s `file` column.
- 🟢 `mt add --format json` (and `mt import`, if it reports each written task) carries the slug hash too.


== Changes

- `crates/mindtape-cli/src/cmd/add/`: compute the slug hash of the written stem and include it in the report and the JSON.
- `crates/mindtape-cli/tests/`: an e2e test asserting the hash `mt add` prints resolves with `mt eval <hash>`.
- `CHANGELOG.md`: an `[Unreleased]` entry; update any capture or doc that shows `wrote …`.


== Rollback

- Revert the commit.


== Out of scope

- Changing how the hash is computed.


== Dogfood

Every task filed here can be named by its hash straight away.

