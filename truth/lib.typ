// The goal check every open task imports. `check-goals(value)` checks a task's
// `x-goals` field against both goal sets, goals.typ (what Mindtape is for)
// and process-goals.typ (how this repo builds it), returning one message per
// problem instead of panicking: a `[[tasks.rules]] check` rule collects every
// string it returns as a breach, so several omissions surface together
// rather than one panic at a time. A task nothing justifies (helps no goal)
// is refused, not filed.
//
// The verdict helpers come from the prelude, so an adopter writes the same
// dict a task does without copying this file; this module re-exports them and
// keeps only what is true of this repo.

#import "@local/mindtape:0.3.0": edge, helps, hurts, neutral
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
