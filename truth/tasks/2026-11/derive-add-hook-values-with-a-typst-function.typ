#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Derive add-hook values with a Typst function",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[one Typst function covers any calculation instead of a second expression language],
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
    "let-rules-and-templates-choose-where-new-tasks-land.typ",
    "add-the-on-add-hook-shape-validation-and-merge.typ",
    "share-one-writable-field-writer-across-mt-set-and-mt-add.typ",
  ),
)

== AS-IS

+ Write an `[[on.add]]` action that sets or appends a literal value.
+ 🔴 The hook cannot express a tag derived from the title, such as the verb or the component name. It needs a Typst helper outside the action.
+ A values language in TOML would be a second parser, with its own types and errors.


== TO-BE

- 🟢 `derive = { typst, function, field }` calls the named function and sets `field`, as a rule's `check` calls a function. It replaces a scalar value. A list returns its full value.
- 🟢 `dir = { derive = { typst, function } }` derives the destination. A static `dir = "tasks/{kind}"` stays the common form.
- 🟢 The function gets the current sanitized draft, including earlier hooks' fields and no `dir`, `path`, ids, depth or set fields. It writes back before the next hook. The action refuses a wrong type or a derived `dir` outside roots, and a derived `dir` stays project-relative.
- 🟢 `mt add --describe` lists the function, and the report states the hook that derived the value.

```toml
[[on.add]]
name = "classify"
when = 'template == "fix"'
actions = [
  { derive = { typst = "truth/add.typ", function = "classify_tags", field = "tags" } },
  { dir = { derive = { typst = "truth/add.typ", function = "classify_dir" } } },
]
```


== Changes

- `crates/mindtape/src/config/add_hooks.rs`: add the `derive` action and the `dir = { derive = ... }` form.
- `crates/mindtape/src/rules/typst.rs`: expose a public typed-call API that shares the module and function loader with rule `check`.
- `crates/mindtape-cli/src/cmd/add/hooks.rs`: pass the sanitized draft, call the function, write the value back into the mutable draft, apply it, and validate a derived `dir`.
- `crates/mindtape-cli/src/cmd/add/mod.rs`: wire the action and report it, and bump the describe schema.
- `crates/mindtape-cli/tests/it/add_e2e.rs`: a derived list, a derived `dir`, a wrong type, a missing function, and a derived `kind` a later hook matches.
- `www/content/hooks.typ`, `www/content/templates.typ`: document `derive`, the `dir` form, both template forms and the function contract.


== Alternatives

- A TOML value expression language: a second parser, type system and set of errors to learn.
- A `run` command returning JSON: a process, an allow-list entry and stdout parsing.
- A template expression cannot serve here, because templates shape tasks, not the project's add defaults.
- A derived `append` mode: return the full list from the function instead of adding a second action mode.
- `compute` or `eval` as the name: `derive` matches the existing derived-field vocabulary.


== Out of scope

- Multiple fields from one call stay out. Start with one `field` per action.
- A derived `append` mode stays out. The function returns the full list.
- File, network or clock access from the function.
- A value that reads `dir`, `path` or the whole set stays out. The draft stays add-safe.


== Dogfood

This repo can derive a `found` or `swept` tag from the title, or a component tag from an `x-` field, without a second config language.

