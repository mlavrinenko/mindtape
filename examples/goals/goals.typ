// The goals this project's tasks rate. goal-dsl.typ turns each value into the
// name a task writes, and lib.typ checks the task body's markers against this
// registry.

#import "goal-dsl.typ": goal

// docs:slice goals
#let goals = (
  answer-fast: goal(
    to: "answer fast",
    from: "answering fast",
    title: "The app answers fast",
  ),
  avoid-lock-in: goal(
    to: "avoid lock-in",
    from: "avoiding lock-in",
    title: "Nothing traps a switcher",
  ),
)
// docs:end
