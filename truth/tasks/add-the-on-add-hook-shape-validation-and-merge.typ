#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add the on.add hook shape, validation and merge",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "tests", "stateless", "no-repro"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a pack ships add hooks and a project overrides or disables one by name],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one hook shape and one validation path across project, layer and pack],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 10),
  links: depends-on(
    "share-one-writable-field-writer-across-mt-set-and-mt-add.typ",
    "match-a-text-prefix-with-a-starts-with-predicate.typ",
  )
    + depends-on("keep-path-and-dir-rules-out-of-the-template-check.typ"),
)

== AS-IS

+ Add a rule or a flip hook; there is no add hook and no directory field.
+ 🔴 A project cannot state a conditional default for a new task, and a `path` rule is the only way to talk about a filed task's directory.
+ A pack has no array to ship an add default, and no name to override or disable.


== TO-BE

- 🟢 `[[on.add]]` takes an optional `when` and `actions = [{ set = {...}, append = {...}, dir = "..." }]`; `template == "name"` and `from_template("name")` are validated at load. The comparison form takes `==`/`!=` and a template-name literal; bare `template`, another operator and non-literals error.
- 🟢 `set`/`append` keys validate against the shared writer, `starts_with` is in the whitelist, `set.dir` is refused, and the same field in `set` and `append` is an error.
- 🟢 `on.add` is one of the arrays that merge by name; a nearer hook replaces or disables a pack's.
- 🟢 `dir` is a project-root-relative template over `{template}` and add-safe scalar fields, expanded after `set`/`append`; an unknown macro, missing value or path-like expansion (`/`, `\`, `.`, `..`, empty) fails, and `dir` joins the filter fields.

```toml
[[on.add]]
name = "fixes"
when = 'template == "fix"'
actions = [{ set = { due = "2026-12-01" }, dir = "tasks/fixes" }]
```


== Changes

- `crates/mindtape/src/config/add_hooks.rs`: add `AddHook` and `AddAction`, optional `when`, `set`/`append`/`dir`, the add-safe whitelist, both template forms, a library `template_names(config, root)`, the `dir` template parser and a `validate_component` helper.
- `crates/mindtape/src/config/hooks.rs`: add the `on.add` array to `OnConfig`, generalize the named-call rewriter for both template forms, reject bare or non-literal `template`, and update the `Config::on` doc.
- `crates/mindtape/src/config/packs.rs`: add `on.add` to the arrays and prefixes that merge by name.
- `crates/mindtape/src/config/layers.rs`: call `validate_add_hooks` with the project root.
- `crates/mindtape/src/model/fields.rs`: add `dir`, the directory part of `path`, to the filter fields.
- `crates/mindtape/src/config/add_hooks_tests.rs`, `crates/mindtape/src/config/packs_tests_add.rs`: the shape, the whitelist, both template forms, `==`/`!=` valid and bare/non-literal/other rejected, a bad template name, an unknown `dir` macro, a path-like or missing expansion, same-action `set` then `dir`, a root-level `dir` of `""`, named replace and `disable`.
- `www/content/hooks.typ`, `www/content/filters.typ`, `www/content/rules.typ`: document the shape, the fields and the merge.


== Alternatives

- Rules only: a rule cannot choose a default.
- A project-only array: a pack cannot ship a placement.
- Validate the template name on the first add: a typo lands instead of failing the config.
- A rule carrying `into`: rules assert by design; an effect belongs in a hook.


== Out of scope

- The add pipeline, reports and `--no-hooks`; the execution task owns them.
- `mt explain add` and origin; a separate task.
- The `derive` action; a separate task.
- The shared field writer, `starts_with` and the template-pass filter; prerequisites, filed.


== Dogfood

This repo's packs can ship a default placement hook, and the project can override or disable it by name.

