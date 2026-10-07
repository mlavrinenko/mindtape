#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Carry the load failure's line into a refused write",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a refused add or batch write names the line of the source that did not load],
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
  status: done(2026, 10, 7),
)

== Problem

+ Running `mt add` or a batch write with a source that fails to evaluate.
+ 🔴 #raw("crates/mindtape-cli/src/pre_write.rs"): the refusal prints the message with no line, though the load error has one.

```
error: parser: did not load: unexpected argument: titel
```
`crates/mindtape/src/loader.rs:30`: `LoadError.line` is `Some` for an eval error.
`crates/mindtape-cli/src/pre_write.rs:301-307`: `verdict` keeps `error.message` only.
`pre_write.rs:270`, `:107-109` and `task_edit.rs:213-214` print it line-less.
`crates/mindtape/src/violation.rs:50`: `Location::Filled { line }` exists for a source not yet on disk.

== TO-BE

- 🟢 A load failure in a refused write prints `<location>: did not load: <message>`, with the file and line when the file is on disk and `the task as filled:<line>` when it is not, the way a breach does.

== Changes

- `crates/mindtape-cli/src/pre_write.rs`: keep the line in `Verdict::load_failure` (a message plus an optional `Location`), built in `verdict` with the same on-disk/`Filled` choice as `pre_write.rs:181-204`.
- `crates/mindtape-cli/src/pre_write.rs:270`, `:107-109` and `crates/mindtape-cli/src/task_edit.rs:213`: print through that location.
- Check `crates/mindtape-cli/src/import/write.rs` and `crates/mindtape-cli/src/link_cmd.rs`, which read `Verdict` too.
- Add a red test first: a `mt add --set` whose source misspells a field must name a line.

== Rollback

- Revert the commit.

== Out of scope

- Load failures with no place at all (`line: None`).

== Dogfood

`mt add` with a broken template, and `mt import` over a bad file.
