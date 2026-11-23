// The goal DSL this example's task bodies write, and the check that reads it.
//
// `goal(...)` declares one value, and goals.typ keeps this project's registry.
// `Helps`, `Hinders` and `Neutral` each emit one metadata marker, under the
// `goal` discriminator, so a check reads a verdict as a value rather than as
// prose. `check-goal-over(goals, markers)` is the `[[tasks.rules]] check`
// entry point: it returns one message per problem.

// docs:slice goal
// The name a task writes: the kebab of the goal's infinitive, so "answer fast"
// is `answer-fast`. The title keeps its capitals; the name never does.
#let _kebab(to) = {
  assert(
    type(to) == str and to != "",
    message: "goal() takes its infinitive as a non-empty string",
  )
  lower(to).split(" ").filter(word => word != "").join("-")
}

// A goal value: `to` is the infinitive a task writes after `to:`, `from` the
// gerund it writes after `from:`, and `title` the sentence a render shows.
#let goal(to: none, from: none, title: none) = (
  name: _kebab(to),
  to: to,
  from: from,
  title: title,
)
// docs:end

// The metadata a verdict emits, with `d`'s keys beside the discriminator.
#let _marker(d) = [#metadata((marker: "goal") + d)]

// The name of the goal value a verdict rates. A missing `to:` names its own
// fix instead of filing a verdict with no goal.
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
    message: label + "() takes a goal value from goals.typ, not " + repr(value),
  )
  value.name
}

// docs:slice verdicts
// A goal the task helps, and why. The reason is the trailing content block.
#let Helps(reason, to: none) = (
  _marker((
    goal: _rated(to, "Helps", "to"),
    effect: "helps",
    reason: reason,
  ))
    + reason
)

// A goal the task hinders, and from which direction.
#let Hinders(reason, from: none) = (
  _marker((
    goal: _rated(from, "Hinders", "from"),
    effect: "hinders",
    reason: reason,
  ))
    + reason
)

// The goals the task leaves alone, one neutral marker each. A neutral verdict
// needs no reason.
#let Neutral(..goals) = {
  let markers = ()
  let names = ()
  for value in goals.pos() {
    assert(
      type(value) == dictionary and "name" in value,
      message: "Neutral() takes goal values from goals.typ, not " + repr(value),
    )
    markers.push(_marker((
      goal: value.name,
      effect: "neutral",
      reason: none,
    )))
    names.push(value.name)
  }
  if names.len() == 0 { return markers.join() }
  markers.join() + [Neutral: #names.join(", ")]
}
// docs:end

// docs:slice check
// `[[tasks.rules]] check` entry point for `field = "markers"`: every goal once,
// with a reason on a helps or hinders, and at least one helps.
#let check-goal-over(goals, markers) = {
  let known = goals.keys()
  let messages = ()
  let rated = ()
  let helped = false
  let values = markers
    .map(marker => marker.at("value", default: none))
    .filter(value => (
      type(value) == dictionary and value.at("marker", default: none) == "goal"
    ))
  for value in values {
    let name = value.at("goal", default: none)
    if name not in known {
      messages.push(
        "unknown goal `" + str(name) + "`; known: " + known.join(", "),
      )
      continue
    }
    if name in rated { messages.push("goal `" + name + "` is rated twice") }
    rated.push(name)
    let effect = value.at("effect", default: none)
    if effect not in ("helps", "hinders", "neutral") {
      messages.push(
        "goal `"
          + name
          + "` has unknown effect `"
          + str(effect)
          + "`; use `helps`, `hinders` or `neutral`",
      )
      continue
    }
    if effect == "helps" { helped = true }
    if effect != "neutral" {
      let reason = str(value.at("reason", default: "")).trim()
      if reason.split().filter(word => word != "").len() < 3 {
        messages.push(
          "goal `"
            + name
            + "` says "
            + effect
            + " but not why; write the reason in a sentence",
        )
      }
    }
  }
  for name in known {
    if name not in rated { messages.push("goal `" + name + "` is not rated") }
  }
  if not helped {
    messages.push("the task helps no goal; if nothing justifies it, cancel it")
  }
  messages
}
// docs:end
