#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`flip` command + per-status shortcut verbs (`done t1 t2`)",
  status: proposed(2026, 6, 27),
  priority: ice(impact: 7, confidence: 0.8, ease: 6),
  tags: ("cli",),
)

== Summary

Mutate a task's status from the CLI without opening the file. Canonical form
`mindtape flip <status> <task>...`, plus auto-generated per-status shortcut verbs
so `mindtape done 1b 2c` == `mindtape flip done 1b 2c`. Stamps the status dated
today, like `add` does for the initial status.

== Argument order (decided)

Status-first, not task-first. The originally-proposed `flip 1b done` breaks under
multiple targets — `flip 1b 2c done` can't tell tasks from status. Status-first
keeps the batch form unambiguous and makes the shortcut a trivial pre-bind:
`done <tasks>...` is `flip` with the status fixed.

== Scope

- New subcommand `flip <status> <task>...`. Each `<task>` resolves via the
  existing `resolve_task` (handle `1b` / slug / path) in
  `crates/mindtape-cli/src/util.rs`.
- Per-status shortcut verbs generated from the configured status legend
  (`[[tasks.status]]`), not a hardcoded list. clap can't derive dynamic
  subcommands; use an `external_subcommand` fallback — unknown verb, look it up
  in `Legend`, dispatch to flip; otherwise error. Builtin subcommands win; warn
  when a legend value shadows one (e.g. a status named `add`/`rm`).
- Status value formatting: reuse the shortcut-vs-`status("name", y, m, d)` logic
  that `add.rs` already has (`STATUS_SHORTCUTS`, `inject_header`). Extract a
  shared helper into the `mindtape` lib so the copy-paste gate (jscpd ≥70 tokens)
  stays green and the rule lives in one place. Edit in place via
  `task_edit::upsert_named_arg` (same path `review` uses).
- Optional note: `-m/--note` (the status trailing `[note]` block). Not a trailing
  positional — ambiguous against the variadic task list.
- Re-validate every touched task after the edit and report violations. Flipping
  to a terminal status relaxes required-unless-terminal; flipping away from
  terminal can newly require fields. Reuse the `check` validation path.
- Resolve all targets before writing any file — handles are view-relative and a
  status change reorders the default view; resolve-before-mutate avoids drift
  mid-batch.

== Open questions

- No-op / re-stamp: `done 1b` when `1b` is already `done` — re-stamp today's date,
  or skip? Lean re-stamp (records the transition moment) but make it explicit.
- Batch atomicity on validation failure: stop at first invalid, or apply the
  valid ones and report the rest? Lean apply-valid + report, matching `rm`'s
  tolerant batch behaviour.
- Shortcut verb discoverability: dynamically-generated verbs won't show in
  `--help`. Surface them in a dedicated help section or `flip --list`.

== Relation

Sibling to a planned `go`/`edit` command (open a task in the configured editor by
handle). Both lean on `resolve_task`; `go` additionally needs a general editor
setting with `{file}` substitution. Kept separate — `flip` needs no editor.
