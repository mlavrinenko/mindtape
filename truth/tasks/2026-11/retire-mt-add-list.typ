#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Retire mt add --list",
  kind: "refactor",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[one surface for the template inventory, not two.],
      implementable-tasks: helps[the agent finds the templates where it starts, not behind a cryptic flag.],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the CLI lists the templates in the command that explains the project.],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the docs point at the listing instead of a retired flag.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: parent("make-mt-explain-the-session-brief.typ")
    + depends-on("list-the-templates-in-mt-explain.typ"),
)

== AS-IS

+ Run `mt add --list`.
+ 🔴 It prints 16 bare names, with no `--json`, no kind and no slot count.
+ Read the flag name.
+ 🔴 `--list` on `add` reads like "list the tasks I added" or "list the fields", not "list the templates".
+ Run `mt explain templates`.
+ 🔴 It does not exist yet, so `mt add --list` is the only surface.

== TO-BE

- 🟢 `mt explain templates` is the one surface for the template inventory.
- 🟢 `mt add --list` becomes a deprecated alias for one minor: it prints the `mt explain templates` listing and a `note: use mt explain templates` to stderr.
- 🟢 The next minor removes the flag, and `mt add --list` fails with a hint that names `mt explain templates`.
- 🟢 The `mt add` unknown-template error and `--help` point at `mt explain templates`.
- 🟢 The completion keeps reading the library `list_templates`, not the flag.

```text
$ mt add --list
decision   kind: decision   8 slots
...
task       kind: any        12 slots   default
note: use mt explain templates
```

== Changes

- `crates/mindtape-cli/src/cmd/add/mod.rs`: the `--list` flag becomes hidden and prints the listing plus the note. The next minor removes it.
- `crates/mindtape-cli/src/cmd/add/values.rs`: the unknown-template hint names `mt explain templates`.
- `www/content/templates.typ`: the `mt add --list` references.
- `www/content/cli/add.typ`: the `--list` flag.
- `docs/src/skill.typ`: the listing in the recipe.
- `skills/mindtape/SKILL.md`: regenerated.

== Out of scope

- The template slots. `mt add -t <name> --describe` owns them.
- The `[[on.add]]` hooks. `mt explain add` owns them.
- A new template format. The listing reads the files `mt add` already reads.
- The `mt explain templates` listing itself. The sibling task owns it.

== Dogfood

This repo's `mt add --list` prints the `mt explain templates` listing and a note, and the skill points at `mt explain templates`.
