#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Refuse set-derived filters and old() fields in add hooks",
  kind: "dev",
  priority: ice(
    impact: 9,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 5,
  links: (
    (
      task: "add-the-on-add-hook-shape-validation-and-merge.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[an add hook that reads ready() is refused at load rather than silently never firing],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
        "self-describing",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[one capability table keeps reads_set and add safety in step],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 11),
)

== AS-IS

+ Reviewing the on.add hook commit, where the add-unsafe field list and the filter name scanner decide what a hook may read.
+ 🔴 #raw("crates/mindtape/src/rules/typst.rs"): bare `ready` and `blocked` are missing from the add-unsafe list, their call forms are missed because the scan reads variable identifiers only, and `old()` refuses only `id`, `path` and `dir`.

```
[[on.add]]
name = "ready-call"
when = 'ready()'
actions = [{ set = { assignee = "x" } }]
```

`mt explain add` accepts it, and `mt add --print` never sets `assignee`. In reverse, `!ready` and `!ready()` fire on every add. `old("depth")`, `old("topo")`, `old("blocks")`, `old("children")` and `old("progress")` also load and read the synthetic draft. `old("file")` loads too, though `file` is in `ADD_UNSAFE`. A rule with `when = 'ready()'` is enforced against a template because `reads_set` also misses the call form.


== TO-BE

- 🟢 An add hook's `when` refuses every field or predicate whose answer depends on the filed task or the task set, in both the bare and call form.
- 🟢 `old()` refuses the same fields, so it cannot read the synthetic draft path.
- 🟢 A rule using `ready()` is dropped from the template pass, just like the bare form.

```
error: on.add[0].when: an add when cannot read `ready`: it is not known until the task is filed
```


== Changes

- `crates/mindtape/src/filter/eval.rs`: give each `FilterFunction` a set-dependent capability, and set it on `ready` and `blocked`.
- `crates/mindtape/src/filter/mod.rs`: add `calls()` and make `reads_set` read it.
- `crates/mindtape/src/rules/typst.rs`: share one add-unknown list with `old`, and reject set-dependent calls in `add_unsafe`.
- `crates/mindtape/src/filter/strings.rs`: make `old_value` refuse every add-unknown field.
- `crates/mindtape/src/config/add_hooks_tests.rs`: pin bare and call forms plus `old("depth")`.


== Out of scope

- The remaining fields of the old draft. This task covers only the set-derived answers an add cannot know.


== Dogfood

This repo's own `[[on.add]]` hooks cannot depend on readiness or the synthetic draft path.

