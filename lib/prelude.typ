// MindTape prelude — per-file task model.
// See docs/decisions/002-per-file-rewrite-and-speconaut-convergence.md.
//
// THIS layer (Typst) is config-agnostic and consumer-blind. It only:
//   - shapes one task per file via `#show: task.with(...)`
//   - emits one labelled metadata marker the Rust layer collects
//   - enforces the extension discipline: any non-core slot must be `x-<ns>`
// The Rust layer, driven by `.mindtape/config.toml`, owns the rest:
//   - the status legend, the numeric priority/difficulty scales, the kind enum,
//     required-unless-terminal
//   - identity (filename stem), the filter language
// Keeping the prelude config-free means the same package serves every project.

// Status is a plain (name, date, note) dict. The project's config decides which
// names are legal; the prelude ships shortcut constructors for a broad common
// set so an unconfigured project still gets ergonomic status calls.
#let status(name, year, month, day, note: none) = (
  name: name,
  date: datetime(year: year, month: month, day: day),
  note: note,
)
// The optional note rides in as a trailing content block: `done(y, m, d)[note]`.
#let _mk(name) = (y, m, d, ..note) => status(name, y, m, d, note: note.pos().at(0, default: none))
#let draft = _mk("draft")
#let proposed = _mk("proposed")
#let accepted = _mk("accepted")
#let todo = _mk("todo")
#let wip = _mk("wip")
#let doing = _mk("doing")
#let blocked = _mk("blocked")
#let in-review = _mk("in-review")
#let done = _mk("done")
#let cancelled = _mk("cancelled")
#let failed = _mk("failed")

// Generic date shortcut: `on(2026, 6, 18)` instead of `datetime(year: …, …)`.
// Serves every date slot uniformly (due, start, reviewed, …).
#let on(y, m, d) = datetime(year: y, month: m, day: d)

// Core slots every consumer shares. Anything else MUST be an x-<ns> extension.
#let _core-slots = (
  "title", "status", "priority", "difficulty", "kind", "estimate",
  "depends-on", "relates-to", "supersedes", "duplicates", "tags", "due", "start", "reviewed", "id",
)

#let task(body, ..meta) = {
  assert(meta.pos().len() == 0, message: "task takes no positional args besides its body")
  let m = meta.named()
  let extras = (:)
  for pair in m.pairs() {
    let key = pair.at(0)
    if key not in _core-slots {
      assert(
        key.starts-with("x-"),
        message: "unknown task slot `" + key + "`; tool extensions must use an x-<ns> slot",
      )
      extras.insert(key, pair.at(1))
    }
  }
  [#metadata((
      marker: "mindtape.task",
      fields: (
        title: m.at("title", default: ""),
        status: m.at("status", default: none),
        priority: m.at("priority", default: none),
        difficulty: m.at("difficulty", default: none),
        kind: m.at("kind", default: none),
        estimate: m.at("estimate", default: (:)),
        depends-on: m.at("depends-on", default: ()),
        relates-to: m.at("relates-to", default: ()),
        supersedes: m.at("supersedes", default: ()),
        duplicates: m.at("duplicates", default: ()),
        tags: m.at("tags", default: ()),
        due: m.at("due", default: none),
        start: m.at("start", default: none),
        reviewed: m.at("reviewed", default: none),
        id: m.at("id", default: none),
      ),
      extras: extras,
    )) <mindtape>]
  body
}
