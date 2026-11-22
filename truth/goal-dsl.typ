// The goal DSL a task body writes: one metadata marker per goal verdict.
//
// A task rates every goal its project declares. `Helps(adopter, to: <goal>)[why]`
// and `Hinders(maintainer, from: <goal>)[why]` name the audience a verdict
// serves and the goal value it rates; `Neutral(<goal>, ..)` rates goals neutral
// and carries no reason. Each call emits a metadata marker, and the project's
// rule reads them through the reserved `markers` check input, so a verdict
// reaches a check as a value rather than as prose.
//
// A missing `to:` or `from:` is a Typst error, not a marker with no goal: a
// verdict always names the goal it rates. A project declares its goals with
// `goal(...)`, fixes the audience in a wrapper of its own, and re-exports the
// values (this repo's `truth/goal-calls.typ` does that for its two sets).

// The audience a goal serves: `adopter` for the people a project's tasks are
// for, `maintainer` for the people who build the project. A registry fixes one
// of them, so the audience a task writes is a wrong-audience check.
#let adopter = "adopter"
#let maintainer = "maintainer"

// The `marker` key every goal verdict carries. It is the discriminator a check
// filters the body's markers on; every other marker is ignored.
#let goal-marker = "goal"

// The name a task writes: the kebab of the goal's infinitive, so
// `"beat Markdown"` is `beat-markdown`. The title keeps its capitals; the name
// never does.
#let _kebab(to) = {
  assert(
    type(to) == str and to != "",
    message: "goal() takes its infinitive as a non-empty string",
  )
  lower(to).split(" ").filter(word => word != "").join("-")
}

// A goal value: `to` is the infinitive a task writes after `to:`, `from` the
// gerund after `from:`, `title` the sentence a render shows, `acid-test` the
// paragraph that says what meeting the goal means, and `audience` the registry
// whose backlog it belongs to. `name` is the kebab of `to`, the one spelling a
// task uses to name the value.
#let goal(audience, to: none, from: none, title: none, acid-test: none) = {
  assert(
    audience in (adopter, maintainer),
    message: "goal() takes an audience, `adopter` or `maintainer`, not "
      + repr(audience),
  )
  (
    audience: audience,
    name: _kebab(to),
    to: to,
    from: from,
    title: title,
    acid-test: acid-test,
  )
}

// The metadata a goal verdict emits, with `d`'s keys beside the discriminator.
// A call is content, so it can stand as a list item in a task body.
#let marker(d) = [#metadata((marker: goal-marker) + d)]

// The name of the goal value a verdict rates. `label` and `key` spell the call
// and its argument in the error, so a missing `to:` names its own fix.
#let _rated(value, label, key) = {
  assert(
    value != none,
    message: label
      + "() needs `"
      + key
      + ": <goal>`; a verdict without a goal is not a verdict",
  )
  assert(
    type(value) == dictionary and "name" in value,
    message: label
      + "() takes a goal value from goal-calls.typ, not "
      + repr(value),
  )
  value.name
}

// A goal this task helps, and why. The reason is the trailing content block.
#let Helps(audience, reason, to: none) = marker((
  audience: audience,
  goal: _rated(to, "Helps", "to"),
  effect: "helps",
  reason: reason,
))

// A goal this task hinders, and from which direction. The reason is the
// trailing content block.
#let Hinders(audience, reason, from: none) = marker((
  audience: audience,
  goal: _rated(from, "Hinders", "from"),
  effect: "hinders",
  reason: reason,
))

// The goals this task leaves alone. One marker per goal, each carrying the
// goal's own audience, since one neutral call spans both registries.
#let Neutral(..goals) = {
  assert(
    goals.named().len() == 0,
    message: "Neutral() takes goal values as positional arguments",
  )
  for value in goals.pos() {
    assert(
      type(value) == dictionary and "name" in value and "audience" in value,
      message: "Neutral() takes goal values from goal-calls.typ, not "
        + repr(value),
    )
    marker((
      audience: value.audience,
      goal: value.name,
      effect: "neutral",
      reason: none,
    ))
  }
}
