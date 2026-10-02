// Hand-written prose per baked-in priority framework, keyed by framework
// name, plus the renderers /priority/ builds its tables from. Everything
// mechanical — the framework list, each one's input names, types and bounds,
// its score formula or category labels, and the shipped priority bands and
// difficulty anchors — comes from www/generated/frameworks.json
// (www/generate-facts.sh, reading crates/mindtape/src/frameworks.toml and
// defaults.toml). This dict carries only what no source declares: what the
// formula means. An entry must never restate a bound or a formula, since the
// page renders both alongside, derived. `just check-doc-facts` fails naming
// any frameworks.json framework with no matching key here, so a sixth
// framework cannot ship unexplained.
//
// The renderers take the fact as an argument rather than reading
// www/generated/ themselves, the way crate-notes.typ and dsl-field-notes.typ
// read nothing generated: the check-doc-facts driver imports this file to
// query the dict's keys, and a file that read a generated artifact at import
// time would make that driver a reader of a directory the parallel
// docs::check arm `rm -rf`s. The page holds the read; this file holds the
// rendering.
//
// Refs: truth/tasks/derive-the-built-in-priority-tables.typ

#let framework-notes = (
  "ice": [
    Impact and ease are the two guesses, confidence is how much you trust
    them — and it multiplies rather than subtracts, so a shaky estimate is
    discounted in proportion to itself instead of by a flat penalty.
  ],
  "rice": [
    ICE with reach in front and effort underneath: the same product of
    guesses, divided by what it costs, so a shallow change that touches
    everyone can outrank a deep one that touches few.
  ],
  "wsjf": [
    Weighted shortest job first: value per unit of size, so the queue orders
    by what finishing soonest is worth rather than by what is worth most.
    Small and valuable beats large and valuable.
  ],
  "eisenhower": [
    Urgency and importance are independent axes, so their combinations are
    four different answers rather than four points on one line. The quadrant
    is the verdict; its score only places that verdict on the scale.
  ],
  "moscow": [
    A commitment level, not a measurement: `must` is a release blocker and
    `wont` is a refusal on the record. That is why it takes one enum rather
    than numbers — there is nothing here to compute.
  ],
)

// An input's declared domain: the enum's values, a number's bounds, or the
// bare type for a bool, which has neither and needs neither.
#let _domain(i) = {
  if i.values != none {
    [#i.type #i.values.map(v => raw(v)).join(" / ")]
  } else if i.min != none {
    [#i.type #str(i.min)–#str(i.max)]
  } else {
    [#i.type]
  }
}

#let _inputs(f) = (
  f.inputs.map(i => [#raw(i.name) #_domain(i)]).join(linebreak())
)

// A framework either scores a formula or classifies into categories — never
// both, which www/frameworks.jq refuses at generation time. So a null `score`
// means this one is categorical, and the cell renders the labels
// it classifies into instead of a formula.
#let _scoring(f) = {
  if f.score != none {
    raw(f.score)
  } else {
    f.categories.map(c => [#c.glyph #raw(c.label)]).join(", ")
  }
}

// The doc-facts gate catches a framework with no prose, but the page compile
// reaches this line first, because www/generate-site-pages.sh compiles every
// page before that gate's set-difference arm runs. An unguarded `.at()` would
// report the miss as a bare "dictionary does not contain key" under
// generate-site-pages.sh's own "no <page-meta> block". So the miss is named
// here, with the fix in the message. The message names no recipe on
// purpose: it is a runtime string, so it cannot go through the generated
// `just` module, and importing that module here would make the gate's own
// key-query driver a reader of www/generated/ (see this file's header).
#let _note(name) = {
  if name not in framework-notes {
    panic(
      "framework-notes.typ: framework `"
        + name
        + "` has no prose entry in this file; add one",
    )
  }
  framework-notes.at(name)
}

#let framework-table(fact) = table(
  columns: 3,
  table.header([Framework], [Inputs], [Scoring]),
  ..fact
    .frameworks
    .map(f => (
      raw(f.name),
      _inputs(f),
      [#_scoring(f) #parbreak() #_note(f.name)],
    ))
    .flatten(),
)

// A representative value for one input, so a rendered example call is
// in bounds by construction and a re-ranged input moves the example with it.
// The midpoint of the declared range, rounded to an integer when the span is
// wide enough to hold one strictly inside it and to one decimal when it is
// not — `confidence`'s 0–1 has no integer between its ends, and an example
// pinned to a bound would read as a limit rather than as a value.
#let _value(i) = {
  if i.values != none {
    "\"" + i.values.first() + "\""
  } else if i.min == none {
    "true"
  } else if i.max - i.min >= 2 {
    str(calc.round((i.min + i.max) / 2))
  } else {
    str(calc.round((i.min + i.max) / 2, digits: 1))
  }
}

#let framework-call(fact, name) = {
  let f = fact.frameworks.find(f => f.name == name)
  if f == none {
    panic(
      "framework-notes.typ: no `"
        + name
        + "` framework in www/generated/frameworks.json",
    )
  }
  raw(
    "priority: "
      + f.name
      + "("
      + f.inputs.map(i => i.name + ": " + _value(i)).join(", ")
      + "),",
    lang: "typ",
    block: true,
  )
}

#let scale-span(scale) = [#str(scale.min) to #str(scale.max)]

// The `[tasks.priority]` example block, rendered from the shipped values
// rather than typed beside them: it sits two lines above a derived sentence
// stating the same bands, and a re-ranged scale that moved one and not the
// other would put the page in contradiction with itself. The `#` comments and
// the `required` line are hand-written on purpose: they name keys the
// scale accepts, and `required` has no default to read at all.
#let scale-config(fact) = {
  let s = fact.scales.priority
  let b = s.bands.last()
  raw(
    "[tasks.priority]\n"
      + "kind = \""
      + s.kind
      + "\"            # int | real\n"
      + "min  = "
      + str(s.min)
      + "\n"
      + "max  = "
      + str(s.max)
      + "\n"
      + "required = \"!terminal\"  # true | false | a filter expression\n\n"
      + "[[tasks.priority.bands]]\n"
      + "label = \""
      + b.label
      + "\"\nmin   = "
      + str(b.min)
      + "\nmax   = "
      + str(b.max)
      + "\nglyph = \""
      + b.glyph
      + "\"\ncolor = \""
      + b.color
      + "\"",
    lang: "toml",
    block: true,
  )
}

// The bare-integer example, pinned to the lowest value in the top band: it
// sits inside the band it names by construction, so a re-ranged scale moves
// the value and the label it claims together rather than one of them.
#let band-example(fact) = {
  let b = fact.scales.priority.bands.last()
  raw(
    "priority: " + str(b.min) + ",       // → band \"" + b.label + "\"",
    lang: "typ",
    block: true,
  )
}

#let priority-bands(fact) = {
  let bands = fact.scales.priority.bands
  bands
    .map(b => [#raw(b.label) #b.glyph at #str(b.min)–#str(b.max)])
    .join(", ", last: " and ")
}

#let difficulty-anchors(fact) = {
  let anchors = fact.scales.difficulty.anchors
  anchors
    .map(a => [#raw(a.label) #a.glyph at #str(a.value)])
    .join(", ", last: " and ")
}
