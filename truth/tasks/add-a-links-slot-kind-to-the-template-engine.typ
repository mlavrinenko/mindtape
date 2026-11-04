#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a links slot kind to the template engine",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "cli", "stateless"),
  estimate: (src-files: 4),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[a template can ask for typed edges, so a filed task carries its links without a second command],
      ),
      ..(
        "readable-tasks",
        "enforced-practice",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the slot renders the same edge dicts the prelude helpers build],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
)

== AS-IS

+ Run `mt add -t <template> -f -` with a `links:` value whose entries are mappings.
+ 🔴 `mt add` refuses it before it reaches the template: `yaml_values` accepts only scalars and string lists, so a typed edge has no slot shape.

```
$ mt add -t probe -f - <<'YAML'
links:
  - kind: related
    task: other.typ
    note: why
YAML
error: nested mappings are unsupported: 'links'
```

== TO-BE

- 🟢 `slot("links", kind: "links")` holds a list of edges, each a `{kind, task, note}` mapping.
- 🟢 A filled slot renders the same `(task: "...", kind: "...", note: ...)` dicts the prelude's `edge(...)` builds, so the marker parser needs no second shape.
- 🟢 An empty list renders `()`, and a template can leave the slot optional.
- 🟢 `mt add --describe` lists the kind, and `mt _cli --json` carries it with the other slot kinds.
- 🟢 The docs page and the skill reference explain the YAML shape.

```
links: [{kind: surfaced-by, task: origin.typ}, {kind: related, task: other.typ, note: why it relates}]
```

becomes

```
links: (
  (task: "origin.typ", kind: "surfaced-by", note: none),
  (task: "other.typ", kind: "related", note: "why it relates"),
),
```

== Changes

- `crates/mindtape/src/template/tests_fill.rs`: the red test first, a links slot renders an edge array, an empty list renders `()`, and a string is a type error.
- `crates/mindtape/src/template/mod.rs`: `SlotKind::Links`, its spelling and its place in `ALL`.
- `crates/mindtape/src/template/fill/mod.rs`: `LinkValue`, `RawValue::Links`, coercion and rendering.
- `crates/mindtape/src/lib.rs`: re-export `LinkValue`.
- `www/utils/slot-kind-notes.typ`: the note the docs page needs for the new kind.

== Alternatives

- A template-local helper over a `list` of `kind|target|note` strings: it invents a second grammar and cannot carry a note containing the separator.
- A `raw` slot in the `links:` argument: it splices caller text as Typst code, which the slot kind's own docs warn against.

== Out of scope

- Parsing YAML and JSON into the new value: the next task does that in `mt add`.
- The finding template and its `surfaced-by` rule: a third task uses this kind.

== Dogfood

The next task fills the `links` slot from `-f`, and this task's tests pin the rendering the finding template will use.
