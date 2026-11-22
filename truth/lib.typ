// The goal check every open task imports. `check-goal-over(markers)` checks a
// task body's goal markers against both goal sets, goals.typ (what Mindtape is
// for) and process-goals.typ (how this repo builds it), returning one message
// per problem instead of panicking: a `[[tasks.rules]] check` rule collects
// every string it returns as a breach, so several omissions surface together
// rather than one panic at a time. A task nothing justifies (helps no goal) is
// refused, not filed.
//
// The check itself lives in the prelude and takes the two registries, because a
// project's goals are its own. This module wraps it with this repo's product and
// process goals. `check-goal-over` reads the reserved `markers` rule input, the
// DSL a task writes as `#Helps`/`#Hinders`/`#Neutral`. The two registries fix
// each goal's audience, which is why the wrapper passes them here rather than
// letting the generic check guess.

#import "@local/mindtape:0.5.0": edge
#import "@local/mindtape:0.5.0": check-goal-over as check-goals-over-registries
#import "goals.typ": goals as product-goals
#import "process-goals.typ": goals as process-goals

// A finding points at the task that surfaced it. The kind is declared in
// `.mindtape/config.toml`; this helper is the project-side spelling of
// `edge("surfaced-by", ...)`, beside the goal check every task imports.
#let surfaced-by(..args) = edge("surfaced-by", ..args)

// The live rule's entry point: the prelude check over this repo's two goal
// registries. The function name stays `check-goal-over`, so the config's rule
// points at this module unchanged.
#let check-goal-over(markers) = check-goals-over-registries(
  product-goals,
  process-goals,
  markers,
)
