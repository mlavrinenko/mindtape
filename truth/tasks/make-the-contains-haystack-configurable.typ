#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "make the contains haystack configurable",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "docs"),
  estimate: (files: 6),
  status: done(2026, 8, 8)[landed 2026-08-08; just check green on merged HEAD],
  links: depends-on("unify-search-and-contains-into-one-function.typ")[
    the function whose default this configures
  ]
    + depends-on("split-filter-rs-before-it-blocks-the-next-change.typ")[
      lands after the split, since both edit the same module
    ],
)

== Summary

`contains("bug")` with no named fields sweeps a hardcoded default of `title`,
`id`, `body`. That default is a judgement about what "the prose surface" means,
and it is not the same judgement in every project. Two things were deliberately
left out and are the obvious first requests:

- Link notes (`related("x")[why]`) are prose, but live on `links[].note`, not
  in `body`.
- `tags` is a controlled vocabulary with `has_tag()` for exact match, so
  substring-matching it is unusual — but a project with free-form tags would
  want it.

Make the default set a config key rather than a constant, so a project states
its own answer.

== Scope

- A config key under the existing filter/ls area naming the default
  `contains()` haystack — an ordered list of field names, defaulting to
  `title`, `id`, `body` so behaviour is unchanged when unset.
- Extend the searchable vocabulary to cover the two excluded surfaces:
  - `note` — every `links[].note` on the task, concatenated for matching.
  - `tags` — authored tags and derived tags both (`Task::all_tags`), matching
    how `has_tag()` already sees them.
- Validate the configured list at config load, against the same
  `is_contains_field` vocabulary an explicit `contains("x", field)` call uses.
  An unknown name is a config error naming the field and listing the valid set
  — never a silent drop, which would leave a project believing it searches a
  surface it does not.
- The vocabulary stays one declared source. Adding `note` and `tags` must not
  reintroduce a second hand-rolled field match beside `crate::fields`; they
  are text-only like `body`, so they extend the same text-only tier rather
  than entering `FIELDS`.
- An explicitly named field keeps working regardless of the configured
  default: `contains("bug", note)` searches notes even when the default set
  omits them.
- Tests: the default unchanged when the key is unset, a project narrowing the
  default to `title` only, a project widening it to include `note` and `tags`,
  an unknown field name rejected at load, and an explicit field overriding a
  narrowed default.
- Docs: `www/content/filters.typ` and `www/content/config.typ`.

== Notes

A default that applies uniformly across every project inherits the strictest
constraint of anything that could land in it — so widening the shipped default
to include notes and tags is the wrong move. Keep the shipped default narrow
and let a project opt in.
