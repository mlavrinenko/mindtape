// The goal checks every open task imports. `check-goals(value)` checks a task's
// `x-goals` field against both goal sets, goals.typ (what Mindtape is for)
// and process-goals.typ (how this repo builds it), returning one message per
// problem instead of panicking: a `[[tasks.rules]] check` rule collects every
// string it returns as a breach, so several omissions surface together
// rather than one panic at a time. A task nothing justifies (helps no goal)
// is refused, not filed.
//
// `check-goal-over(markers)` is the same check over the body's goal markers, the
// DSL in goal-dsl.typ that a task writes as `#Helps`/`#Hinders`/`#Neutral`. It
// reads the reserved `markers` rule input and sits beside `check-goals` until
// the migration retires the field rule; no rule in this repo reads it yet.
//
// The verdict helpers come from the prelude, so an adopter writes the same
// dict a task does without copying this file; this module re-exports them and
// keeps only what is true of this repo.

#import "@local/mindtape:0.3.0": edge, helps, hurts, neutral
#import "goal-dsl.typ": goal-marker
#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

// A finding points at the task that surfaced it. The kind is declared in
// `.mindtape/config.toml`; this helper is the project-side spelling of
// `edge("surfaced-by", ...)`, beside the goal check every task imports.
#let surfaced-by(..args) = edge("surfaced-by", ..args)

#let _choices = "helps[...], hurts[...] or neutral"

// One group's worth of breaches ("product" or "process"): every registry
// goal not addressed, every addressed key the registry does not recognize,
// every value that is not helps[...]/hurts[...]/neutral, and every
// helps/hurts with no reason. Returns (messages, effects) — effects is the
// `effect` string of every well-shaped entry, so the caller can tell whether
// anything in either group helps.
#let _check-group(group, registry, given) = {
  if given == none {
    return (
      (
        group
          + " is missing from x-goals; add "
          + group
          + ": (...), one verdict per goal",
      ),
      (),
    )
  }
  let messages = ()
  let effects = ()
  for key in registry.keys() {
    if key not in given {
      messages.push(
        group + " goal `" + key + "` is not addressed; say " + _choices,
      )
    }
  }
  for (key, value) in given {
    if key not in registry {
      messages.push(
        "unknown "
          + group
          + " goal `"
          + key
          + "`; they are "
          + registry.keys().join(", "),
      )
      continue
    }
    if type(value) != dictionary or "effect" not in value {
      messages.push(group + " goal `" + key + "` takes " + _choices)
      continue
    }
    effects.push(value.effect)
    if (
      value.effect != "neutral"
        and value.at("reason", default: none) in (none, "", [], [ ])
    ) {
      messages.push(
        group + " goal `" + key + "` says " + value.effect + " but not how",
      )
    }
  }
  (messages, effects)
}

// `[[tasks.rules]] check` entry point for `field = "x-goals"`: `value` is
// `none` when the task carries no `x-goals` at all. Returns an array of
// breach strings — empty means the task passes.
#let check-goals(value) = {
  if value == none or type(value) != dictionary {
    return (
      "x-goals is missing; add x-goals: (product: (...), process: (...))",
    )
  }
  let (product-messages, product-effects) = _check-group(
    "product",
    product-goals,
    value.at("product", default: none),
  )
  let (process-messages, process-effects) = _check-group(
    "process",
    process-goals,
    value.at("process", default: none),
  )
  let messages = product-messages + process-messages
  let effects = product-effects + process-effects
  if effects.len() > 0 and effects.all(effect => effect != "helps") {
    messages.push("the task helps no goal; if nothing justifies it, cancel it")
  }
  messages
}

// Every goal of both registries, keyed by the name a task writes (the kebab of
// its infinitive) rather than the registry key, which still spells the old
// `x-goals` name until the migration. A goal's audience says which set it came
// from, so two registries answer to one namespace.
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
