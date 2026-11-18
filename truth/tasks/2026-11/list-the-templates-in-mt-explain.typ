#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "List the templates in mt explain",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the templates read as a list with their kind and size, not 16 bare labels.],
      implementable-tasks: helps[an agent picks the template before it files, without a guess.],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI answers which templates exist, not only which slots one has.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt explain templates`.
+ 🔴 It fails with `unexpected argument 'templates'`, and the index never mentions templates.
+ Run `mt add --list`.
+ 🔴 It prints 16 bare names with no kind, slot count or `--json`. The reader still runs `mt add -t <name> --describe` for each.
+ Run `mt add -t dev --describe`.
+ 🔴 It describes one template at a time, so comparing 16 means 16 commands.

== TO-BE

- 🟢 `mt explain templates` lists every template, one line each: name, the `kind` it sets, slots, and `default` on the one `mt add` picks.
- 🟢 The `kind` column reads `any` for `task`, whose `kind` slot has no fixed value. The `status` column disappears, because `mt add` stamps it.
- 🟢 `--json` wraps the list in `templates` under `$schema`.
- 🟢 The index's `task:` block names the templates and the default.
- 🟢 The listing reads the same template files `mt add` reads, so a template added to `.mindtape/templates/` appears without a config change.
- 🟢 The `mt add` unknown-template error points at `mt explain templates`.

```text
decision   kind: decision   8 slots
dev        kind: dev        12 slots
task       kind: any        12 slots   default
```

== Changes

- `crates/mindtape-cli/src/cmd/explain/templates.rs` (new): the listing, its JSON and its tests.
- `crates/mindtape-cli/src/cmd/explain/mod.rs`: the `Templates` variant and its dispatch.
- `crates/mindtape-cli/src/cmd/add/values.rs`: expose the template summary the listing reads.
- `schema/output/explain-templates/1.schema.json` (new): the listing's envelope.
- `www/content/cli/explain.typ`: the `templates` listing.
- `www/content/templates.typ`: the `mt add --list` retirement.
- `docs/src/skill.typ`: the listing in the recipe.
- `skills/mindtape/SKILL.md`: regenerated.

== Alternatives

- Fold the names into the index only: the reader still cannot see a template's kind or slot count without one command per template.
- Alias `mt explain templates` to `mt add --list`: the names arrive, but the kind and slot count do not.
- A `templates:` block in the index with every slot: the index becomes the template reference, and `mt add --describe` loses its job.
- Keep `mt add --list` as the only surface: the flag is cryptic, has no `--json`, and two commands would answer one question.
- A `status` column: every template leaves the status to `mt add`, so the column is constant.

== Out of scope

- The slot details. `mt add -t <name> --describe` owns them.
- The `[[on.add]]` hooks. `mt explain add` owns them.
- A new template format. The listing reads the files `mt add` already reads.
- The `mt add --list` removal. The sibling task owns it.

== Dogfood

This repo's `mt explain templates` lists the 16 templates, and the skill points at it before `mt add`.
