#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Accent overdue tasks in `ls`",
  status: done(2026, 6, 27),
  priority: ice(impact: 6, confidence: 0.9, ease: 7),
  tags: ("cli",),
)

== Summary

`mindtape ls` prints the DUE column as plain text, so a task whose due date
has already passed reads no differently from one due next month. Overdue,
non-terminal tasks should stand out: colour the DUE cell red so a glance at the
list surfaces what has slipped.

== Scope

- In `crates/mindtape-cli/src/render/table.rs`, replace the `plain(...)` due
  cell in `build_rows` with a `due_cell(task, config)` helper that returns a
  `CellSpec`.
- Overdue test: the due date parses (`mindtape::date::parse_ymd`, or plain ISO
  string compare against `mindtape::date::today_iso()` — `YYYY-MM-DD` sorts
  lexically) and is strictly before today, AND the task is non-terminal
  (`task.is_terminal(&config.tasks.status.terminal())` is false). When both
  hold, set `color: parse_color("red")`; otherwise no colour (plain).
- A shipped task is never overdue: terminal status (done/cancelled/failed)
  suppresses the accent even with a past due date.
- Empty due (`None`) renders blank as today, no colour.
- Colour activates only on a TTY via the existing `Painter` path — piped output
  stays plain, same as every other coloured cell.

== Notes

- Accent colour is fixed red for now (matches the `failed`/`high` red already in
  the legend); a configurable accent can come later if asked.
- Test the `due_cell` helper directly (it returns a `CellSpec`): the painter is
  off in the non-TTY test harness, so assert on `spec.color`, not the painted
  string. Cover three cases: overdue non-terminal → `Some(red)`; overdue but
  terminal → `None`; future due → `None`.
- No new crates; `mindtape::date` already exposes `today_iso`/`parse_ymd`.
