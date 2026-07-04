#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt add: array-append --set syntax and +tag sugar",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  tags: ("cli", "docs",),
  status: done(2026, 7, 4)[shipped: key\[\]= array-append, +tag sugar, nested-slot docs],
)

== Summary

`mt add --set tags=a,b,c` silently produces one tag `"a,b,c"` instead of three — `coerce_list` wraps any `Str` RawValue in a single-element vec, and `--set` never produces `RawValue::List`. Fix by giving `--set` an array-append syntax (`key[]=value`, repeatable, HTTPie-precedented) instead of comma-splitting, since comma-splitting would break any list value that legitimately contains a comma (tags, depends-on stems). Add `+tag` positional sugar to `mt add` (peel `+word` tokens from the title args into the `tags` slot) matching the dominant task-manager CLI convention (Taskwarrior/todo.txt/dstask). Document the existing nested-slot pattern (`.mindtape/templates/task.typ`'s `ice(impact: slot(...))`) as the supported way to expose structured fields from a template, since it already works via typst-edit's recursive `find_calls` walk and needs no new syntax.

== Scope

- `crates/mindtape/src/template.rs`: `coerce_list` rejects a bare `Str` for a `List` slot (was: silently wraps it in a 1-element vec) with a helpful error naming the `key[]=` fix.
- `crates/mindtape-cli/src/add_values.rs`: `collect_values` parses a `--set` key of the form `name[]` as "append to list `name`", accumulating repeats into `RawValue::List` instead of last-write-wins.
- `crates/mindtape-cli/src/add.rs` / `add_values.rs`: `+tag` positional-word sugar — peel leading `+`-prefixed title words into the `tags` slot's fill value before title-joining; title keeps the remaining words.
- Regression tests: repeated `key[]=` accumulates; a bare `key=` into a list slot errors with the new message; a `[]=` value containing a literal comma survives untouched; `+tag` sugar splits correctly and coexists with `--set tags[]=`.
- Docs: note the nested-slot pattern for structured fields (ICE-style) in the `add` section of `CLAUDE.md`/README/`skills/mindtape/SKILL.md` so it's discoverable without re-deriving it.
- Out of scope: dotted-path (`a.b.c=`) nesting, `:=` raw-JSON typed `--set` values — no concrete need yet (YAGNI per research: https://github.com/helm/helm/issues/2952 shows the escaping cost of a richer grammar).
