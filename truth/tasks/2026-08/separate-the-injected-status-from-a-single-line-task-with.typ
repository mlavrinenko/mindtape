#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "separate the injected status from a single-line task with",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli", "tests"),
  estimate: (files: 3),
  status: done(2026, 9, 3),
)

== Summary

`mt add` injects its own `status:` into the template's `task.with(…)` call. When
that call sits on one line with no trailing comma, the injection lands
with no separator and writes a file that cannot parse.

Template:

```typ
#show: task.with(title: slot("title"))
```

Written:

```typ
#show: task.with(title: "No raw here"status: proposed(2026, 8, 17),
)
```

`mt add` then reports `written task failed to load: eval error: expected comma`,
after the broken file is already on disk. A template with one plain `string`
slot reproduces it, so the bug has nothing to do with slot kinds. The same
template with the argument on its own line and a trailing comma files fine.

No repo code has hit it, because every in-tree template is multi-line with
trailing commas: the built-in `BUILTIN_TASK`, the four sweep templates, both
example projects. The one-line form is legal Typst and a reasonable thing for
someone to write. The shape goes untested rather than unsupported.

== Scope

- `crates/mindtape-cli/src/task_edit.rs`: `upsert_named_arg` (or whichever of
  the injection helpers owns the seam) emits a separator when the existing
  argument list does not end in one. The fix belongs where the comma decision
  lives, not in a post-hoc string patch of the rendered call.
- A regression test over the one-line-no-trailing-comma template, beside the
  existing injection tests. The multi-line shape is already covered, which is
  precisely why this got through. The new test's whole value is being the other
  shape.
- Worth checking the same helper against an empty argument list
  (`task.with()`), which has the same "nothing to separate from" question with
  the opposite correct answer: no leading comma.

== Note

Independent of the input-grammar and slot-kind work that surfaced it. It
reproduces on an unmodified template with no `raw` slot and no tokens. Filed
separately rather than folded in.
