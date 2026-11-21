#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Give the templates their own command",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "cli", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[a reader sees which templates exist and which one mt add picks],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 21),
)

== AS-IS

+ Declare templates in `[templates] dirs`.
+ Ask which it accepts and which it picks.
+ 🔴 The only listing is `mt explain templates`, which retires. `mt add --list` stays hidden:

```
$ mt add --list
decision           kind: decision  22 slots
dev                kind: dev       21 slots
dev-stateful       kind: dev       22 slots
docs               kind: docs      22 slots
feat               kind: feat      22 slots
feat-stateful      kind: feat      23 slots
finding            kind: dev       25 slots
fix                kind: fix       22 slots
fix-stateful       kind: fix       23 slots
sweep-dry          kind: refactor  9 slots
sweep-mutants      kind: test      11 slots
sweep-repeats      kind: refactor  9 slots
sweep-size         kind: refactor  9 slots
sweep-undertested  kind: test      12 slots
task               kind: any       22 slots   default
tweak              kind: any        2 slots
note: use mt explain templates
```

== TO-BE

- 🟢 `mt templates` lists every template with the `kind` it sets, its slot count and which one `mt add` picks.
- 🟢 It reads the same template files `mt add` reads, so a template added to a configured directory appears without a config change.
- 🟢 `--json` carries the same list.

```
$ mt templates
decision   kind: decision  22 slots
…
task       kind: any       22 slots   default
tweak      kind: any        2 slots
```

== Changes

- `crates/mindtape-cli/src/cmd/templates.rs`: the new command, moved from `cmd/explain/templates.rs`.
- `crates/mindtape-cli/src/cmd/mod.rs`: register `templates`.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: `--list` retires, and `--template`'s help points at `mt templates`.
- `crates/mindtape-cli/tests/it/templates_e2e.rs`: the listing and the JSON shape.
- `schema/output/templates/1.schema.json`: the new output, replacing the `explain-templates` one.
- `www/content/cli/templates.typ`: the new page.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Added`.

== Alternatives

- Un-retire `mt add --list`: the list is not about adding, and a template command has room to grow.
- Keep the list in `mt config show`: the names come from a directory scan, so the config cannot hold them.

== Out of scope

- A template's slots. `mt add --describe -t <name>` prints them.
- The template directory scan. It moves unchanged.

== Dogfood

This repo declares 16 templates in `.mindtape/templates`, and `mt templates` lists them with `task` marked default.
