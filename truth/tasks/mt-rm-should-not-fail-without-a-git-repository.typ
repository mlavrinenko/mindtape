#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm with no git needs a configurable strategy for referenced tasks",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("cli", "tests"),
  estimate: (files: 4),
  links: related(
    "mt-rm-must-not-require-git-for-a-task-nothing-references.typ",
  )[sibling fix: gate git on need, not presence],
  status: proposed(2026, 8, 3),
)

== Summary

`mt rm` retires a task and rewrites every inbound `links:` edge onto a git-pinned
SWHID tombstone before deleting the file. When the task tree is not a git
repository that rewiring is impossible, and today `mt rm 1f` fails outright at
`git::toplevel(&root)?` (`crates/mindtape-cli/src/rm.rs:110`) before anything is
planned:

```
Error: git: Could not find a git repository in '<tasks-root>' or in any of its parents
```

A hard-refusal is the wrong response, but neither is a bare delete that silently
leaves survivors pointing at a vanished file. When a surviving task still links
to the one being removed, there is still a decision to make, so the gitless path
must be consumer-configurable — and the `gix::discover` error must not leak the
caller's absolute path.

== Scope

Add a `[rm] strategy` key (values `"git-tombstone" | "retire" | "unlink" |
"reject"`): when a git repo is detected it defaults to `git-tombstone` (today's
behaviour); when there is no git repo it defaults to `reject` — the consumer
must opt into another strategy. Per-strategy behaviour:

- `reject` — fail with an explanation listing every *surviving* task that links
  to the removed task(s), so the consumer can edit those edges first;
- `unlink` — strip each inbound edge from the surviving tasks, then delete;
- `retire` — leave the file in place but flip it to a terminal status, so
  inbound edges keep resolving (a tombstone without git). Configurable via a
  `[retire]` section:
  - `[retire] status = "done"` — the terminal status to flip to; if unset, use
    the legend's first terminal status; if the legend has *no* terminal status,
    error.
  - `[retire] rename = ".archive"` — optionally move the task file into a
    sibling folder; if omitted, the file stays.

Also:

- Batch awareness: a task removed in the same `mt rm` run is not "waiting for
  attention". The strategy may act only on references from *surviving* tasks
  outside the batch; a batch that removes every referrer (e.g. deleting all
  tasks) must not reject or trim its own edges.
- Redact the repository path in the gix no-repo error: surface a generic
  placeholder, never the caller's absolute task-root path, in `mt rm` output.
- When no surviving task references any removed task (see the sibling task), no
  strategy is required — plain deletion, no git at all.
- `--dry-run` must report the chosen strategy and its planned edits without a
  git repo.
- Tests (unit `src/rm_tests.rs` + e2e `tests/rm_e2e.rs`): one per strategy (with
  `retire` covering both defaults — first-terminal-status and `"done"` — plus
  the no-terminal-status error and the rename move), and a batch case where a
  removed task's only referrer is also being removed.
