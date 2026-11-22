// The goal check every open task imports. `check-goal-over(markers)` checks a
// task body's goal markers against both goal sets, goals.typ (what Mindtape is
// for) and process-goals.typ (how this repo builds it), returning one message
// per problem instead of panicking: a `[[tasks.rules]] check` rule collects
// every string it returns as a breach, so several omissions surface together
// rather than one panic at a time. A task nothing justifies (helps no goal) is
// refused, not filed.
//
// `check-goal-over` reads the reserved `markers` rule input, the DSL in
// goal-dsl.typ that a task writes as `#Helps`/`#Hinders`/`#Neutral`. The two
// registries fix each goal's audience, which is why the check lives here rather
// than in the generic DSL.
//
// The verdict helpers come from the prelude, so an adopter writes the same
// dict a task does without copying this file; this module keeps only what is
// true of this repo.

#import "@local/mindtape:0.3.0": edge
#import "goal-dsl.typ": goal-marker
#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

// A finding points at the task that surfaced it. The kind is declared in
// `.mindtape/config.toml`; this helper is the project-side spelling of
// `edge("surfaced-by", ...)`, beside the goal check every task imports.
#let surfaced-by(..args) = edge("surfaced-by", ..args)

// Every goal of both registries, keyed by the name a task writes (the kebab of
// its infinitive). A goal's audience says which set it came from, so two
// registries answer to one namespace.
#let _goals-by-name() = {
  let by-name(registry) = registry
    .values()
    .map(value => (value.name, value))
    .to-dict()
  by-name(product-goals) + by-name(process-goals)
}

// Whether a `Helps`/`Hinders` verdict says why. A reason is markup, so an empty
// block reaches the check as the empty string, and an omitted one as `none`.
#let _says-why(reason) = reason in (none, "", [ ])

// The problems one goal verdict has, given the registry's goal value for its
// name. The caller has already rejected an unknown name.
#let _verdict-problems(value, goal) = {
  let messages = ()
  let name = goal.name
  let audience = value.at("audience", default: none)
  if audience != goal.audience {
    messages.push(
      "goal `"
        + name
        + "` is "
        + goal.audience
        + " work, rated "
        + str(audience)
        + "; write #Helps("
        + goal.audience
        + ", to: "
        + name
        + ")",
    )
  }
  let effect = value.at("effect", default: none)
  if effect != "neutral" and _says-why(value.at("reason", default: none)) {
    messages.push("goal `" + name + "` says " + str(effect) + " but not why")
  }
  messages
}

// `[[tasks.rules]] check` entry point for `field = "markers"`: the body's goal
// markers, filtered out of every other marker, must rate every goal of both
// registries once, under the audience that registry declares, with a reason on
// a `Helps`/`Hinders` and at least one `Helps`. Returns an array of breach
// strings — empty means the task passes. A task whose body emits no marker
// arrives as `[]`, so it is refused like one that rates nothing.
#let check-goal-over(markers) = {
  if type(markers) != array {
    return ("the goal check reads the body's markers; got " + repr(markers),)
  }
  let known = _goals-by-name()
  let messages = ()
  let rated = ()
  let helped = false
  for marker in markers {
    if type(marker) != dictionary {
      continue
    }
    let value = marker.at("value", default: none)
    if type(value) != dictionary {
      continue
    }
    if value.at("marker", default: none) != goal-marker {
      continue
    }
    let name = value.at("goal", default: none)
    if type(name) != str {
      messages.push(
        "a goal marker names no goal; write #Helps(..), #Hinders(..) or #Neutral(..)",
      )
      continue
    }
    if name in rated {
      messages.push("goal `" + name + "` is rated twice; rate each goal once")
    }
    rated.push(name)
    let goal = known.at(name, default: none)
    if goal == none {
      messages.push(
        "unknown goal `" + name + "`; they are " + known.keys().join(", "),
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
      messages.push(
        "goal `"
          + name
          + "` is not rated; write #Helps(..), #Hinders(..) or #Neutral(..)",
      )
    }
  }
  if not helped {
    messages.push("the task helps no goal; if nothing justifies it, cancel it")
  }
  messages
}
