// examples/self-validating/lib.typ — the function `[[tasks.rules]] check`
// calls against every task's `x-goals` field. Mirrors truth/lib.typ's split
// of code (here) from data (goals.typ), pared down to the one rule this page
// demonstrates: every goal in goals.typ needs a verdict.

#import "goals.typ": goals

// `[[tasks.rules]] check` entry point for `field = "x-goals"`: `value` is
// `none` when the task carries no `x-goals` at all. Returns an array of
// breach strings — empty means the task passes.
#let check-goals(value) = {
  if value == none {
    return ("x-goals is missing; add one verdict per goal in goals.typ",)
  }
  goals
    .keys()
    .filter(key => key not in value)
    .map(key => "goal `" + key + "` is not addressed")
}
