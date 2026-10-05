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
// Same discipline as `edge` below: a second block, a named argument or a bare
// string would land in the sink and be dropped on the floor, so each is an
// error instead of a note that quietly never arrives.
#let _mk(name) = (y, m, d, ..note) => {
  let pos = note.pos()
  assert(
    note.named().len() == 0,
    message: "`"
      + name
      + "()` takes no named arguments; the note is a trailing block, e.g. `"
      + name
      + "(2026, 6, 14)[note]`",
  )
  let count = str(pos.len())
  assert(
    pos.len() <= 1,
    message: "`" + name + "()` takes at most one note block, got " + count,
  )
  assert(
    pos.len() == 0 or type(pos.at(0, default: none)) == content,
    message: "`"
      + name
      + "()` note must be a content block, e.g. `"
      + name
      + "(2026, 6, 14)[note]`, got "
      + str(type(pos.at(0, default: none))),
  )
  status(name, y, m, d, note: pos.at(0, default: none))
}
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
#let framework(name, ..inputs) = {
  assert(
    type(name) == str and name != "",
    message: "framework() name must be a string, got " + str(type(name)),
  )
  assert(
    inputs.pos().len() == 0,
    message: "`"
      + name
      + "()` inputs must be named, e.g. `ice(impact: 8)`; got "
      + str(inputs.pos().len())
      + " positional",
  )
  (framework: name, inputs: inputs.named())
}
#let ice(impact: none, confidence: none, ease: none) = framework(
  "ice",
  impact: impact,
  confidence: confidence,
  ease: ease,
)
#let rice(
  reach: none,
  impact: none,
  confidence: none,
  effort: none,
) = framework(
  "rice",
  reach: reach,
  impact: impact,
  confidence: confidence,
  effort: effort,
)
#let wsjf(cost_of_delay: none, job_size: none) = framework(
  "wsjf",
  cost_of_delay: cost_of_delay,
  job_size: job_size,
)
#let eisenhower(urgent: false, important: false) = framework(
  "eisenhower",
  urgent: urgent,
  important: important,
)
#let moscow(level) = framework("moscow", level: level)

// Typed relation edges. Each helper packages one or more plain stem-string
// targets into edges of a fixed kind — `depends-on("a.typ", "b.typ")` yields
// two edges — and the whole call already IS the flat array `links:` expects
// (no wrapping tuple needed); combine several kinds with `+`:
// `links: depends-on("a.typ") + related("b.typ")[why]`. A trailing content
// block is the note, uniform with the status `_mk` idiom. Helpers take plain
// strings, never `link(...)`: the Rust layer resolves the string directly,
// and the render re-wraps it in `link()` for a clickable PDF. `edge` is the
// generic escape hatch for a project's config-declared custom kind; the six
// named helpers below are pure sugar over it.
//
// Targets and the note are told apart by type, so every argument that is
// neither is an error rather than a silent drop. That asymmetry is the point:
// `depends-on(link("a.typ"))` passes content, content reads as the note, and
// the call yields zero edges — a loss nothing downstream can see, since the
// marker then carries an empty `links` and `mt check` has no dependency left
// to complain about (the task even reports ready). Anything that DOES reach
// the marker — an unknown kind, an unresolvable stem, a missing required note
// — stays the Rust layer's to report, where the config lives. The prelude
// asserts only where the mistake destroys the value before the marker is
// written. Assert messages evaluate eagerly, so each one stays total on the
// passing path too (hence `at(0, default: none)` over `at(0)`).
#let edge(kind, ..args) = {
  assert(
    type(kind) == str and kind != "",
    message: "edge() kind must be a non-empty string, got " + str(type(kind)),
  )
  assert(
    args.named().len() == 0,
    message: "`"
      + kind
      + "()` takes no named arguments; targets are positional strings and the "
      + "note is a trailing block",
  )
  let pos = args.pos()
  let targets = pos.filter(p => type(p) == str)
  let notes = pos.filter(p => type(p) == content)
  let stray = pos.filter(p => type(p) != str and type(p) != content)
  assert(
    stray.len() == 0,
    message: "`"
      + kind
      + "()` target must be a plain stem string like \"other.typ\", got "
      + str(type(stray.at(0, default: none))),
  )
  let count = str(notes.len())
  assert(
    notes.len() <= 1,
    message: "`" + kind + "()` takes at most one note block, got " + count,
  )
  // Bound before the assert, not inlined into `message`: in a code block a
  // leading `+` on a continuation line parses as a unary plus, not as
  // concatenation, so the string stays on one line here.
  let hint = if notes.len() > 0 {
    "; `link(...)` is content, so it read as the note, not a target"
  } else { "" }
  assert(
    targets.len() > 0,
    message: "`"
      + kind
      + "()` needs at least one target, e.g. `"
      + kind
      + "(\"other.typ\")`"
      + hint,
  )
  let note = notes.at(0, default: none)
  targets.map(t => (task: t, kind: kind, note: note))
}
#let depends-on(..args) = edge("depends-on", ..args)
#let parent(..args) = edge("parent", ..args)
#let recurred-from(..args) = edge("recurred-from", ..args)
#let supersedes(..args) = edge("supersedes", ..args)
#let duplicates(..args) = edge("duplicates", ..args)
#let related(..args) = edge("related", ..args)

// Normalize `links:` to the flat edge array the marker promises. Each helper
// already RETURNS an array, so `+` between two of them is array concatenation
// — which means the array spelling an author reaches for first,
// `links: (depends-on("a.typ"), depends-on("b.typ"))`, is an array of arrays,
// and every entry in it is an array the Rust layer's per-entry parse cannot
// read. Nesting has exactly one possible meaning here (concatenation, the same
// thing `+` spells), so flattening it loses nothing and leaves nothing to
// guess — unlike the `link(...)` target above, where `content` genuinely meant
// two things and an assert was the only honest answer. This is normalization,
// not validation: it fires where the Rust layer cannot see, since the printed
// card below groups the SAME value by kind and drops a nested entry just as
// silently. A non-array `links:` (`links: "a.typ"`) passes through untouched,
// so the Rust layer still reports it against the expected shape rather than
// Typst failing on a method a string does not have.
#let _flat-links(links) = if type(links) == array { links.flatten() } else {
  links
}

// Weekday constants for `every(..., on: (...))`'s weekday set — plain string
// labels; the Rust layer maps them to real weekdays, no date math here.
#let mon = "mon"
#let tue = "tue"
#let wed = "wed"
#let thu = "thu"
#let fri = "fri"
#let sat = "sat"
#let sun = "sun"

// Recurrence. A task advances to its next occurrence only via `mt recur`,
// never as a side effect of a read (see command-semantics-cqrs). `every()`
// shapes one period unit (days/weeks/months/years) — optionally narrowed to a
// weekday set via `on:` — plus the policy riding along with it: `anchor`
// ("completion", the default, or "due"), the series stop conditions
// `until`/`count`, and `materialize` ("spawn"/"mutate"). No math lives here,
// same discipline as the priority frameworks: an omitted policy arg stays
// `none` and the Rust layer resolves it from `[tasks.recur]`, validates the
// rule, and computes the next date.
#let every(
  days: none,
  weeks: none,
  months: none,
  years: none,
  on: none,
  anchor: none,
  until: none,
  count: none,
  materialize: none,
) = {
  let units = (
    (days, "days"),
    (weeks, "weeks"),
    (months, "months"),
    (years, "years"),
  ).filter(u => u.at(0) != none)
  assert(
    units.len() == 1,
    message: "every() takes exactly one of days/weeks/months/years",
  )
  let (value, unit) = units.at(0)
  (
    rule: (kind: "period", unit: unit, value: value, on: on),
    anchor: anchor,
    until: until,
    count: count,
    materialize: materialize,
  )
}

// The rrule escape hatch (its own task, rrule-escape-hatch): shapes the SAME
// policy dict, a raw rule string standing in for the period. Expansion is a
// later feature; until then the Rust layer parses and gates this rule kind
// behind `[tasks.recur] rrule` rather than computing a next date from it.
#let rrule(rule, anchor: none, until: none, count: none, materialize: none) = (
  rule: (kind: "rrule", raw: rule),
  anchor: anchor,
  until: until,
  count: count,
  materialize: materialize,
)

// Template authoring helper. `mt add` discovers `slot(...)` calls on the
// parse plane and replaces each with a filled literal, so a filed task holds
// plain values and never calls `slot`. At eval time a slot returns its `default`,
// keeping a template a previewable, valid task. `kind` drives how `add` renders a
// filled value ("string", "int", "float", "bool", "date", "list", "text",
// "choice", "multichoice"); `prompt`/`required`/`choices` steer the fill UI. `env` names an
// environment variable `add` reads at fill time (missing/unreadable variables
// fall through to the next source); with `env: none` every slot still fills
// from `MINDTAPE_ADD_<NAME>`. `example` is a sample value `--describe` shows
// and `mt check` fills; it is never filed and is ignored here. Args must be
// literals — `add` reads them without evaluating the template.
#let slot(
  name,
  default: none,
  example: none,
  kind: "string",
  prompt: none,
  required: true,
  choices: none,
  env: none,
) = default

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

// Scalar → plain string (bools read as yes/no; numbers and strings pass
// through). TOTAL by construction — anything else renders as its `repr` rather
// than failing `str()`. The render layer owes nobody validation: a wrong-typed
// value must reach the marker so the Rust layer can report it by field and
// position (`tags[1]`, `estimate.days`), which an eval error here naming
// neither the field nor the task cannot do. Every accessor below is defensive
// for the same reason.
#let _scalar(v) = if type(v) == bool { if v { "yes" } else { "no" } } else if (
  type(v) == str
) { v } else if type(v) == int or type(v) == float { str(v) } else { repr(v) }

// A framework value is a `(framework, inputs)` dict; a raw priority is an int/float.
#let _framework-name(v) = if type(v) == dictionary and "framework" in v {
  v.framework
} else { none }

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
  if type(st) != dictionary { return none }
  let name = st.at("name", default: none)
  if name == none { return none }
  let style = _status-styles.at(name, default: (luma(80), luma(230)))
  _pill(name, fg: style.at(0), bg: style.at(1))
}
#let _status-detail(st) = {
  if type(st) != dictionary { return none }
  let name = st.at("name", default: none)
  if name == none { return none }
  let date = st.at("date", default: none)
  if type(date) == datetime {
    name + " · " + date.display("[year]-[month]-[day]")
  } else { name }
}

// Priority pill: the framework name (ICE) or the raw score (P8); the meta detail
// carries a framework's inputs, and none for a raw int (the pill already shows it).
#let _priority-pill(v) = {
  if v == none { return none }
  let fw = _framework-name(v)
  if fw != none {
    _pill(upper(fw), fg: rgb("#44506b"), bg: rgb("#eceff5"))
  } else if type(v) == int or type(v) == float {
    _pill("P" + str(v), fg: rgb("#44506b"), bg: rgb("#eceff5"))
  } else { none }
}
#let _priority-detail(v) = {
  if _framework-name(v) != none {
    _fmt-inputs(v.at("inputs", default: (:)))
  } else { none }
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
    let u = if val == 1 and unit.ends-with("s") { unit.slice(0, -1) } else {
      unit
    }
    parts.push(_scalar(val) + " " + u)
  }
  parts.join(", ")
}

// The rule half of `_fmt-recur`: a period (with its optional weekday set) or
// an `rrule(...)` raw string. Defined first: Typst `#let` bindings capture
// the scope at definition time (no hoisting), so `_fmt-recur` below must be
// able to see this name already bound when IT is defined, not just when
// called.
#let _fmt-recur-rule(rule) = {
  let kind = rule.at("kind", default: none)
  if kind == "rrule" {
    let raw = rule.at("raw", default: none)
    return if type(raw) == str { "rrule: " + raw } else { none }
  }
  if kind != "period" { return none }
  let unit = rule.at("unit", default: none)
  let value = rule.at("value", default: none)
  let core = if type(value) == int and type(unit) == str {
    (
      "every "
        + _scalar(value)
        + " "
        + (if value == 1 { unit.slice(0, -1) } else { unit })
    )
  } else { none }
  let on = rule.at("on", default: none)
  if type(on) == array and on.len() > 0 {
    let days = on.map(_scalar).join(", ")
    if core == none { "on " + days } else { core + " on " + days }
  } else { core }
}

// Recurrence as compact text (`every 1 week`, `every 3 days on mon, wed, fri`,
// `rrule: FREQ=MONTHLY;BYDAY=1MO`), plus `anchor …`/`until …`/`count …` when
// set. `none` when `recur` is unset or malformed — the rendered card skips
// the row entirely rather than showing a blank one.
#let _fmt-recur(r) = {
  if type(r) != dictionary { return none }
  let rule = r.at("rule", default: none)
  if type(rule) != dictionary { return none }
  let base = _fmt-recur-rule(rule)
  if base == none { return none }
  let anchor = r.at("anchor", default: none)
  let until = r.at("until", default: none)
  let count = r.at("count", default: none)
  let parts = (base,)
  if type(anchor) == str { parts.push("anchor " + anchor) }
  if type(until) == datetime {
    parts.push("until " + until.display("[year]-[month]-[day]"))
  }
  if type(count) == int { parts.push("count " + str(count)) }
  parts.join(" · ")
}

// A datetime → `YYYY-MM-DD`; a bare string passes through; none stays none.
#let _fmt-date(d) = {
  if d == none { none } else if type(d) == datetime {
    d.display("[year]-[month]-[day]")
  } else { _scalar(d) }
}

// The bare stem of a link target (`../sub/other.typ` → `other`).
#let _stem(path) = {
  let base = path.split("/").last()
  if base.ends-with(".typ") { base.slice(0, -4) } else { base }
}

// One edge dict → a clickable stem chip (re-wraps the plain `task` string in
// `link()`, since the marker now carries the string directly — see `edge`).
// `none` for a malformed entry (no string `task`); the Rust layer flags those.
#let _edge-chip(e) = {
  let dest = if type(e) == dictionary { e.at("task", default: none) } else {
    none
  }
  if type(dest) == str { link(dest, raw(_stem(dest))) } else { none }
}

// Comma-joined chips for one kind's edges.
#let _edges-row(edges) = {
  if type(edges) != array or edges.len() == 0 { return none }
  edges.map(_edge-chip).filter(c => c != none).join(", ")
}

// Group the flat `links` array by kind, first-seen order, as (kind, edges)
// pairs — a malformed entry (no string `kind`, e.g. a bare-string edge with no
// helper) is dropped here; the Rust layer validates and reports those.
#let _links-by-kind(arr) = {
  if type(arr) != array or arr.len() == 0 { return () }
  let order = ()
  let groups = (:)
  for e in arr {
    let k = if type(e) == dictionary { e.at("kind", default: none) } else {
      none
    }
    if type(k) != str or k == "" { continue }
    if k not in groups {
      groups.insert(k, ())
      order.push(k)
    }
    groups.insert(k, groups.at(k) + (e,))
  }
  order.map(k => (k, groups.at(k)))
}

// A tag chip, and the authored tags as a wrapping row of chips.
#let _tag-chip(t) = box(
  fill: luma(244),
  stroke: 0.5pt + luma(212),
  inset: (x: 0.45em, y: 0.2em),
  radius: 4pt,
  text(size: 0.72em, fill: luma(90), raw(_scalar(t))),
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
  set page(width: 15.5cm, height: auto, margin: (
    x: 1.6cm,
    top: 1.5cm,
    bottom: 1.7cm,
  ))
  set text(size: 10.5pt)
  set par(leading: 0.62em)
  set heading(numbering: none)
  show heading: it => block(above: 1em, below: 0.55em, text(
    size: 1.05em,
    weight: "bold",
    it.body,
  ))

  marker

  let status = m.at("status", default: none)
  let pills = (
    _status-pill(status),
    _priority-pill(m.at("priority", default: none)),
    _kind-pill(m.at("kind", default: none)),
  ).filter(p => p != none)
  if pills.len() > 0 { block(below: 0.5em, pills.join(h(0.4em))) }

  let title = m.at("title", default: "")
  if type(title) == str and title != "" {
    block(below: 0.35em, text(size: 1.55em, weight: "bold", title))
  }

  let note = if type(status) != dictionary { none } else {
    status.at("note", default: none)
  }
  if type(note) == str or type(note) == content {
    block(below: 0.6em, text(fill: _muted, style: "italic", note))
  }

  // Link rows are dynamic: one row per kind present in `links`, grouped, in
  // first-seen order — not a fixed depends-on/relates-to/supersedes/duplicates
  // set, since a project can add its own kinds.
  let link-rows = _links-by-kind(m.at("links", default: ())).map(pair => (
    pair.at(0),
    _edges-row(pair.at(1)),
  ))

  let rows = (
    (
      ("status", _status-detail(status)),
      ("priority", _priority-detail(m.at("priority", default: none))),
      ("difficulty", _num-field(m.at("difficulty", default: none))),
      (
        "assignee",
        if type(m.at("assignee", default: none)) == str
          and m.at("assignee", default: "") != "" {
          text(m.at("assignee", default: ""))
        } else { none },
      ),
      ("estimate", _fmt-estimate(m.at("estimate", default: (:)))),
      ("due", _fmt-date(m.at("due", default: none))),
      ("start", _fmt-date(m.at("start", default: none))),
      ("reviewed", _fmt-date(m.at("reviewed", default: none))),
      ("recur", _fmt-recur(m.at("recur", default: none))),
    )
      + link-rows
      + (
        (
          "id",
          if type(m.at("id", default: none)) == str
            and m.at("id", default: "") != "" { raw(m.id) } else { none },
        ),
      )
  ).filter(row => row.at(1) != none)
  let meta = _meta(rows)
  if meta != none { block(below: 0.7em, meta) }

  let tags = _tags-row(m.at("tags", default: ()))
  if tags != none { block(below: 0.8em, tags) }

  block(below: 0.85em, line(length: 100%, stroke: 0.5pt + _rule))

  body
}

// The task show-rule. Every core field is an explicit named parameter, so the
// signature IS the core-slot enumeration — no separate `_core-slots` list to
// drift from the marker builder, and an editor (tinymist) can offer completion,
// signature help and spelling feedback while a task is authored. That typed
// authoring is a main reason tasks are Typst and not YAML frontmatter.
// `..meta` stays as the extension sink only: anything reaching it is by
// definition not a core field, so the x-<ns> discipline is one assert over the
// sink rather than a membership test per key. Defaults match what the Rust
// layer reads for an unset field ("" / none / empty), keeping an omitted field
// and an explicitly-defaulted one indistinguishable downstream.
#let task(
  body,
  title: "",
  status: none,
  priority: none,
  difficulty: none,
  kind: none,
  assignee: none,
  estimate: (:),
  links: (),
  tags: (),
  due: none,
  start: none,
  reviewed: none,
  recur: none,
  id: none,
  ..meta,
) = {
  assert(
    meta.pos().len() == 0,
    message: "task takes no positional args besides its body",
  )
  let extras = (:)
  for (key, value) in meta.named() {
    assert(
      key.starts-with("x-"),
      message: "unknown task slot `"
        + key
        + "`; tool extensions must use an x-<ns> slot",
    )
    extras.insert(key, value)
  }
  let fields = (
    title: title,
    status: status,
    priority: priority,
    difficulty: difficulty,
    kind: kind,
    assignee: assignee,
    estimate: estimate,
    links: _flat-links(links),
    tags: tags,
    due: due,
    start: start,
    reviewed: reviewed,
    recur: recur,
    id: id,
  )
  let marker = [#metadata((
    marker: "mindtape.task",
    fields: fields,
    extras: extras,
  )) <mindtape>]
  _render-task(fields, marker, body)
}
