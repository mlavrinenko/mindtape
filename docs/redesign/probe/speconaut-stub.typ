// Probe stub: how speconaut would consume the MindTape core as a library.
// speconaut depends on mindtape, re-exports its `task` + status shortcuts so a
// task file needs ONE import, and adds the typed `bounded-context` entity plus
// the closed-allowlist `task-extras` validator (the old `task()` finish()
// rejection, relocated here).

#import "../prelude.typ": *

#let bounded-context(name, layer: none) = (kind: "bounded-context", name: name, layer: layer)

#let _extras-slots = (
  "contexts", "reviewed", "decisions", "code", "supersedes", "duplicates",
)
#let task-extras(..args) = {
  assert(args.pos().len() == 0, message: "task-extras takes only named args")
  for pair in args.named().pairs() {
    assert(
      pair.at(0) in _extras-slots,
      message: "unknown speconaut task field `" + pair.at(0) + "`",
    )
  }
  (ns: "speconaut", ..args.named())
}
