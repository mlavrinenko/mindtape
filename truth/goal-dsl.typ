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
//
// This module is project Typst, not prelude Typst: the prelude keeps the
// mechanism (`field = "markers"` and the reserved input), and a project writes
// its own goal module. `check-goal-over(product, process, markers)` is the
// `[[tasks.rules]] check` entry point over the two registries a project passes.

// The audience a goal serves: `adopter` for the people a project's tasks are
// for, `maintainer` for the people who build the project. A registry fixes one
// of them, so the audience a task writes is a wrong-audience check.
#let adopter = "adopter"
#let maintainer = "maintainer"

// The `marker` key every goal verdict carries. It is the discriminator a check
// filters the body's markers on; every other marker is ignored.
#let _goal-marker = "goal"

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
#let _goal-marker-of(d) = [#metadata((marker: _goal-marker) + d)]

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
#let Helps(audience, reason, to: none) = _goal-marker-of((
  audience: audience,
  goal: _rated(to, "Helps", "to"),
  effect: "helps",
  reason: reason,
))

// A goal this task hinders, and from which direction. The reason is the
// trailing content block.
#let Hinders(audience, reason, from: none) = _goal-marker-of((
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
    _goal-marker-of((
      audience: value.audience,
      goal: value.name,
      effect: "neutral",
      reason: none,
    ))
  }
}

// Every goal name the two registries declare, in registry order.
#let _goal-names(product, process) = {
  let names = product.values().map(value => value.name)
  names + process.values().map(value => value.name)
}

// The names declared more than once across the two registries. A duplicate
// would collapse to one `known` entry and leave one goal unrated, so the check
// refuses the registry instead of merging it.
#let _duplicate-goals(product, process) = {
  let seen = ()
  let duplicates = ()
  for name in _goal-names(product, process) {
    if name in seen {
      if name not in duplicates {
        duplicates.push(name)
      }
    } else {
      seen.push(name)
    }
  }
  duplicates
}

// Every goal value the two registries declare, keyed by the name a task writes.
// Both registries are the project's own; `_duplicate-goals` has already refused
// a name either declares twice.
#let _goals-by-name(product, process) = {
  let by-name(registry) = registry
    .values()
    .map(value => (value.name, value))
    .to-dict()
  by-name(product) + by-name(process)
}

// What a verdict gets wrong, if anything: an unknown effect, a wrong audience,
// or a `Helps` or `Hinders` that gives no reason. A neutral verdict needs none.
// The effect is checked first, so a hand-written marker cannot reach the other
// checks with a verdict the DSL cannot spell.
#let _verdict-problems(value, goal) = {
  let messages = ()
  let effect = value.at("effect", default: none)
  if effect == none {
    messages.push(
      "goal `"
        + goal.name
        + "` has no effect; use `helps`, `hinders` or `neutral`",
    )
    return messages
  }
  if effect not in ("helps", "hinders", "neutral") {
    messages.push(
      "goal `"
        + goal.name
        + "` has unknown effect `"
        + str(effect)
        + "`; use `helps`, `hinders` or `neutral`",
    )
    return messages
  }
  let audience = value.at("audience", default: none)
  if audience != goal.audience {
    let fix = "#Helps(" + goal.audience + ", to: " + goal.name + ")"
    messages.push(
      "goal `"
        + goal.name
        + "` is "
        + goal.audience
        + " work, rated "
        + str(audience)
        + "; write "
        + fix,
    )
  }
  if (
    effect != "neutral" and value.at("reason", default: none) in (none, "", [ ])
  ) {
    messages.push(
      "goal `" + goal.name + "` says " + str(effect) + " but not why",
    )
  }
  messages
}

// `[[tasks.rules]] check` entry point for `field = "markers"`: every goal once,
// under its registry's audience, with a reason on a `Helps`/`Hinders` and at
// least one `Helps`. A task with no goal marker arrives as `[]` and is refused.
#let check-goal-over(product, process, markers) = {
  if type(markers) != array {
    return ("the goal check reads the body's markers; got " + repr(markers),)
  }
  let duplicates = _duplicate-goals(product, process)
  if duplicates.len() > 0 {
    return duplicates.map(name => (
      "goal `"
        + name
        + "` is declared twice; rename one `to` or split the registries"
    ))
  }
  let known = _goals-by-name(product, process)
  let messages = ()
  let rated = ()
  let helped = false
  let values = markers
    .filter(marker => type(marker) == dictionary)
    .map(marker => marker.at("value", default: none))
    .filter(value => (
      type(value) == dictionary
        and value.at("marker", default: none) == _goal-marker
    ))
  for value in values {
    let name = value.at("goal", default: none)
    if type(name) != str {
      messages.push("a goal marker names no goal")
      continue
    }
    if name in rated {
      messages.push("goal `" + name + "` is rated twice")
    }
    rated.push(name)
    let goal = known.at(name, default: none)
    if goal == none {
      messages.push(
        "unknown goal `" + name + "`; known: " + known.keys().join(", "),
      )
      continue
    }
    messages += _verdict-problems(value, goal)
    if value.at("effect", default: none) == "helps" {
      helped = true
    }
  }
  for name in known.keys() {
    if name not in rated {
      messages.push("goal `" + name + "` is not rated")
    }
  }
  if not helped {
    messages.push("the task helps no goal; if nothing justifies it, cancel it")
  }
  messages
}
