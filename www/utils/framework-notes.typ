// What each built-in priority framework's score means, keyed by framework
// name, and the renderers /priority/ builds its tables from.
//
// Everything else on those tables comes from www/generated/frameworks.json,
// which www/generate-facts.sh derives from crates/mindtape/src/frameworks.toml
// and defaults.toml: the framework list, each input's name, type and bounds,
// the score formula or categories, the shipped bands and difficulty anchors.
// So a note here never restates a bound or a formula. `just check-doc-facts`
// fails on a framework in frameworks.json with no note here.
//
// The renderers take the fact as an argument instead of reading it, because
// the check-doc-facts driver imports this file to read the note keys, and an
// import that read www/generated/ would race the docs::check arm that deletes
// it.

#let framework-notes = (
  "ice": [
    Impact and ease are guesses, and confidence says how far to trust them.
    Confidence multiplies, so a shaky guess loses score in proportion to how
    shaky it is.
  ],
  "rice": [
    ICE with reach in front and effort below. A small change that reaches
    every user can outrank a large one that reaches few.
  ],
  "wsjf": [
    Weighted shortest job first: value per unit of size. Of two equally
    valuable tasks, the smaller one ranks higher.
  ],
  "eisenhower": [
    Urgency and importance are independent, so their four combinations are
    four categories. The category is the answer, and its score only places it
    on the scale.
  ],
  "moscow": [
    A commitment level: `must` blocks the release and `wont` records a
    refusal. It takes one enum, because there is nothing to compute.
  ],
)

// An input's domain: an enum's values, a number's bounds, or the bare type
// for a bool.
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

// A framework has a score formula or a list of categories, never both
// (www/frameworks.jq refuses both). A null `score` means categories.
#let _scoring(f) = {
  if f.score != none {
    raw(f.score)
  } else {
    f.categories.map(c => [#c.glyph #raw(c.label)]).join(", ")
  }
}

// www/generate-site-pages.sh compiles every page before the doc-facts gate
// runs, so a framework with no note fails here first. The panic names the
// fix, because a bare `.at()` would fail with "dictionary does not contain
// key".
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

#let scale-span(scale) = [#str(scale.min) to #str(scale.max)]

// The shipped priority bands, one row per `[[tasks.priority.bands]]` entry.
#let bands-table(fact) = table(
  columns: 4,
  table.header([`label`], [`min`–`max`], [`glyph`], [`color`]),
  ..fact
    .scales
    .priority
    .bands
    .map(b => (
      raw(b.label),
      [#str(b.min)–#str(b.max)],
      b.glyph,
      raw(b.color),
    ))
    .flatten(),
)

// The shipped difficulty anchors, one row per `[[tasks.difficulty.anchors]]`
// entry.
#let anchors-table(fact) = table(
  columns: 4,
  table.header([`value`], [`label`], [`glyph`], [`color`]),
  ..fact
    .scales
    .difficulty
    .anchors
    .map(a => (str(a.value), raw(a.label), a.glyph, raw(a.color)))
    .flatten(),
)
