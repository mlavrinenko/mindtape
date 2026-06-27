#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "`mindtape ls --interactive` — pick-by-ordinal, open in editor",
  status: wip(2026, 6, 28),
  priority: ice(impact: 6, confidence: 0.7, ease: 5),
  tags: ("cli",),
)

== Summary

`mindtape ls --interactive` (`-i`) renders the same list, then holds the
terminal in a key loop: type the `#`-column ordinal to highlight a task, press
Enter to open it in the configured editor (`go`), and keep working without
re-running `ls`. A picker over the list `ls` already prints — no new query
surface, same tasks, same order.

== Keys (decided)

- `0`–`9` — append a digit to the ordinal buffer. Rows whose 1-based ordinal has
  the buffer as a prefix render reverse-video in the `#` column; the "first
  highlighted" is the lowest such ordinal (topmost row).
- `Backspace` — drop the last digit.
- `Del` (or `Ctrl+Backspace` where the terminal supports it) — clear the buffer.
- `Enter` — `go` on the first highlighted task. No-op when the buffer is empty or
  nothing matches. After the editor exits, redraw the list.
- `Ctrl+Enter` — open every listed task in the editor, in display order, then
  redraw.
- `Space` — refresh: re-open the project (re-eval), recompute the view, clear the
  buffer.
- `q` (or `Ctrl+C`) — leave interactive mode.

A dim footer lists the live hotkeys under the table.

== Input backend (decided)

`crossterm` (new `mindtape-cli` dependency). Rationale over hand-rolled termios:
modifier keys are the deciding factor. In a legacy terminal `Ctrl+Enter` is
byte-identical to `Enter` and `Ctrl+Backspace` collides with `Backspace`; they
separate only under the Kitty keyboard protocol (CSI-u disambiguation). crossterm
negotiates it (`supports_keyboard_enhancement` + `PushKeyboardEnhancementFlags`)
and parses every escape sequence cross-platform with no `unsafe` on our side.
Termios would mean reimplementing escape + Kitty parsing for one feature.

Caveat carried by the protocol, not us: on terminals without the enhancement,
`Ctrl+Enter` degrades to `Enter` (opens the first highlighted, not all) and
`Ctrl+Backspace` is unavailable — `Del` is the reliable clear, always distinct.

== Scope

- `--interactive` / `-i` flag on `ls::LsArgs`. Conflicts with `--json` and
  `--format {json,csv}` (table-only). Requires a TTY on stdin and stdout; errors
  with a clear message when piped.
- Extract the `ls` view computation (filter, select, sort, truncate → `(tasks,
  readiness)`) out of `ls::run_with` into a shared helper so the loop and the
  one-shot path produce identical lists; `Space` re-runs it against a freshly
  opened project.
- New `crates/mindtape-cli/src/ls_interactive.rs`: raw-mode + alternate-screen
  event loop holding the ordinal buffer, redrawing on each key. Split tests into
  a `_tests.rs` sibling if it nears the 500-line linecop limit; keep functions
  ≤70 lines / ≤5 args / cognitive-complexity ≤20.
- Highlight seam in `render::table`: a `render_highlighted` entry taking a
  per-row highlight, post-processing the `#` column's cells to reverse-video
  (`anstyle` `Effects::INVERT`). `render` stays the plain delegate. Highlight
  rides the table's existing painter, so `--color never` drops the affordance
  (documented; consistent with the colour contract).
- Reuse `go`'s editor machinery: factor `resolve_editor` + `spawn_editor`
  (template→argv, `{task}` substitution, spawn-and-wait) into shared helpers in
  `go.rs` that both `go::run` and the loop call — keeps the jscpd ≥70-token gate
  green. The loop suspends the TUI (leave raw mode + alternate screen, show
  cursor) around each spawn so a terminal editor gets a clean screen, then
  re-enters and redraws. "Open all" spawns the editor once per task, sequentially.

== Notes

- The buffer is prefix selection, not a parser: `1` highlights `1`, `10`–`19`,
  `100`…; Enter opens the topmost (`1`). A leading `0` matches no ordinal (none
  start with `0`), so the buffer simply highlights nothing until cleared.
- Resolve-before-open is not needed within one frame: ordinals are the live
  display positions, and the list is static between redraws. `go` re-opens the
  project itself, so the opened file is always the one under the chosen ordinal.
- Sibling to `go-command.typ` and `flip-status-command.typ`; all three pivot on
  the handle/ordinal view that bare `ls` prints.
