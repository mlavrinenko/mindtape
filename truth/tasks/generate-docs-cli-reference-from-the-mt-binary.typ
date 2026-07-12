#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Generate docs CLI reference from the mt binary",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  tags: ("docs",),
  links: depends-on(
      "pinnable-clock-for-reproducible-output.typ",
      "learn-piano-example-tracker.typ",
    )
    + related("render-task-files-as-typeset-documents.typ")[
      the typeset-card face is that task's job; both build on the same
      typeset-render face
    ],
  status: wip(2026, 7, 12),
)

== Summary

Render the docs CLI reference, help text, version, and sample output from the
real `mt` binary at build time, so the docs cannot drift. The `examples/`
fixture (learn-piano) supplies the data; a pinned clock (pinnable-clock) makes
the output reproducible. This is the v1 delivery — plain text, verbatim help.

== Approach

- Build-time only; no committed snapshots. Regeneration from the live binary is
  no-drift by construction, so there is nothing to gate. Output to
  `www/generated/` (sibling of `content/`, so tola never scans it as pages),
  git-ignored.
- Single seam, both sides: every transcript funnels through
  `terminal(read("../generated/<slug>.txt"))`; every capture funnels through the
  generator. No inline hand-typed transcript survives — this keeps the later
  colour upgrade (colorize-injected-terminal-output) a two-seam change.
- Determinism: the generator exports `MINDTAPE_NOW=2026-07-06` (the docs clock;
  the example authors its dates around it). Fixed clock ⇒ the same commit renders
  the same site.
- No `mt` re-implementation in Typst (no table rendering): output is shown
  verbatim.

== Generator (`just docs-gen`)

- A script under `www/` invoked by `just docs-gen`; `docs-build` / `docs-serve` /
  `docs-check` run it first.
- Builds `mt` (release), then for each entry in an explicit command→file map:
  - read commands (`ls`, `ls '<filter>'`, `views`, `check`, `config show
    --origin`, `add --describe`, `eval`, `--help`, `mt <cmd> --help`, `--version`)
    run against `examples/learn-piano` in place;
  - mutating commands (`add`, `done` / `flip`) run against a throwaway `cp` of the
    example in `$TMPDIR`, captured, then discarded.
  - capture stdout verbatim (plain text, `--color never`) → `www/generated/<slug>.txt`.
- dumps `mt --version`, `mt --help`, and per-command `mt <cmd> --help` verbatim.

== Injection (rewrite pages — www is WIP, not frozen)

- Replace every hand-typed `#terminal("…")` in the hero (`www/utils/ui.typ`),
  quickstart, filters, config, agents, templates, and cli pages with
  `terminal(read("../generated/<slug>.txt"))`.
- Hero source face: read a real task file from the example instead of the
  `_hero-source` literal.
- CLI reference: inject `mt --version` and per-command `mt <cmd> --help` verbatim
  under the authored section headings; drop the hand-typed global-flags table
  (`mt --help` carries it). Narrative framing stays authored; every fact comes
  from the binary.
- Rewrite prose wherever the piano example's real output differs from the current
  fabricated numbers.

== Scope

- `just docs-gen` script + command→file map; temp-copy for mutating commands;
  `MINDTAPE_NOW` pin.
- `www/generated/` gitignored; the docs recipes run `docs-gen` first.
- rewrite the injected-output pages (hero/`ui.typ`, quickstart, filters, config,
  agents, templates, cli) to `terminal(read(...))` / injected help+version.
- verbatim `--version` + per-command `--help` injection into `cli.typ`; drop the
  hand-typed flag table.
- keep the `terminal(read(...))` single render seam intact for the future colour
  swap.
