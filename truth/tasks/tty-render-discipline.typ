#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Central TTY, colour, and width rendering discipline",
  status: done(2026, 6, 19),
  priority: 5,
  estimate: (hours: 3),
  tags: ("cli", "tooling"),
)

== Summary

Centralise terminal-output policy in one mindtape-cli module used by every
command, so no command re-decides colour and width ad hoc and pipes stay clean
(no ANSI, no truncation, machine-stable separators). It owns TTY detection,
`NO_COLOR`, `--color=auto|always|never`, emoji gating with an ASCII fallback,
unicode-width truncation and padding, and the separator policy.

== Scope

- Module exposing: is-tty, colour decision, glyph mode, and width-aware
  pad / truncate helpers.
- Consumed by the list renderer and every human-facing command output.
- Colours accept named ANSI and `#RRGGBB` truecolor (downsampled when the
  terminal lacks truecolor), via anstyle.
