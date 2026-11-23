// The function the `goals` rule in .mindtape/config.toml calls on each task.
// docs:slice lib
#import "goal-dsl.typ": check-goal-over
#import "goals.typ": goals

// `markers` is the task body's metadata markers, the reserved `check` input.
#let check-goals(markers) = check-goal-over(goals, markers)
// docs:end
