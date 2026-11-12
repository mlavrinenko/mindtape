#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Share one writable-field writer across mt set and mt add",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("mt", "tests", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (:),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one field vocabulary and renderer, so mt set and mt add cannot drift],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Run `mt add --set due=2026-12-01 "Sort tasks"` in a project whose template has no `due` slot.
+ 🔴 `mt add --set` refuses `due` as an unknown slot, while `mt set` writes the same field on a filed task.
+ `mt set` keeps its writable table, shapes and renderer private in the CLI, so a third caller would copy them.


== TO-BE

- 🟢 One `WritableField` table and renderer live in the library, shared by `mt set`, `mt add --set` and add hooks.
- 🟢 `mt add --set due=2026-12-01` inserts a field the template omits; `--set tags+=bug` appends to the slot default or the explicit base, and `--set tags=...` replaces. The token parser keeps the append, so `fill` cannot replace it with one item.
- 🟢 One rule covers every list: actions apply in order; `=` replaces, `+=` and hook `append` append, a later `set` drops earlier appends, and explicit `=` is the base for `append` and blocks a hook `set`.
- 🟢 `mt set dir=...` is refused: a filed task's directory is not a field, and `mt mv` only renames.

```
$ mt add --set due=2026-12-01 --set tags+=bug "Remove the legacy import"
wrote tasks/remove-the-legacy-import.typ
```


== Changes

- `crates/mindtape/src/model/writable.rs`: the writable table, shapes, renderer and elsewhere hints.
- `crates/mindtape-cli/src/cmd/set.rs`: use the shared table and renderer.
- `crates/mindtape-cli/src/cmd/add/field_values.rs`: collect non-slot writable fields, and apply fields and list appends to the evaluated slot base.
- `crates/mindtape-cli/src/cmd/add/values.rs`: accept a writable field that names no slot, preserve `+=` as an append operation from `tokens.rs`, and route it to the field writer.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: apply field values after fill and before pre-verify.
- `crates/mindtape-cli/tests/it/add_e2e.rs`, `crates/mindtape-cli/tests/it/set_e2e.rs`: a field the template omits, a template default plus `+=`, explicit replace, a later `set` dropping earlier appends, and the `dir` refusal.
- `www/content/templates.typ`, `www/content/cli/add.typ`, `www/content/cli/set.typ`: document the shared field surface.


== Alternatives

- Keep private copies: the second caller drifts on field names, shapes or escaping.
- Put the writer in the CLI only: the library config cannot validate hook actions at load.
- Generate a renderer per field: more code, and the shapes still need one table.


== Out of scope

- `estimate`, framework priorities and `x-` fields; their shapes stay with their owners.
- Direct `--due` flags; they are a separate task.
- `mt import`; it keeps its record shape.


== Dogfood

This repo's add hooks and `mt set` share the writer, and `mt add --set due=...` works on the built-in template.

