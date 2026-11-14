#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Hyperlink the file column with OSC 8",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("cli", "docs"),
  estimate: (files: 13),
  links: depends-on("opt-in-file-column-for-mt-ls.typ"),
  status: done(2026, 7, 28),
)

== Summary

A working-directory-relative path is clickable in an `IDE`-integrated terminal,
which detects paths on its own. In a standalone terminal nothing detects it,
and the way to make a cell clickable is the OSC 8 hyperlink sequence:
`ESC ] 8 ; ; <uri> ST <text> ESC ] 8 ; ; ST`.

Support stays a guess: the protocol lacks a capability query, so the tool cannot
detect the terminal. The feature is opt-in rather than automatic for that
reason.

One trap worth recording, since it costs an hour to rediscover: the target must
be a URI with a scheme. A bare relative path is not one, and every terminal
ignores it, so a hand test with a relative target fails even on a terminal with
full support.

== Decisions

- `--hyperlinks <auto|always|never>` as a global flag plus `[ls] hyperlinks`,
  mirroring the shape and the one-shot resolution `--color` already uses.
- Default is `never`, not `auto` as colour has. Colour degrades to nothing on a
  terminal that ignores it. An unsupported terminal can render an OSC 8 payload
  as visible garbage instead, and nothing tells the tool in advance which
  terminal it faces. A default that can corrupt output is not a safe default.
- `auto` means stdout is a TTY and `TERM` is not `dumb`. It is a heuristic, so
  document it as one.
- The code builds the target from a template so the scheme stays the user's choice:
  `[ls] hyperlink-target` with `{abs}` and `{rel}` tokens, defaulting to
  `file://{abs}`. Anyone whose editor registers its own scheme points it there
  instead.
- Only the `file` column links here. The handle column is
  hyperlink-the-handle-column.
- Escape sequences never reach CSV or JSON: those are data, not display.

== Scope

- A `render/link.rs` holding the sequence builder and the process-global enable
  resolution, alongside `render/color.rs` and following its shape.
- `cli.rs` flag, `main.rs` init, `[ls] hyperlinks` and `[ls] hyperlink-target`
  in `config_ls.rs`, and a regenerated `schema/config.schema.json`.
- Column widths must measure the visible text, never the escape payload. The
  table already computes widths before the painter styles a cell. Confirm the
  hyperlink wrap sits on the same side of that boundary. Pin it with a test that
  renders a linked cell and asserts the layout is byte-identical to the unlinked
  one.
- Tests: sequence shape. Disabled by default. `never` beats a config opt-in.
  CSV and JSON carry no escapes. Width accounting as above.
- This task owns the documentation reconcile for all three pieces: the `file`
  column, the `file` filter and sort field, and hyperlinks. Since `cli.rs` is a
  `cli-docs` outdatty source, its dependents must come along in the same commit: `www/content/cli.typ`, `www/content/filters.typ`,
  `www/content/config.typ`, `www/content/quickstart.typ`,
  `www/utils/cli-notes.typ`, and `skills/mindtape/SKILL.md`. Then
  `just outdatty-update` and commit the refreshed `outdatty.lock`.

== Outcome

Landed as specified. `Column` gained `link_uris`, populated only for the file
column, and `render_grid` wraps the already-width-fitted, already-painted cell,
on the same side of the width boundary as colour. CSV and JSON never construct a
`Column` at all, so they cannot leak an escape structurally. Tests pin it anyway. The config-side and CLI-side `HyperlinkChoice` are separate types
bridged by a `From` `impl`, because `mindtape` must not depend on clap.

One design note worth keeping: `hyperlink_enabled` takes `term_is_dumb` as an
explicit parameter rather than reading `TERM` internally, unlike
`color_enabled`'s `NO_COLOR` read. An agent sandbox runs with `TERM=dumb`, so
an env-reading pure core made the pty test flaky. A pure core that reads the
environment is not pure.

The estimate was wrong, and in a way worth recording rather than editing away:
13 files declared, 26 touched. The overrun is entirely the documentation
reconcile, which covered three features because `cli.rs` is a `cli-docs`
outdatty source and this was the first commit in the sequence to touch it. The
lesson is not that the 15-file ceiling is too tight: it caught a truly
oversized change, which is its job. A doc reconcile forced by a drift group
belongs in its own task, sized on its own. It should not ride along with the
feature that happens to trip the gate.
