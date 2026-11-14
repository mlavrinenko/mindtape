#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Let add hooks fill fields and choose where new tasks land",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.5,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: helps[A directory per kind lets a reader browse one slice of the backlog.],
      implementable-tasks: neutral,
      enforced-practice: helps[A destination default and a dir rule both run in mt add. Nobody has to remember the layout.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
  links: depends-on(
    "add-the-on-add-hook-shape-validation-and-merge.typ",
    "share-one-writable-field-writer-across-mt-set-and-mt-add.typ",
    "unify-mt-add-and-mt-import-destinations-under-dir.typ",
    "keep-path-and-dir-rules-out-of-the-template-check.typ",
  ),
)

== AS-IS

+ Run `mt add "Remove the legacy import" -t fix` in a project that keeps each task kind in its own directory.
+ 🔴 `mt add` writes the file to the first task root, beside every other kind. `--set` can set a field once per call, but no hook fills it conditionally and no template states a destination.
+ `--dir` places one call at a time, and a rule can only catch a misplaced task after it exists. The only hook, `[[on.flip]]`, fires on a status change.


== TO-BE

- 🟢 `[[on.add]]` hooks run in order. `set` writes any writable field, `append` a list, and `dir` the destination. The hook inserts a field the template omits, and an absent `when` fires every add.
- 🟢 Each `when` sees the task as earlier hooks left it. Explicit input wins, a later `set` wins, and `append` accumulates.
- 🟢 `when` reads the current draft, `template ==` or `from_template(...)`, and `terminal`. `dir` is a path template over fields, a `dir` field lets a rule assert, and a hooked add refuses an outside-root destination.

```toml
[[on.add]]
name = "fixes"
when = 'starts_with("fix", title) || template == "fix"'
actions = [{ set = { due = "2026-12-01" }, dir = "tasks/fixes" }]

[[tasks.rules]]
name = "fixes-live-in-their-dir"
when = 'kind == "fix"'
assert = 'dir == "tasks/fixes"'
```


== Changes

- `crates/mindtape/src/template/fill/mod.rs`: add `fill_partial`, which replaces explicit slot values and leaves unfilled slots intact, so a draft can evaluate without placeholders.
- `crates/mindtape-cli/src/cmd/add/draft.rs`: apply explicit slot values with `fill_partial`, and `+=` appends to the slot base. Apply non-slot fields with the shared writer. Evaluate at `<first root>/__draft__.typ`, refuse a draft that does not load, and clear derived tags.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: run hooks in order over one mutable draft, rewrite both template forms against the resolved name, return slot updates, post-fill fields and `dir`, and expand `dir` after `set`/`append` with the config helper.
- `crates/mindtape-cli/src/cmd/add/placement.rs`: compute the stem and dir after hooks, refuse a hook-chosen destination outside every root, and resolve duplicate stems.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: refuse an explicit outside-root destination before the draft when hooks exist. Inject status and id once. Fill slot values, apply post-fill fields, then pre-verify. Add `--no-hooks`, report on stderr, and bump the describe schema.
- `crates/mindtape-cli/tests/it/add_e2e.rs`: one test per action, a field the template omits, a hardcoded field, and a template default plus `+=` reaching `when`. The tests also cover a later hook reacting to `set` and `derive`, a later `set` dropping earlier appends, and no rewrite of an earlier hook. They cover same-action `set` then `dir`, a bad `dir` macro, explicit-wins, outside-root refusal for flags and hooks, the draft path, and a rule refusal.
- `www/content/hooks.typ`, `www/content/templates.typ`, `www/content/filters.typ`, `www/content/rules.typ`, `www/content/dsl.typ`, `www/content/cli/add.typ`, `docs/src/agents.typ`: document add hooks, the action vocabulary, `dir` and the field-first rule.


== Alternatives

- A single pre-action snapshot stops a hook from reacting to an earlier hook, and order cannot express a chain.
- A per-hook original/current mode: it makes `append` and `set` bases ambiguous. A read-only `old("field")` predicate is the smaller later escape hatch.
- `[add] into` plus `[templates.into]`: two special cases where one ordered hook list works.
- A `[[tasks.rules]]` entry carrying `into`: rules assert by design. An effect-only rule is a hook in the wrong section.
- A slot-only `set`: misses a hardcoded `kind` and an omitted `due`.
- `set.dir` or `--set dir`: a destination is not a task field. A separate action keeps the writer honest.
- A `derive` action: the escape hatch for calculated values, filed separately.


== Out of scope

- The `on.add` shape, validation and merge stay out. The config foundation task owns them.
- The shared field writer, `starts_with`, template-pass filtering, pack explain/origin, `derive`, direct per-field flags, the `old()` predicate and the `--dir` refactor, each filed as its own task.
- `mt import` and `mt recur` stay out. Both keep their own field and placement paths.
- Moving a filed task between directories stays out. `mt mv` still renames only.
- `estimate`, framework priorities and `x-` fields in add hooks.


== Dogfood

This repo can set `assignee` and `due` from hooks and route by kind. When the flat backlog splits, the `dir` rule holds each kind in place without editing a template.

