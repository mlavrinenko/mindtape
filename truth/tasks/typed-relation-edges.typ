#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Typed relation edges (links field)",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  tags: (),
  status: proposed(2026, 7, 10),
)

== Summary

Collapse the four task-to-task link fields (`depends-on`, `relates-to`,
`supersedes`, `duplicates`) into one typed edge list, `links:`. Each edge is a
helper-produced dict carrying a target stem, a `kind`, and an optional note.
Ship a fixed set of built-in kinds; let a project (or a consumer like speconaut)
add extra kinds through config. Precedent: iCalendar `RELATED-TO;RELTYPE`, RFC
9253, and Jira's single extensible `issuelinks` list. This is a breaking change
to the DSL, the marker, and the model; there are no external users, so migrate
in place with no back-compat shim.

This unblocks recurrence linking (`recurred-from`) and the speconaut convergence
directive (a generic labelled-edge model both tools share).

== Decisions

- Field name is `links:`. The old four fields are removed, not aliased.
- Helpers take plain stem strings, never `link(...)`. The prelude helper shapes
  `(task: "stem.typ", kind: "…", note: none)`. The typeset render re-wraps the
  string in `link()` for a clickable PDF; harvest reads the string directly (no
  more pulling `dest` out of link content — simpler than today).
- Helpers are variadic: each positional string is one edge of that kind, so
  `depends-on("a.typ", "b.typ")` yields two edges. This offsets the extra words
  from folding `depends-on` into the list.
- No bare strings. Every edge uses a helper; a bare `"stem.typ"` in the list is
  a validation error. Intent must be explicit (see the `related` note rule).
- Built-in kinds (immutable, shipped): `depends-on`, `parent`, `recurred-from`,
  `supersedes`, `duplicates`, `related`. No `child` (derive the inverse of
  `parent` as a view), no `blocks` (derive the inverse of `depends-on`).
- Per-kind rules:
  - `depends-on` — cardinality ∞, note optional, blocks readiness, acyclic.
  - `parent` — cardinality 1 (two parents fail), note optional, acyclic.
  - `recurred-from` — cardinality 1, note optional.
  - `supersedes` / `duplicates` — cardinality ∞, note optional.
  - `related` — cardinality ∞, note REQUIRED. It is the escape hatch; a
    missing/empty note fails. Authored as `related("epic.typ")[why]`.
- Generic constructor `edge(kind, ..targets)[note]` builds an edge of any
  config-declared custom kind. Trailing content block is the note, uniform with
  status notes.
- Project/app config `[[tasks.links.kinds]]` adds extra kinds — `name`, `max`
  (cardinality), `note` (`required`/`optional`/`forbidden`), `blocking` (bool),
  `acyclic` (bool). Built-ins cannot be overridden or removed; users only add.

== Constraints

- The metadata marker emits `links` as an array of dicts; it drops the four old
  keys. `_core-slots` drops them and gains `links`.
- `parse_links` becomes a dict parser producing `Vec<Link>` where
  `Link { task, kind, note }`. Link resolution (target names a loaded task)
  applies to every edge. Cycle detection runs over `depends-on` and, separately,
  `parent`.
- Readiness (`ready.rs`) reads edges where `kind == "depends-on"`; rewriting it
  is fine (no external consumers).
- The filter language keeps `depends_on` as sugar for `links(kind:"depends-on")`
  and exposes `links(kind: …)`; `supersedes`/`duplicates` predicates map onto it.
- Migrate every `truth/tasks/*.typ` to `links:` in the same change; `mt check`
  stays green.

== Scope

- prelude: built-in helpers + generic `edge`; variadic targets + trailing note;
  swap `_core-slots`; render links grouped by kind.
- model / harvest: `Link` type, dict parsing, `links: Vec<Link>`.
- validate: resolution, `depends-on` + `parent` cycles, per-kind cardinality,
  required-note enforcement, unknown-kind rejection (built-in ∪ config).
- config: `[[tasks.links.kinds]]` parse + baked built-in defaults.
- ready / filter: read the folded edges.
- migrate the in-tree backlog; tests for cardinality, required-note, custom
  kind, both cycle checks, resolution, and the migration.
- Convergence note: shape `Link`/kind config so speconaut's labelled
  `depends-on` and artifact refs converge onto it (see its
  `commands/{cleanup_refs,link_scanner}.rs`, `engine/dsl_links.rs`).
