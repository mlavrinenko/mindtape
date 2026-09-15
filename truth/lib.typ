// Helpers every open task imports. `goals(...)` checks a task's verdicts
// against both goal sets, goals.typ (what MindTape is for) and
// process-goals.typ (how this repo builds it), and renders them as a list.
// It refuses a task that skips a goal, names an unknown one, gives helps or
// hurts no reason, or helps none: a task nothing justifies is cancelled, not
// filed.

#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

#let helps(reason) = (effect: "helps", reason: reason)
#let hurts(reason) = (effect: "hurts", reason: reason)
#let neutral = (effect: "neutral", reason: none)

#let _markers = (helps: "👍", hurts: "👎", neutral: "➖")
#let _choices = "helps[...], hurts[...] or neutral"

#let _check(group, registry, given) = {
  if type(given) != dictionary {
    panic("goals(...) needs " + group + ": (...), one verdict per goal")
  }
  for key in registry.keys() {
    if key not in given {
      panic(group + " goal `" + key + "` is not addressed; say " + _choices)
    }
  }
  for (key, value) in given {
    if key not in registry {
      panic(
        "unknown "
          + group
          + " goal `"
          + key
          + "`; they are "
          + registry.keys().join(", "),
      )
    }
    if type(value) != dictionary or "effect" not in value {
      panic(group + " goal `" + key + "` takes " + _choices)
    }
    if value.effect != "neutral" and value.reason in (none, [], [ ]) {
      panic(group + " goal `" + key + "` says " + value.effect + " but not how")
    }
  }
}

#let _items(registry, given) = {
  given
    .pairs()
    .map(((key, value)) => {
      let title = strong(registry.at(key).title)
      let marker = _markers.at(value.effect)
      if (
        value.reason == none
      ) [#marker #title] else [#marker #title: #value.reason]
    })
}

#let goals(product: none, process: none) = {
  _check("product", product-goals, product)
  _check("process", process-goals, process)
  let verdicts = product.values() + process.values()
  if verdicts.all(value => value.effect != "helps") {
    panic("the task helps no goal; if nothing justifies it, cancel it")
  }
  list(.._items(product-goals, product), .._items(process-goals, process))
}
