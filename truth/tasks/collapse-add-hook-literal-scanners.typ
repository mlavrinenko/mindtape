#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Collapse the add-hook literal scanners with split_once",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 7,
  ),
  difficulty: 3,
  links: (
    (
      task: "add-the-on-add-hook-shape-validation-and-merge.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[a guard that cannot fail has no untested error to explain],
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
        sound-codebase: helps[one quoted-literal reader replaces three find-and-get copies],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 11),
)

== AS-IS

+ Reviewing the on.add when scanner, where four guards close one string literal.
+ 🔴 #raw("crates/mindtape/src/config/add_hooks_when.rs"): `parse_template_comparison` repeats one error four times, two `get` guards among its four error returns cannot fail, and the same find-and-get shape is in `parse_named_call` and `DirTemplate::parse`.

```
"function": "parse_template_comparison",
"cyclomatic": 7.0,
"coverage": 85.71428571428571,
"crap": 7.142857142857143,
```

`tail.strip_prefix('"')` is followed by `literal.find('"')`, then `literal.get(..end)` and `literal.get(end + 1..)`. The last two are char-boundary slices of a string `find` just validated, so they are always `Some`. `old_value` also repeats `old("kind") == "feat"` four times, and `when_templates` promises source order but runs two passes.


== TO-BE

- 🟢 One quoted-literal helper serves the template comparison, the named call and the dir macro.
- 🟢 The malformed-literal message and the `old()` example exist once.
- 🟢 `when_templates` returns names in source order for both call and comparison forms.

```
let (name, consumed) = tail
    .trim_start()
    .strip_prefix('"')
    .and_then(|literal| literal.split_once('"'))
    .ok_or_else(|| "template compares with a string literal".to_owned())?;
```


== Changes

- `crates/mindtape/src/config/add_hooks_when.rs`: use `split_once`, one error constant, and collect names with offsets.
- `crates/mindtape/src/config/hooks.rs`: add the shared `quoted()` helper and use it in `parse_named_call`.
- `crates/mindtape/src/config/add_hooks_dir.rs`: use `strip_prefix('{')` plus `split_once('}')` in `DirTemplate::parse`.
- `crates/mindtape/src/filter/strings.rs`: give `old()` one `const EXAMPLE`.
- `crates/mindtape/src/config/add_hooks_tests.rs`: pin source order and the two-form scanner.


== Out of scope

- Escaped quotes inside a template name. The current scanner does not unescape them either.


== Dogfood

The next add-hook parser keeps one error string and one boundary check.

