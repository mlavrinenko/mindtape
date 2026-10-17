// What each filter field holds, keyed by the name a filter writes, in the
// order the /filters/ page lists them. The page reads which fields exist and
// which sort from www/generated/filter-fields.json, which www/generate-facts.sh
// reads off `FIELDS` in crates/mindtape/src/model/fields.rs, so a note here
// never says whether its field sorts.
//
// A key that isn't in that list, such as `role` or `tags`, is a filter-only
// field, and the page says it doesn't sort. A field in the list with no note
// here fails the page build. Consecutive fields with the same note share a
// row.

#import "site.typ": u

#let filter-field-notes = (
  "status": [The status name. It sorts by position in the #link(u("/statuses/"))[status legend].],
  "status.name": [The name of the status stamp, sorted alphabetically.],
  "status.date": [The ISO date of the status stamp.],
  "status.note": [The note of the status stamp.],
  "status.index": [The status's 0-based position in the legend, as a number. A status outside the legend has none.],
  "priority": [Numbers. A #link(u("/priority/"))[framework] priority reads as its score.],
  "difficulty": [Numbers. A #link(u("/priority/"))[framework] priority reads as its score.],
  "kind": [Text.],
  "title": [Text.],
  "id": [Text.],
  "assignee": [Text.],
  "due": [ISO dates, compared as text, as in `due < now()`.],
  "start": [ISO dates, compared as text, as in `due < now()`.],
  "reviewed": [ISO dates, compared as text, as in `due < now()`.],
  "path": [The task file's path from the project root, with `/` on every platform.],
  "depth": [How many folders deep the file sits under its root, 0 in the root itself.],
  "role": [Matches when the assignee holds that role in `[[actors]]`.],
  "topo": [The task's place in the dependency graph, described below.],
  "blocks": [The task's place in the dependency graph, described below.],
  "children": [The number of #link(u("/links/"))[`parent`] children.],
  "progress": [The finished share of the children, from 0 to 1. A task with no children has none.],
  "estimate.<unit>": [One #link(u("/estimates/"))[estimate] unit, as a number.],
  "estimate": [The estimate dict. Only `has()` and `miss()` read it, as in `has(estimate, "days")`.],
  "tags": [The written and derived tags. `has(tags, "frontend")` reads them, and `contains` searches them.],
)
