// What each built-in link kind means, keyed by the kind's spelling. The
// /links/ page reads the kinds and their rules (`max`, `blocking`, `acyclic`)
// from www/generated/link-kinds.json, which www/generate-facts.sh reads off
// `BUILTIN_KINDS` in crates/mindtape/src/link/mod.rs, so this dictionary holds
// only the meaning and never restates a rule the code holds. `just
// check-doc-facts` fails on a kind in that list with no entry here, and the
// page fails to build on the same gap.

#import "site.typ": u

#let link-kind-notes = (
  "depends-on": [This task can't start until the target is finished.],
  "parent": [This task is part of the target.],
  "recurred-from": [The target is the previous occurrence of this recurring task. `mt recur` writes it. See #link(u("/recurring/"))[Recurring tasks].],
  "supersedes": [This task replaces the target.],
  "duplicates": [This task is the same work as the target.],
  "related": [The two tasks are connected. The edge needs a note that says how.],
)
