#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Give every settable field a direct mt add flag",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        self-describing: helps[every settable field appears in mt add --help as its own flag],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (:),
      ..("sound-codebase", "docs-match-code"),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
  links: depends-on(
    "share-one-writable-field-writer-across-mt-set-and-mt-add.typ",
    "unify-mt-add-and-mt-import-destinations-under-dir.typ",
  ),
)

== AS-IS

+ Run `mt add --set due=2026-12-01 --set tags+=bug "Sort tasks"`.
+ 🔴 `due` and `tags` are not named in `mt add --help`; every value goes through the generic token grammar.
+ The field writer exists, but the flag surface does not advertise it.


== TO-BE

- 🟢 `mt add --due 2026-12-01`, `--assignee agent` and `--priority 6` are direct aliases for `--set field=value`.
- 🟢 `--set` already sets any writable field; `--dir` stays the separate destination flag, because a directory is not a task field.
- 🟢 Lists keep `--set tags+=bug`; a scalar flag and `--set` naming the same field is refused by name.

```
$ mt add --due 2026-12-01 --dir tasks/fixes "Remove the legacy import"
wrote tasks/fixes/remove-the-legacy-import.typ
```


== Changes

- `crates/mindtape-cli/src/cmd/add/mod.rs`: add the scalar flags and route each through the shared field writer.
- `crates/mindtape-cli/tests/it/add_e2e.rs`: each scalar flag, a field the template omits, and a flag/`--set` conflict.
- `www/content/templates.typ`, `www/content/cli/add.typ`: document the flags and the `--set` equivalence.


== Alternatives

- Keep `--set` only: fewer flags, but no field is discoverable in `--help`.
- A flag for every field including lists: help clutter and an ambiguous list syntax.
- Generate flags from config: clap needs static flags, and a generated wrapper hides the surface.


== Out of scope

- `--set dir`; a directory is not a task field, and `--dir` already names it.
- `mt import` flags; `mt add` first.
- Short aliases such as `-d`, and completion changes beyond the existing.
- Framework priorities and estimate dicts; keep the raw number and the current rules.


== Dogfood

This repo's docs and skill examples use `--due` and `--dir` instead of `--set` when they name a field.

