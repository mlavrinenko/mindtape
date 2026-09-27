#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a touches filter over the files a task's commits changed",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "cli"),
  estimate: (files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[before editing a file, an agent reads which tasks shaped it],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[the link from code to task is derived from git, never hand-kept in the task],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: depends-on("check-commit-trailers-against-live-tasks.typ"),
)

== Problem

+ An agent is about to edit `crates/mindtape/src/ready.rs`.
+ 🔴 No command answers which tasks shaped that file; the only route is `git log` on it and reading each `Refs:` trailer by hand.


== TO-BE

- 🟢 `touches("<path>")` matches tasks named by the `[commits] trailer` of any commit that changed the path:

```
$ mt ls 'touches("crates/mindtape/src/ready.rs")' ''
#  TITLE                                  STATUS
—  Derive ready from the links graph      ✅ done
```

- 🟢 A value resolves by its stem, written as a stem or a legacy path.
- 🟢 A stem renamed by `mt mv` resolves to the current task through git renames under the task roots (`git log -M --diff-filter=R`).
- 🟢 A stem retired by `mt rm` matches nothing; `--json` lists the retired stems met, for git archaeology.
- 🟢 Outside a git repository `touches()` fails naming the reason; every other command keeps working.


== Changes

- `crates/mindtape/src/commit_refs.rs` (new, tests first): trailers and renames from `git log`, cached by `HEAD`.
- `crates/mindtape/src/filter_predicates.rs`: `touches()`.
- `crates/mindtape/src/canonical.rs`: the retired stems in `--json`.
- `www/content/filters.typ`: document it.


== Alternatives

- A hand-kept `modifiedFiles` field as in Backlog.md: drifts from what was committed.
- A `formerly:` field written by `mt mv`: `mt rm` deletes tasks, so resolution needs git history anyway.


== Out of scope

- Line-level attribution: file level first.


== Dogfood

Agents here run `touches()` on a file before editing it.

