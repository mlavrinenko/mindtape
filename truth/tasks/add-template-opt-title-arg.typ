#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Make `mindtape add` template an option, title a positional",
  status: done(2026, 6, 23),
  priority: ice(impact: 6, confidence: 0.8, ease: 8),
  relates-to: (
    link("add-title-from-stdin.typ"),
    link("add-slots-from-env.typ"),
  ),
  tags: ("cli",),
)

== Summary

Today `mindtape add [TEMPLATE]` takes the template stem as the first positional,
so the title only arrives via stdin, `-f`, `--set title=`, or the editor. Flip
this: template moves to `-t/--template` (default still `[templates] default`),
and the positional becomes the task title. `mindtape add cleanup kitchen` joins
the words into the title "cleanup kitchen" and files `cleanup-kitchen.typ`. When
only the `title` slot is required this is the whole interaction; templates with
more unfilled required slots still open the editor as before.

== Scope

- Drop the positional `template`; add `-t/--template <TEMPLATE>` (`Option<String>`).
- Add a variadic positional `TITLE` (`Vec<String>`); join with spaces, trim, and
  seed the `title` slot when non-empty.
- Precedence: `--set` > `-f` > positional title > `MINDTAPE_ADD_*` env > stdin
  title > slot default. Positional title seeds before `layer_env`/stdin so those
  do not override it, and an explicit `-f`/`--set title=` still wins.
- Stem flows through `output_stem` -> `slug` unchanged: `slug("cleanup kitchen")`
  is `cleanup-kitchen`.
- Editor still opens for any remaining unfilled required slot (existing
  `should_open` gate); a title-only template skips it.

== Notes

- Touches `crates/mindtape-cli/src/add.rs` (args + run + helper) only; `mindtape`
  stays format-blind, no library churn.
- Update README and `www/index.html` usage: `add -t bug` for the template form,
  `add cleanup kitchen` for the title form.
