// The function the `goals` rule in .mindtape/config.toml calls on each task.

// docs:slice check-goals
#import "goals.typ": goals

// `value` is the task's `x-goals`, or `none` when the task has none. Each
// string returned is one breach, and an empty array passes the task.
#let check-goals(value) = {
  if value == none {
    return ("x-goals is missing; add one verdict per goal in goals.typ",)
  }
  goals
    .keys()
    .filter(key => key not in value)
    .map(key => "goal `" + key + "` is not addressed")
}
// docs:end
