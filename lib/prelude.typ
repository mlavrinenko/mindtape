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

// Priority framework packagers. These are pure sugar: they only shape a
// `(framework, inputs)` dict for the Rust layer to score — no math, bounds, or
// classification live here (those are config + Rust concerns). `framework` is
// the generic escape hatch; the named helpers fix one built-in's input shape.
#let framework(name, ..inputs) = (framework: name, inputs: inputs.named())
#let ice(impact: none, confidence: none, ease: none) = framework(
  "ice", impact: impact, confidence: confidence, ease: ease,
)
#let rice(reach: none, impact: none, confidence: none, effort: none) = framework(
  "rice", reach: reach, impact: impact, confidence: confidence, effort: effort,
)
#let wsjf(cost_of_delay: none, job_size: none) = framework(
  "wsjf", cost_of_delay: cost_of_delay, job_size: job_size,
)
#let eisenhower(urgent: false, important: false) = framework(
  "eisenhower", urgent: urgent, important: important,
)
#let moscow(level) = framework("moscow", level: level)

// Template authoring helper. `mt add` discovers `slot(...)` calls on the
// parse plane and replaces each with a filled literal, so a filed task holds
// plain values and never calls `slot`. At eval time a slot returns its `default`,
// keeping a template a previewable, valid task. `kind` drives how `add` renders a
// filled value ("string", "int", "float", "bool", "date", "list", "text",
// "choice"); `prompt`/`required`/`choices` steer the fill UI. Args must be
// literals — `add` reads them without evaluating the template.
#let slot(name, default: none, kind: "string", prompt: none, required: true, choices: none) = default

// Core slots every consumer shares. Anything else MUST be an x-<ns> extension.
#let _core-slots = (
  "title", "status", "priority", "difficulty", "kind", "estimate",
  "depends-on", "relates-to", "supersedes", "duplicates", "tags", "due", "start", "reviewed", "id",
)

// ── Typeset render ────────────────────────────────────────────────────────────
// The third face. `#show: task.with(...)` wraps the whole file; below the marker
// we lay the authored fields out as a printed index card (a title, status /
// priority pills, a metadata grid, tag chips, then the body). This is paged
// content only — headings, boxes, grids, `set page` — never `html.elem` (that
// path is the docs SSG's). It is inert under `typst-harvest` eval: the harvester
// evaluates the module and walks it for the `<mindtape>` metadata element,
// laying nothing out, so the chrome costs harvesting nothing and the marker
// stays the single queryable payload. Every field is rendered only when set and
// skipped cleanly when none/empty.

#let _muted = luma(110)
#let _rule = luma(205)

// A rounded pill: bold, small, coloured wash. `fg`/`bg` default to a neutral grey.
#let _pill(body, fg: luma(70), bg: luma(232)) = box(
  fill: bg,
  inset: (x: 0.55em, y: 0.3em),
  radius: 1em,
  text(size: 0.72em, weight: "bold", fill: fg, tracking: 0.02em, body),
)

// Per-status wash, keyed by the prelude's shortcut names. An unlisted status
// (a project may legislate its own) falls back to a neutral grey — colour is
// cosmetic, never authoritative; the Rust layer owns the legend.
#let _status-styles = (
  draft: (luma(95), luma(234)),
  proposed: (rgb("#1a56b0"), rgb("#e6effb")),
  accepted: (rgb("#0e6b7a"), rgb("#e0f3f6")),
  todo: (rgb("#44506b"), rgb("#eceff5")),
  wip: (rgb("#a97b00"), rgb("#fbf1d6")),
  doing: (rgb("#a97b00"), rgb("#fbf1d6")),
  blocked: (rgb("#b3453f"), rgb("#fbe6e4")),
  "in-review": (rgb("#6b3fa0"), rgb("#f0e9fb")),
  done: (rgb("#2f7d4f"), rgb("#e2f3e8")),
  cancelled: (rgb("#6b7280"), rgb("#eceef1")),
  failed: (rgb("#b3453f"), rgb("#fbe6e4")),
)

// Scalar → plain string (bools read as yes/no; numbers and strings pass through).
#let _scalar(v) = if type(v) == bool { if v { "yes" } else { "no" } } else { str(v) }

// A framework value is a `(framework, inputs)` dict; a raw priority is an int/float.
#let _framework-name(v) = if type(v) == dictionary and "framework" in v { v.framework } else { none }

// Join a framework's inputs (`impact 6 · confidence 0.6 · ease 4`), skipping none.
#let _fmt-inputs(inputs) = {
  if type(inputs) != dictionary { return none }
  let parts = ()
  for (k, v) in inputs {
    if v != none { parts.push(k + " " + _scalar(v)) }
  }
  if parts.len() == 0 { none } else { parts.join(" · ") }
}

// Status pill (name only) and the meta-grid detail (name + date), each or none.
#let _status-pill(st) = {
  if st == none { return none }
  let name = st.at("name", default: none)
  if name == none { return none }
  let style = _status-styles.at(name, default: (luma(80), luma(230)))
  _pill(name, fg: style.at(0), bg: style.at(1))
}
#let _status-detail(st) = {
  if st == none { return none }
  let name = st.at("name", default: none)
  if name == none { return none }
  let date = st.at("date", default: none)
  if type(date) == datetime { name + " · " + date.display("[year]-[month]-[day]") } else { name }
}

// Priority pill: the framework name (ICE) or the raw score (P8); the meta detail
// carries a framework's inputs, and none for a raw int (the pill already shows it).
#let _priority-pill(v) = {
  if v == none { return none }
  let fw = _framework-name(v)
  if fw != none { _pill(upper(fw), fg: rgb("#44506b"), bg: rgb("#eceff5")) } else if type(v) == int or type(v) == float { _pill("P" + str(v), fg: rgb("#44506b"), bg: rgb("#eceff5")) } else { none }
}
#let _priority-detail(v) = {
  if _framework-name(v) != none { _fmt-inputs(v.at("inputs", default: (:))) } else { none }
}

// A numeric field (difficulty) as text: `ice · impact 6 …` for a framework, the
// raw value otherwise.
#let _num-field(v) = {
  if v == none { return none }
  let fw = _framework-name(v)
  if fw != none {
    let d = _fmt-inputs(v.at("inputs", default: (:)))
    if d == none { fw } else { fw + " · " + d }
  } else if type(v) == int or type(v) == float { str(v) } else { none }
}

// A kind pill, if set.
#let _kind-pill(k) = if type(k) == str and k != "" { _pill(k) } else { none }

// The estimate unit dict (`(days: 1, hours: 2)` → `1 day, 2 hours`), authored
// order preserved, unit singularised at one.
#let _fmt-estimate(est) = {
  if type(est) != dictionary or est.len() == 0 { return none }
  let parts = ()
  for (unit, val) in est {
    let u = if val == 1 and unit.ends-with("s") { unit.slice(0, -1) } else { unit }
    parts.push(_scalar(val) + " " + u)
  }
  parts.join(", ")
}

// A datetime → `YYYY-MM-DD`; a bare string passes through; none stays none.
#let _fmt-date(d) = {
  if d == none { none } else if type(d) == datetime { d.display("[year]-[month]-[day]") } else { str(d) }
}

// The bare stem of a link target (`../sub/other.typ` → `other`).
#let _stem(path) = {
  let base = path.split("/").last()
  if base.ends-with(".typ") { base.slice(0, -4) } else { base }
}

// A link array (depends-on and the other edges) → comma-joined clickable stems.
// Each entry is a builtin `link("stem.typ")`; harvest reads its URL, so here we
// pull the same `dest` and render the stem, staying clickable in the PDF.
#let _links(arr) = {
  if type(arr) != array or arr.len() == 0 { return none }
  let chips = ()
  for lnk in arr {
    let dest = if type(lnk) == content { lnk.at("dest", default: none) } else { none }
    if type(dest) == str { chips.push(link(dest, raw(_stem(dest)))) } else { chips.push(lnk) }
  }
  chips.join(", ")
}

// A tag chip, and the authored tags as a wrapping row of chips.
#let _tag-chip(t) = box(
  fill: luma(244),
  stroke: 0.5pt + luma(212),
  inset: (x: 0.45em, y: 0.2em),
  radius: 4pt,
  text(size: 0.72em, fill: luma(90), raw(t)),
)
#let _tags-row(tags) = {
  if type(tags) != array or tags.len() == 0 { return none }
  tags.map(_tag-chip).join(h(0.35em))
}

// A label→value metadata grid from the set rows (none-valued rows already dropped).
#let _meta(rows) = {
  if rows.len() == 0 { return none }
  let cells = ()
  for (label, value) in rows {
    cells.push(text(fill: _muted, size: 0.82em, raw(label)))
    cells.push(text(size: 0.92em, value))
  }
  grid(columns: (auto, 1fr), column-gutter: 1.1em, row-gutter: 0.55em, ..cells)
}

// Lay the whole card out. Set rules come first — before the marker — so page 1
// carries the card page settings rather than the document default.
#let _render-task(m, marker, body) = {
  set page(width: 15.5cm, height: auto, margin: (x: 1.6cm, top: 1.5cm, bottom: 1.7cm))
  set text(size: 10.5pt)
  set par(leading: 0.62em)
  set heading(numbering: none)
  show heading: it => block(above: 1em, below: 0.55em, text(size: 1.05em, weight: "bold", it.body))

  marker

  let status = m.at("status", default: none)
  let pills = (
    _status-pill(status),
    _priority-pill(m.at("priority", default: none)),
    _kind-pill(m.at("kind", default: none)),
  ).filter(p => p != none)
  if pills.len() > 0 { block(below: 0.5em, pills.join(h(0.4em))) }

  let title = m.at("title", default: "")
  if title != none and title != "" {
    block(below: 0.35em, text(size: 1.55em, weight: "bold", title))
  }

  let note = if status == none { none } else { status.at("note", default: none) }
  if note != none { block(below: 0.6em, text(fill: _muted, style: "italic", note)) }

  let rows = (
    ("status", _status-detail(status)),
    ("priority", _priority-detail(m.at("priority", default: none))),
    ("difficulty", _num-field(m.at("difficulty", default: none))),
    ("estimate", _fmt-estimate(m.at("estimate", default: (:)))),
    ("due", _fmt-date(m.at("due", default: none))),
    ("start", _fmt-date(m.at("start", default: none))),
    ("reviewed", _fmt-date(m.at("reviewed", default: none))),
    ("depends-on", _links(m.at("depends-on", default: ()))),
    ("relates-to", _links(m.at("relates-to", default: ()))),
    ("supersedes", _links(m.at("supersedes", default: ()))),
    ("duplicates", _links(m.at("duplicates", default: ()))),
    ("id", if type(m.at("id", default: none)) == str and m.at("id", default: "") != "" { raw(m.id) } else { none }),
  ).filter(row => row.at(1) != none)
  let meta = _meta(rows)
  if meta != none { block(below: 0.7em, meta) }

  let tags = _tags-row(m.at("tags", default: ()))
  if tags != none { block(below: 0.8em, tags) }

  block(below: 0.85em, line(length: 100%, stroke: 0.5pt + _rule))

  body
}

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
  let marker = [#metadata((
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
  _render-task(m, marker, body)
}
